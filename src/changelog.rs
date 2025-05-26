use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use chrono::Utc;
use semver::Version;
use crate::Changeset;
use crate::config::Config;

pub fn generate(
    config: &Config,
    changesets: &[(PathBuf, Changeset)],
    version_updates: &[(String, Version, Version)],
) -> Result<()> {
    let changelog_path = Path::new(&config.changelog.file);
    
    // Read existing changelog or create new
    let existing_content = if changelog_path.exists() {
        fs::read_to_string(changelog_path)?
    } else {
        String::from("# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n")
    };
    
    // Generate new entry
    let date = Utc::now().format("%Y-%m-%d");
    let mut new_entry = format!("## [{}] - {}\n\n", date, date);
    
    // Group changesets by type
    let mut major_changes = Vec::new();
    let mut minor_changes = Vec::new();
    let mut patch_changes = Vec::new();
    
    for (_, changeset) in changesets {
        match changeset.change_type {
            crate::ChangeType::Major => major_changes.push(changeset),
            crate::ChangeType::Minor => minor_changes.push(changeset),
            crate::ChangeType::Patch => patch_changes.push(changeset),
        }
    }
    
    // Add version updates section
    if !version_updates.is_empty() {
        new_entry.push_str("### Version Updates\n\n");
        for (package, old_version, new_version) in version_updates {
            new_entry.push_str(&format!("- **{}**: {} → {}\n", package, old_version, new_version));
        }
        new_entry.push('\n');
    }
    
    // Add changes by type
    if !major_changes.is_empty() {
        new_entry.push_str("### Breaking Changes\n\n");
        for changeset in &major_changes {
            new_entry.push_str(&format!("- {}\n", changeset.description));
            if changeset.packages.len() > 1 {
                new_entry.push_str(&format!("  - Affects: {}\n", changeset.packages.join(", ")));
            }
        }
        new_entry.push('\n');
    }
    
    if !minor_changes.is_empty() {
        new_entry.push_str("### Added\n\n");
        for changeset in &minor_changes {
            new_entry.push_str(&format!("- {}\n", changeset.description));
            if changeset.packages.len() > 1 {
                new_entry.push_str(&format!("  - Affects: {}\n", changeset.packages.join(", ")));
            }
        }
        new_entry.push('\n');
    }
    
    if !patch_changes.is_empty() {
        new_entry.push_str("### Fixed\n\n");
        for changeset in &patch_changes {
            new_entry.push_str(&format!("- {}\n", changeset.description));
            if changeset.packages.len() > 1 {
                new_entry.push_str(&format!("  - Affects: {}\n", changeset.packages.join(", ")));
            }
        }
        new_entry.push('\n');
    }
    
    // Insert new entry after the header
    let lines: Vec<&str> = existing_content.lines().collect();
    let insert_position = lines.iter()
        .position(|line| line.starts_with("## "))
        .unwrap_or(lines.len());
    
    // Reconstruct changelog
    let mut new_content = String::new();
    
    // Add header lines
    for line in lines.iter().take(insert_position) {
        new_content.push_str(line);
        new_content.push('\n');
    }
    
    // Add new entry
    new_content.push_str(&new_entry);
    
    // Add remaining content
    for line in lines.iter().skip(insert_position) {
        new_content.push_str(line);
        new_content.push('\n');
    }
    
    // Write updated changelog
    fs::write(changelog_path, new_content)?;
    
    Ok(())
} 