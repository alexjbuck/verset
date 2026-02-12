use crate::detect::detect_packages;
use crate::{ChangeType, Changeset};
use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ChangesetFile {
    packages: Vec<String>,
    #[serde(rename = "type")]
    change_type: String,
}

pub fn add_non_interactive(
    package: Option<String>,
    change_type: Option<String>,
    message: Option<String>,
    all: bool,
) -> Result<()> {
    let config = crate::config::load()?;
    let packages = detect_packages(&config)?;

    if packages.is_empty() {
        return Err(anyhow::anyhow!("No packages found"));
    }

    // Determine target packages
    let target_packages = if all {
        packages.iter().map(|p| p.name.clone()).collect()
    } else if let Some(package_names) = package {
        let names: Vec<String> = package_names
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        // Validate package names
        for name in &names {
            if !packages.iter().any(|p| &p.name == name) {
                return Err(anyhow::anyhow!("Unknown package: {}", name));
            }
        }
        names
    } else {
        return Err(anyhow::anyhow!(
            "Must specify --package, --all, or use interactive mode"
        ));
    };

    // Parse change type
    let change_type = change_type
        .ok_or_else(|| anyhow::anyhow!("Must specify --type"))?
        .parse::<ChangeType>()?;

    // Get message
    let message = message.ok_or_else(|| anyhow::anyhow!("Must specify --message"))?;

    // Create changeset
    let changeset = Changeset {
        packages: target_packages,
        change_type,
        description: message,
        created_at: Utc::now(),
    };

    save_changeset(&changeset)?;

    Ok(())
}

pub fn save_changeset(changeset: &Changeset) -> Result<()> {
    let changesets_dir = Path::new(".verset/changesets");
    if !changesets_dir.exists() {
        return Err(anyhow::anyhow!(
            "Changesets directory not found. Run {} first.",
            "verset init".cyan()
        ));
    }

    // Generate filename
    let timestamp = changeset.created_at.format("%Y%m%d%H%M%S");
    let id = Uuid::new_v4().to_string();
    let id = id.split('-').next().unwrap_or(&id).to_string();
    let filename = format!("{timestamp}-{id}.md");
    let filepath = changesets_dir.join(&filename);

    // Create frontmatter
    let frontmatter = ChangesetFile {
        packages: changeset.packages.clone(),
        change_type: changeset.change_type.to_string(),
    };

    // Create file content
    let content = format!(
        "---\n{}\n---\n\n{}",
        serde_yaml::to_string(&frontmatter)?.trim(),
        changeset.description
    );

    // Write file
    fs::write(&filepath, content)?;

    println!(
        "{} Created changeset: {}",
        "✓".green().bold(),
        filename.cyan()
    );
    println!("\nChangeset summary:");
    println!("  {} {}", "Packages:".bold(), changeset.packages.join(", "));
    println!("  {} {}", "Type:".bold(), changeset.change_type);
    println!("  {} {}", "Description:".bold(), changeset.description);

    Ok(())
}

pub fn load_changesets() -> Result<Vec<(PathBuf, Changeset)>> {
    let changesets_dir = Path::new(".verset/changesets");
    if !changesets_dir.exists() {
        return Ok(vec![]);
    }

    let mut changesets = Vec::new();

    for entry in fs::read_dir(changesets_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path)?;

            // Parse frontmatter
            if let Some(stripped) = content.strip_prefix("---\n") {
                // Find the end of frontmatter
                if let Some(end_pos) = stripped.find("\n---\n") {
                    let frontmatter = &stripped[..end_pos];
                    let description = stripped[end_pos + 5..].trim().to_string();

                    let changeset_file: ChangesetFile = serde_yaml::from_str(frontmatter)
                        .context(format!("Failed to parse changeset: {}", path.display()))?;

                    let changeset = Changeset {
                        packages: changeset_file.packages,
                        change_type: changeset_file.change_type.parse()?,
                        description,
                        created_at: Utc::now(), // We could parse from filename if needed
                    };

                    changesets.push((path, changeset));
                }
            }
        }
    }

    // Sort by filename (which includes timestamp)
    changesets.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(changesets)
}

pub fn status(package_filter: Option<String>) -> Result<()> {
    let config = crate::config::load()?;
    let packages = detect_packages(&config)?;
    let changesets = load_changesets()?;

    if changesets.is_empty() {
        println!("{}", "No pending changesets".yellow());
        return Ok(());
    }

    println!(
        "{} {} pending changesets\n",
        "Found".green().bold(),
        changesets.len()
    );

    // Calculate version impacts
    let mut version_impacts: std::collections::HashMap<String, Vec<ChangeType>> =
        std::collections::HashMap::new();

    for (_, changeset) in &changesets {
        for package_name in &changeset.packages {
            if let Some(filter) = &package_filter {
                if package_name != filter {
                    continue;
                }
            }
            version_impacts
                .entry(package_name.clone())
                .or_default()
                .push(changeset.change_type.clone());
        }
    }

    // Display current versions and impacts
    println!("{}", "Package versions:".cyan().bold());
    for package in &packages {
        if let Some(filter) = &package_filter {
            if &package.name != filter {
                continue;
            }
        }

        let current = &package.current_version;
        let impacts = version_impacts.get(&package.name);

        if let Some(changes) = impacts {
            let next_version = calculate_next_version(current, changes);
            println!(
                "  {} {} → {} ({})",
                package.name.bold(),
                current.to_string().red(),
                next_version.to_string().green(),
                format_changes(changes)
            );
        } else {
            println!(
                "  {} {} (no changes)",
                package.name.bold(),
                current.to_string().white()
            );
        }
    }

    // Display changeset details
    println!("\n{}", "Changesets:".cyan().bold());
    for (path, changeset) in &changesets {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let should_show = if let Some(filter) = &package_filter {
            changeset.packages.contains(filter)
        } else {
            true
        };

        if should_show {
            println!("\n  {} {}", "•".cyan(), filename.yellow());
            println!(
                "    {} {}",
                "Packages:".bold(),
                changeset.packages.join(", ")
            );
            println!("    {} {}", "Type:".bold(), changeset.change_type);
            println!("    {} {}", "Description:".bold(), changeset.description);
        }
    }

    Ok(())
}

fn calculate_next_version(current: &semver::Version, changes: &[ChangeType]) -> semver::Version {
    let mut next = current.clone();

    // Find the highest priority change
    let has_major = changes.iter().any(|c| matches!(c, ChangeType::Major));
    let has_minor = changes.iter().any(|c| matches!(c, ChangeType::Minor));
    let has_patch = changes.iter().any(|c| matches!(c, ChangeType::Patch));

    if has_major {
        next.major += 1;
        next.minor = 0;
        next.patch = 0;
    } else if has_minor {
        next.minor += 1;
        next.patch = 0;
    } else if has_patch {
        next.patch += 1;
    }

    // Clear pre-release and build metadata
    next.pre = semver::Prerelease::EMPTY;
    next.build = semver::BuildMetadata::EMPTY;

    next
}

fn format_changes(changes: &[ChangeType]) -> String {
    let mut counts = std::collections::HashMap::new();
    for change in changes {
        *counts.entry(change.to_string()).or_insert(0) += 1;
    }

    let mut parts = Vec::new();
    for (change_type, count) in counts {
        if count > 1 {
            parts.push(format!("{} {}s", count, change_type));
        } else {
            parts.push(format!("{} {}", count, change_type));
        }
    }

    parts.join(", ")
}

pub fn apply(package_filter: Option<String>, dry_run: bool, no_changelog: bool) -> Result<()> {
    let config = crate::config::load()?;
    let mut packages = detect_packages(&config)?;
    let changesets = load_changesets()?;

    if changesets.is_empty() {
        println!("{}", "No changesets to apply".yellow());
        return Ok(());
    }

    // Calculate version updates
    let mut updates: std::collections::HashMap<String, (semver::Version, Vec<ChangeType>)> =
        std::collections::HashMap::new();

    for (_, changeset) in &changesets {
        for package_name in &changeset.packages {
            if let Some(filter) = &package_filter {
                if package_name != filter {
                    continue;
                }
            }

            if let Some(package) = packages.iter().find(|p| &p.name == package_name) {
                let entry = updates
                    .entry(package_name.clone())
                    .or_insert_with(|| (package.current_version.clone(), Vec::new()));
                entry.1.push(changeset.change_type.clone());
            }
        }
    }

    if updates.is_empty() {
        println!("{}", "No packages to update".yellow());
        return Ok(());
    }

    // Calculate new versions
    let mut version_updates = Vec::new();
    for (package_name, (current_version, changes)) in &updates {
        let new_version = calculate_next_version(current_version, changes);
        version_updates.push((package_name.clone(), current_version.clone(), new_version));
    }

    // Display what will be done
    println!("{}", "Version updates:".cyan().bold());
    for (package_name, current, new) in &version_updates {
        println!(
            "  {} {} → {}",
            package_name.bold(),
            current.to_string().red(),
            new.to_string().green()
        );
    }

    if dry_run {
        println!("\n{}", "Dry run complete. No changes were made.".yellow());
        return Ok(());
    }

    // Apply version updates
    println!("\n{}", "Applying version updates...".green().bold());
    for (package_name, _, new_version) in &version_updates {
        if let Some(package) = packages.iter_mut().find(|p| &p.name == package_name) {
            crate::version::update_version(package, new_version)?;
            package.current_version = new_version.clone();
            println!(
                "{} Updated {} to {}",
                "✓".green().bold(),
                package_name,
                new_version
            );
        }
    }

    // Generate changelog
    if !no_changelog && config.changelog.enabled {
        println!("\n{}", "Generating changelog...".green().bold());
        crate::changelog::generate(&config, &changesets, &version_updates)?;
        println!("{} Updated {}", "✓".green().bold(), config.changelog.file);
    }

    // Remove applied changesets
    println!("\n{}", "Cleaning up changesets...".green().bold());
    for (path, _) in &changesets {
        // Only remove changesets that were actually applied
        let changeset_packages: Vec<String> = {
            let content = fs::read_to_string(path)?;
            if let Some(stripped) = content.strip_prefix("---\n") {
                if let Some(end_pos) = stripped.find("\n---\n") {
                    let frontmatter = &stripped[..end_pos];
                    let changeset_file: ChangesetFile = serde_yaml::from_str(frontmatter)?;
                    changeset_file.packages
                } else {
                    continue;
                }
            } else {
                continue;
            }
        };

        let should_remove = if let Some(filter) = &package_filter {
            changeset_packages.contains(filter)
        } else {
            true
        };

        if should_remove {
            fs::remove_file(path)?;
            println!(
                "{} Removed {}",
                "✓".green().bold(),
                path.file_name().unwrap().to_str().unwrap()
            );
        }
    }

    // Run post-apply commands
    if !config.post_apply.commands.is_empty() {
        println!("\n{}", "Running post-apply commands...".green().bold());
        for command in &config.post_apply.commands {
            println!("{} {}", "→".cyan(), command);
            // In a real implementation, we would execute these commands
            // For now, we'll just print them
        }
    }

    println!("\n{}", "Changesets applied successfully!".green().bold());

    Ok(())
}
