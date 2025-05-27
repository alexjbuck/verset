use crate::{Package, PackageType};
use anyhow::Result;
use semver::Version;
use std::fs;

pub fn update_version(package: &Package, new_version: &Version) -> Result<()> {
    let content = fs::read_to_string(&package.version_file)?;

    let new_content = match &package.package_type {
        PackageType::Rust => update_rust_version(&content, new_version)?,
        PackageType::Node => update_node_version(&content, new_version)?,
        PackageType::Python => {
            if package.version_file.file_name() == Some(std::ffi::OsStr::new("__init__.py")) {
                update_python_init_version(&content, new_version)?
            } else {
                update_toml_version(&content, &package.version_path, new_version)?
            }
        }
        _ => {
            // Try to detect format and update
            if toml::from_str::<toml::Value>(&content).is_ok() {
                update_toml_version(&content, &package.version_path, new_version)?
            } else if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                update_json_version(&content, &package.version_path, new_version)?
            } else {
                return Err(anyhow::anyhow!(
                    "Unable to update version in {}",
                    package.version_file.display()
                ));
            }
        }
    };

    fs::write(&package.version_file, new_content)?;
    Ok(())
}

fn update_rust_version(content: &str, new_version: &Version) -> Result<String> {
    let mut cargo: toml::Value = toml::from_str(content)?;

    cargo["package"]["version"] = toml::Value::String(new_version.to_string());

    Ok(toml::to_string_pretty(&cargo)?)
}

fn update_node_version(content: &str, new_version: &Version) -> Result<String> {
    let mut package: serde_json::Value = serde_json::from_str(content)?;

    package["version"] = serde_json::Value::String(new_version.to_string());

    Ok(serde_json::to_string_pretty(&package)?)
}

fn update_python_init_version(content: &str, new_version: &Version) -> Result<String> {
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut updated = false;

    for line in &mut lines {
        if line.contains("__version__") {
            if let Some(eq_pos) = line.find('=') {
                let prefix = &line[..=eq_pos];
                let quote_char = if line.contains('"') { '"' } else { '\'' };
                *line = format!("{} {}{}{}", prefix, quote_char, new_version, quote_char);
                updated = true;
                break;
            }
        }
    }

    if !updated {
        return Err(anyhow::anyhow!("Could not find __version__ in __init__.py"));
    }

    Ok(lines.join("\n"))
}

fn update_toml_version(content: &str, version_path: &str, new_version: &Version) -> Result<String> {
    let mut value: toml::Value = toml::from_str(content)?;

    set_nested_toml_value(
        &mut value,
        version_path,
        toml::Value::String(new_version.to_string()),
    )?;

    Ok(toml::to_string_pretty(&value)?)
}

fn update_json_version(content: &str, version_path: &str, new_version: &Version) -> Result<String> {
    let mut value: serde_json::Value = serde_json::from_str(content)?;

    set_nested_json_value(
        &mut value,
        version_path,
        serde_json::Value::String(new_version.to_string()),
    )?;

    Ok(serde_json::to_string_pretty(&value)?)
}

fn set_nested_toml_value(
    value: &mut toml::Value,
    path: &str,
    new_value: toml::Value,
) -> Result<()> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let toml::Value::Table(table) = current {
                table.insert((*part).to_string(), new_value);
                return Ok(());
            }
            return Err(anyhow::anyhow!("Expected table at path"));
        }
        // Navigate deeper
        current = current
            .get_mut(part)
            .ok_or_else(|| anyhow::anyhow!("Path {} not found", part))?;
    }

    Ok(())
}

fn set_nested_json_value(
    value: &mut serde_json::Value,
    path: &str,
    new_value: serde_json::Value,
) -> Result<()> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let serde_json::Value::Object(map) = current {
                map.insert((*part).to_string(), new_value);
                return Ok(());
            }
            return Err(anyhow::anyhow!("Expected object at path"));
        }
        // Navigate deeper
        current = current
            .get_mut(part)
            .ok_or_else(|| anyhow::anyhow!("Path {} not found", part))?;
    }

    Ok(())
}
