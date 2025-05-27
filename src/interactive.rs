use crate::changeset::save_changeset;
use crate::detect::detect_packages;
use crate::{ChangeType, Changeset};
use anyhow::Result;
use chrono::Utc;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};

pub fn add_changeset() -> Result<()> {
    let config = crate::config::load()?;
    let packages = detect_packages(&config)?;

    if packages.is_empty() {
        return Err(anyhow::anyhow!("No packages found"));
    }

    // Select packages
    let package_names: Vec<String> = packages.iter().map(|p| p.name.clone()).collect();
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select packages to include in this changeset")
        .items(&package_names)
        .interact()?;

    if selections.is_empty() {
        println!(
            "{}",
            "No packages selected. Changeset creation cancelled.".yellow()
        );
        return Ok(());
    }

    let selected_packages: Vec<String> = selections
        .iter()
        .map(|&i| package_names[i].clone())
        .collect();

    // Select change type
    let change_types = vec!["patch", "minor", "major"];
    let change_type_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the type of change")
        .items(&change_types)
        .default(0)
        .interact()?;

    let change_type = change_types[change_type_index].parse::<ChangeType>()?;

    // Get description
    let description: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a description for this changeset")
        .interact_text()?;

    if description.trim().is_empty() {
        return Err(anyhow::anyhow!("Description cannot be empty"));
    }

    // Create and save changeset
    let changeset = Changeset {
        packages: selected_packages,
        change_type,
        description: description.trim().to_string(),
        created_at: Utc::now(),
    };

    save_changeset(&changeset)?;

    Ok(())
}
