use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use semver::Version;

pub mod config;
pub mod package;
pub mod changeset;
pub mod version;
pub mod changelog;
pub mod publish;
pub mod detect;
pub mod interactive;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Major,
    Minor,
    Patch,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Major => write!(f, "major"),
            ChangeType::Minor => write!(f, "minor"),
            ChangeType::Patch => write!(f, "patch"),
        }
    }
}

impl std::str::FromStr for ChangeType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "major" => Ok(ChangeType::Major),
            "minor" => Ok(ChangeType::Minor),
            "patch" => Ok(ChangeType::Patch),
            _ => Err(anyhow::anyhow!("Invalid change type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    pub packages: Vec<String>,
    pub change_type: ChangeType,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub path: PathBuf,
    pub package_type: PackageType,
    pub version_file: PathBuf,
    pub version_path: String,
    pub current_version: Version,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageType {
    Rust,
    Node,
    Python,
    Go,
    Java,
    Dotnet,
    Custom { command: String },
}

impl std::fmt::Display for PackageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageType::Rust => write!(f, "Rust"),
            PackageType::Node => write!(f, "Node.js"),
            PackageType::Python => write!(f, "Python"),
            PackageType::Go => write!(f, "Go"),
            PackageType::Java => write!(f, "Java"),
            PackageType::Dotnet => write!(f, ".NET"),
            PackageType::Custom { command } => write!(f, "Custom ({})", command),
        }
    }
} 