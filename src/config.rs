use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use colored::Colorize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub packages: PackagesConfig,
    #[serde(default)]
    pub changelog: ChangelogConfig,
    #[serde(default)]
    pub post_apply: PostApplyConfig,
    #[serde(default)]
    pub publish: PublishConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackagesConfig {
    #[serde(default = "default_patterns")]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub manual: Vec<ManualPackage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualPackage {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub package_type: String,
    pub version_file: String,
    pub version_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangelogConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_changelog_file")]
    pub file: String,
    #[serde(default = "default_changelog_format")]
    pub format: String,
    pub template_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PostApplyConfig {
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PublishConfig {
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub rust: LanguagePublishConfig,
    #[serde(default)]
    pub node: LanguagePublishConfig,
    #[serde(default)]
    pub python: LanguagePublishConfig,
    #[serde(default)]
    pub packages: Vec<PackagePublishConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguagePublishConfig {
    pub registry: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackagePublishConfig {
    pub name: String,
    pub commands: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            packages: PackagesConfig {
                patterns: default_patterns(),
                manual: vec![],
            },
            changelog: ChangelogConfig::default(),
            post_apply: PostApplyConfig::default(),
            publish: PublishConfig::default(),
        }
    }
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            file: "CHANGELOG.md".to_string(),
            format: "keep-a-changelog".to_string(),
            template_path: None,
        }
    }
}

impl Default for LanguagePublishConfig {
    fn default() -> Self {
        Self {
            registry: None,
            command: None,
        }
    }
}

fn default_patterns() -> Vec<String> {
    vec!["packages/*".to_string(), "apps/*".to_string(), "libs/*".to_string(), ".".to_string()]
}

fn default_true() -> bool {
    true
}

fn default_changelog_file() -> String {
    "CHANGELOG.md".to_string()
}

fn default_changelog_format() -> String {
    "keep-a-changelog".to_string()
}

pub fn init() -> Result<()> {
    let verset_dir = Path::new(".verset");
    
    if verset_dir.exists() {
        println!("{}", "Verset is already initialized in this repository".yellow());
        return Ok(());
    }
    
    // Create .verset directory
    fs::create_dir(verset_dir).context("Failed to create .verset directory")?;
    
    // Create changesets directory
    fs::create_dir(verset_dir.join("changesets"))
        .context("Failed to create changesets directory")?;
    
    // Create default config
    let config = Config::default();
    let config_path = verset_dir.join("config.toml");
    let config_toml = toml::to_string_pretty(&config)
        .context("Failed to serialize default config")?;
    
    fs::write(&config_path, config_toml)
        .context("Failed to write config file")?;
    
    println!("{} Created .verset/config.toml", "✓".green().bold());
    println!("{} Created .verset/changesets/", "✓".green().bold());
    
    // Auto-detect packages
    let packages = crate::detect::detect_packages(&config)?;
    
    if !packages.is_empty() {
        println!("\n{}", "Detected packages:".cyan().bold());
        for package in &packages {
            println!("  {} {} ({})", 
                "•".cyan(),
                package.name.bold(),
                package.package_type
            );
        }
    }
    
    println!("\n{}", "Verset initialized successfully!".green().bold());
    println!("Next steps:");
    println!("  • Review .verset/config.toml");
    println!("  • Run {} to create your first changeset", "verset add".cyan());
    
    Ok(())
}

pub fn load() -> Result<Config> {
    let config_path = Path::new(".verset/config.toml");
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Verset is not initialized. Run {} first.",
            "verset init".cyan()
        ));
    }
    
    let config_str = fs::read_to_string(config_path)
        .context("Failed to read config file")?;
    
    let config: Config = toml::from_str(&config_str)
        .context("Failed to parse config file")?;
    
    Ok(config)
}

pub fn validate() -> Result<()> {
    let config = load()?;
    
    println!("{} Configuration loaded successfully", "✓".green().bold());
    
    // Validate patterns
    if config.packages.patterns.is_empty() && config.packages.manual.is_empty() {
        return Err(anyhow::anyhow!("No package patterns or manual packages defined"));
    }
    
    // Detect packages
    let packages = crate::detect::detect_packages(&config)?;
    
    if packages.is_empty() {
        println!("{} No packages detected", "⚠".yellow().bold());
    } else {
        println!("{} {} packages detected", "✓".green().bold(), packages.len());
        for package in &packages {
            println!("  {} {} v{} ({})",
                "•".cyan(),
                package.name,
                package.current_version,
                package.package_type
            );
        }
    }
    
    // Check changesets directory
    let changesets_dir = Path::new(".verset/changesets");
    if !changesets_dir.exists() {
        return Err(anyhow::anyhow!("Changesets directory not found"));
    }
    
    // Count pending changesets
    let changeset_count = fs::read_dir(changesets_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .count();
    
    println!("{} {} pending changesets", "✓".green().bold(), changeset_count);
    
    println!("\n{}", "Configuration is valid!".green().bold());
    
    Ok(())
} 