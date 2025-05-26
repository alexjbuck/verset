use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use glob::glob;
use crate::{Package, PackageType};
use crate::config::Config;
use semver::Version;

pub fn detect_packages(config: &Config) -> Result<Vec<Package>> {
    let mut packages = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();
    
    // First, add manual packages
    for manual in &config.packages.manual {
        let path = PathBuf::from(&manual.path);
        if seen_paths.insert(path.clone()) {
            let package_type = parse_package_type(&manual.package_type)?;
            let version_file = path.join(&manual.version_file);
            let current_version = read_version(&version_file, &package_type, &manual.version_path)?;
            
            packages.push(Package {
                name: manual.name.clone(),
                path,
                package_type,
                version_file,
                version_path: manual.version_path.clone(),
                current_version,
            });
        }
    }
    
    // Then, auto-detect based on patterns
    for pattern in &config.packages.patterns {
        for entry in glob(pattern)? {
            let path = entry?;
            if path.is_dir() && seen_paths.insert(path.clone()) {
                if let Some(package) = detect_package_in_dir(&path)? {
                    packages.push(package);
                }
            }
        }
    }
    
    // Sort packages by name for consistent output
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(packages)
}

fn parse_package_type(type_str: &str) -> Result<PackageType> {
    match type_str.to_lowercase().as_str() {
        "rust" => Ok(PackageType::Rust),
        "node" | "nodejs" => Ok(PackageType::Node),
        "python" => Ok(PackageType::Python),
        "go" => Ok(PackageType::Go),
        "java" => Ok(PackageType::Java),
        "dotnet" | ".net" => Ok(PackageType::Dotnet),
        _ => Err(anyhow::anyhow!("Unknown package type: {}", type_str)),
    }
}

fn detect_package_in_dir(dir: &Path) -> Result<Option<Package>> {
    // Check for Rust project
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() {
        return detect_rust_package(dir, &cargo_toml);
    }
    
    // Check for Node.js project
    let package_json = dir.join("package.json");
    if package_json.exists() {
        return detect_node_package(dir, &package_json);
    }
    
    // Check for Python project
    let pyproject_toml = dir.join("pyproject.toml");
    if pyproject_toml.exists() {
        return detect_python_package(dir, &pyproject_toml);
    }
    
    // Check for Go project
    let go_mod = dir.join("go.mod");
    if go_mod.exists() {
        return detect_go_package(dir, &go_mod);
    }
    
    // Check for Maven project
    let pom_xml = dir.join("pom.xml");
    if pom_xml.exists() {
        return detect_maven_package(dir, &pom_xml);
    }
    
    // Check for Gradle project
    let build_gradle = dir.join("build.gradle");
    let build_gradle_kts = dir.join("build.gradle.kts");
    if build_gradle.exists() || build_gradle_kts.exists() {
        let gradle_file = if build_gradle.exists() { build_gradle } else { build_gradle_kts };
        return detect_gradle_package(dir, &gradle_file);
    }
    
    // Check for .NET project
    let csproj_files: Vec<_> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "csproj")
                .unwrap_or(false)
        })
        .collect();
    
    if let Some(csproj) = csproj_files.first() {
        return detect_dotnet_package(dir, &csproj.path());
    }
    
    Ok(None)
}

fn detect_rust_package(dir: &Path, cargo_toml: &Path) -> Result<Option<Package>> {
    let content = fs::read_to_string(cargo_toml)?;
    let cargo: toml::Value = toml::from_str(&content)?;
    
    let package = cargo.get("package").context("No [package] section in Cargo.toml")?;
    let name = package.get("name")
        .and_then(|v| v.as_str())
        .context("No package name in Cargo.toml")?;
    let version_str = package.get("version")
        .and_then(|v| v.as_str())
        .context("No package version in Cargo.toml")?;
    
    let version = Version::parse(version_str)?;
    
    Ok(Some(Package {
        name: name.to_string(),
        path: dir.to_path_buf(),
        package_type: PackageType::Rust,
        version_file: cargo_toml.to_path_buf(),
        version_path: "package.version".to_string(),
        current_version: version,
    }))
}

fn detect_node_package(dir: &Path, package_json: &Path) -> Result<Option<Package>> {
    let content = fs::read_to_string(package_json)?;
    let package: serde_json::Value = serde_json::from_str(&content)?;
    
    let name = package.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| dir.file_name().unwrap().to_str().unwrap());
    let version_str = package.get("version")
        .and_then(|v| v.as_str())
        .context("No version in package.json")?;
    
    let version = Version::parse(version_str)?;
    
    Ok(Some(Package {
        name: name.to_string(),
        path: dir.to_path_buf(),
        package_type: PackageType::Node,
        version_file: package_json.to_path_buf(),
        version_path: "version".to_string(),
        current_version: version,
    }))
}

fn detect_python_package(dir: &Path, pyproject_toml: &Path) -> Result<Option<Package>> {
    let content = fs::read_to_string(pyproject_toml)?;
    let pyproject: toml::Value = toml::from_str(&content)?;
    
    // Try [project] section first (PEP 621)
    if let Some(project) = pyproject.get("project") {
        let name = project.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| dir.file_name().unwrap().to_str().unwrap());
        
        if let Some(version_str) = project.get("version").and_then(|v| v.as_str()) {
            let version = Version::parse(version_str)?;
            return Ok(Some(Package {
                name: name.to_string(),
                path: dir.to_path_buf(),
                package_type: PackageType::Python,
                version_file: pyproject_toml.to_path_buf(),
                version_path: "project.version".to_string(),
                current_version: version,
            }));
        }
    }
    
    // Try [tool.poetry] section
    if let Some(tool) = pyproject.get("tool") {
        if let Some(poetry) = tool.get("poetry") {
            let name = poetry.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| dir.file_name().unwrap().to_str().unwrap());
            
            if let Some(version_str) = poetry.get("version").and_then(|v| v.as_str()) {
                let version = Version::parse(version_str)?;
                return Ok(Some(Package {
                    name: name.to_string(),
                    path: dir.to_path_buf(),
                    package_type: PackageType::Python,
                    version_file: pyproject_toml.to_path_buf(),
                    version_path: "tool.poetry.version".to_string(),
                    current_version: version,
                }));
            }
        }
    }
    
    // Check for __init__.py with __version__
    let init_py = dir.join("__init__.py");
    if init_py.exists() {
        let content = fs::read_to_string(&init_py)?;
        if let Some(version_str) = extract_python_version(&content) {
            let version = Version::parse(&version_str)?;
            let name = dir.file_name().unwrap().to_str().unwrap();
            return Ok(Some(Package {
                name: name.to_string(),
                path: dir.to_path_buf(),
                package_type: PackageType::Python,
                version_file: init_py,
                version_path: "__version__".to_string(),
                current_version: version,
            }));
        }
    }
    
    Ok(None)
}

fn extract_python_version(content: &str) -> Option<String> {
    // Look for __version__ = "x.y.z" pattern
    for line in content.lines() {
        if line.contains("__version__") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        return Some(line[start + 1..end].to_string());
                    }
                }
            } else if let Some(start) = line.find('\'') {
                if let Some(end) = line.rfind('\'') {
                    if start < end {
                        return Some(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }
    None
}

fn detect_go_package(dir: &Path, go_mod: &Path) -> Result<Option<Package>> {
    let _content = fs::read_to_string(go_mod)?;
    let name = dir.file_name().unwrap().to_str().unwrap();
    
    // Go doesn't have versions in go.mod for the module itself
    // We'll need to look for a version.go or similar file
    let version_file = dir.join("version.go");
    if version_file.exists() {
        let version_content = fs::read_to_string(&version_file)?;
        if let Some(version_str) = extract_go_version(&version_content) {
            let version = Version::parse(&version_str)?;
            return Ok(Some(Package {
                name: name.to_string(),
                path: dir.to_path_buf(),
                package_type: PackageType::Go,
                version_file,
                version_path: "Version".to_string(),
                current_version: version,
            }));
        }
    }
    
    // Default to 0.0.0 if no version found
    Ok(Some(Package {
        name: name.to_string(),
        path: dir.to_path_buf(),
        package_type: PackageType::Go,
        version_file: go_mod.to_path_buf(),
        version_path: String::new(),
        current_version: Version::parse("0.0.0")?,
    }))
}

fn extract_go_version(content: &str) -> Option<String> {
    // Look for const Version = "x.y.z" pattern
    for line in content.lines() {
        if line.contains("Version") && line.contains('=') {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        return Some(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }
    None
}

fn detect_maven_package(dir: &Path, pom_xml: &Path) -> Result<Option<Package>> {
    // For now, return a placeholder
    // Full XML parsing would require an XML library
    let name = dir.file_name().unwrap().to_str().unwrap();
    Ok(Some(Package {
        name: name.to_string(),
        path: dir.to_path_buf(),
        package_type: PackageType::Java,
        version_file: pom_xml.to_path_buf(),
        version_path: "project.version".to_string(),
        current_version: Version::parse("0.0.0")?,
    }))
}

fn detect_gradle_package(dir: &Path, gradle_file: &Path) -> Result<Option<Package>> {
    // For now, return a placeholder
    // Full Gradle parsing would be complex
    let name = dir.file_name().unwrap().to_str().unwrap();
    Ok(Some(Package {
        name: name.to_string(),
        path: dir.to_path_buf(),
        package_type: PackageType::Java,
        version_file: gradle_file.to_path_buf(),
        version_path: "version".to_string(),
        current_version: Version::parse("0.0.0")?,
    }))
}

fn detect_dotnet_package(dir: &Path, csproj: &Path) -> Result<Option<Package>> {
    // For now, return a placeholder
    // Full XML parsing would require an XML library
    let name = csproj.file_stem().unwrap().to_str().unwrap();
    Ok(Some(Package {
        name: name.to_string(),
        path: dir.to_path_buf(),
        package_type: PackageType::Dotnet,
        version_file: csproj.to_path_buf(),
        version_path: "Version".to_string(),
        current_version: Version::parse("0.0.0")?,
    }))
}

pub fn read_version(version_file: &Path, package_type: &PackageType, version_path: &str) -> Result<Version> {
    let content = fs::read_to_string(version_file)?;
    
    match package_type {
        PackageType::Rust => {
            let cargo: toml::Value = toml::from_str(&content)?;
            let version_str = cargo.get("package")
                .and_then(|p| p.get("version"))
                .and_then(|v| v.as_str())
                .context("Failed to read version from Cargo.toml")?;
            Version::parse(version_str).context("Invalid version in Cargo.toml")
        }
        PackageType::Node => {
            let package: serde_json::Value = serde_json::from_str(&content)?;
            let version_str = package.get("version")
                .and_then(|v| v.as_str())
                .context("Failed to read version from package.json")?;
            Version::parse(version_str).context("Invalid version in package.json")
        }
        PackageType::Python => {
            if version_file.file_name() == Some(std::ffi::OsStr::new("__init__.py")) {
                if let Some(version_str) = extract_python_version(&content) {
                    return Version::parse(&version_str).context("Invalid version in __init__.py");
                }
            } else {
                let pyproject: toml::Value = toml::from_str(&content)?;
                let version_str = get_nested_value(&pyproject, version_path)
                    .and_then(|v| v.as_str())
                    .context("Failed to read version from pyproject.toml")?;
                return Version::parse(version_str).context("Invalid version in pyproject.toml");
            }
            Err(anyhow::anyhow!("Failed to read Python version"))
        }
        _ => {
            // For other types, try to parse as TOML or JSON
            if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
                let version_str = get_nested_value(&toml_value, version_path)
                    .and_then(|v| v.as_str())
                    .context("Failed to read version from TOML file")?;
                Version::parse(version_str).context("Invalid version in TOML file")
            } else if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
                let version_str = get_nested_json_value(&json_value, version_path)
                    .and_then(|v| v.as_str())
                    .context("Failed to read version from JSON file")?;
                Version::parse(version_str).context("Invalid version in JSON file")
            } else {
                Err(anyhow::anyhow!("Unable to parse version file"))
            }
        }
    }
}

fn get_nested_value<'a>(value: &'a toml::Value, path: &str) -> Option<&'a toml::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    
    for part in parts {
        current = current.get(part)?;
    }
    
    Some(current)
}

fn get_nested_json_value<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    
    for part in parts {
        current = current.get(part)?;
    }
    
    Some(current)
} 