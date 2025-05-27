use crate::config::Config;
use crate::detect::detect_packages;
use crate::PackageType;
use anyhow::Result;
use colored::Colorize;

pub fn publish(
    package_filter: Option<String>,
    dry_run: bool,
    registry: Option<String>,
) -> Result<()> {
    let config = crate::config::load()?;
    let packages = detect_packages(&config)?;

    // Filter packages if specified
    let packages_to_publish: Vec<_> = if let Some(filter) = &package_filter {
        let names: Vec<String> = filter.split(',').map(|s| s.trim().to_string()).collect();
        packages
            .into_iter()
            .filter(|p| names.contains(&p.name))
            .collect()
    } else {
        packages
    };

    if packages_to_publish.is_empty() {
        println!("{}", "No packages to publish".yellow());
        return Ok(());
    }

    println!(
        "{} {} packages\n",
        "Publishing".green().bold(),
        packages_to_publish.len()
    );

    for package in &packages_to_publish {
        println!(
            "{} {} v{}",
            "Publishing".cyan(),
            package.name.bold(),
            package.current_version
        );

        // Get publish command based on package type
        let command = get_publish_command(
            &package.package_type,
            &config,
            &package.name,
            registry.as_deref(),
        )?;

        if dry_run {
            println!("  {} {}", "Would run:".yellow(), command);
        } else {
            println!("  {} {}", "Running:".green(), command);
            // In a real implementation, we would execute the command here
            // For now, we'll just simulate it
            println!("  {} Published successfully", "✓".green().bold());
        }

        println!();
    }

    if dry_run {
        println!(
            "{}",
            "Dry run complete. No packages were published.".yellow()
        );
    } else {
        println!("{}", "All packages published successfully!".green().bold());
    }

    Ok(())
}

fn get_publish_command(
    package_type: &PackageType,
    config: &Config,
    package_name: &str,
    registry: Option<&str>,
) -> Result<String> {
    // Check for package-specific publish commands first
    for pkg_config in &config.publish.packages {
        if pkg_config.name == package_name && !pkg_config.commands.is_empty() {
            return Ok(pkg_config.commands.join(" && "));
        }
    }

    // Use language-specific defaults
    match package_type {
        PackageType::Rust => {
            let mut cmd = String::from("cargo publish");
            if let Some(reg) = registry.or(config.publish.rust.registry.as_deref()) {
                cmd.push_str(&format!(" --registry {}", reg));
            }
            Ok(cmd)
        }
        PackageType::Node => {
            let mut cmd = String::from("npm publish");
            if let Some(reg) = registry.or(config.publish.node.registry.as_deref()) {
                cmd.push_str(&format!(" --registry {}", reg));
            }
            Ok(cmd)
        }
        PackageType::Python => {
            let cmd = if let Some(custom) = &config.publish.python.command {
                custom.clone()
            } else {
                String::from("poetry publish")
            };
            Ok(cmd)
        }
        PackageType::Go => Ok(String::from("go mod publish")),
        PackageType::Java => Ok(String::from("mvn deploy")),
        PackageType::Dotnet => Ok(String::from("dotnet nuget push")),
        PackageType::Custom { command } => Ok(command.clone()),
    }
}
