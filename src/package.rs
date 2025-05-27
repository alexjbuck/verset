use crate::detect::detect_packages;
use anyhow::Result;
use colored::Colorize;

pub fn list() -> Result<()> {
    let config = crate::config::load()?;
    let packages = detect_packages(&config)?;

    if packages.is_empty() {
        println!("{}", "No packages found".yellow());
        return Ok(());
    }

    println!("{} {} packages\n", "Found".green().bold(), packages.len());

    // Find the longest package name for alignment
    let max_name_len = packages.iter().map(|p| p.name.len()).max().unwrap_or(0);

    println!(
        "{:<width$} {} {} {}",
        "Package".cyan().bold(),
        "Version".cyan().bold(),
        "Type".cyan().bold(),
        "Path".cyan().bold(),
        width = max_name_len
    );

    println!("{}", "-".repeat(80).dimmed());

    for package in &packages {
        println!(
            "{:<width$} {} {} {}",
            package.name.bold(),
            package.current_version.to_string().green(),
            format!("{}", package.package_type).yellow(),
            package.path.display().to_string().dimmed(),
            width = max_name_len
        );
    }

    Ok(())
}
