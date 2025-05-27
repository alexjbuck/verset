use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "verset")]
#[command(about = "Universal changeset management tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize verset in current repository
    Init,

    /// Add a new changeset
    Add {
        /// Target specific package(s), comma-separated
        #[arg(short, long)]
        package: Option<String>,

        /// Change type (major|minor|patch)
        #[arg(short = 't', long)]
        r#type: Option<String>,

        /// Change description
        #[arg(short, long)]
        message: Option<String>,

        /// Apply to all packages
        #[arg(long)]
        all: bool,
    },

    /// Show pending changesets and version impacts
    Status {
        /// Show status for specific package
        #[arg(short, long)]
        package: Option<String>,
    },

    /// Apply pending changesets, update versions, and generate changelog
    Apply {
        /// Apply only to specific package(s)
        #[arg(short, long)]
        package: Option<String>,

        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,

        /// Skip changelog generation
        #[arg(long)]
        no_changelog: bool,
    },

    /// Publish packages with new versions to registries
    Publish {
        /// Publish only specific package(s)
        #[arg(short, long)]
        package: Option<String>,

        /// Preview what would be published
        #[arg(long)]
        dry_run: bool,

        /// Override default registry
        #[arg(long)]
        registry: Option<String>,
    },

    /// List all detected packages with their current versions
    List,

    /// Validate configuration and package definitions
    Validate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("{}", "Initializing verset...".green().bold());
            verset::config::init()?;
        }
        Commands::Add {
            package,
            r#type,
            message,
            all,
        } => {
            if package.is_none() && r#type.is_none() && message.is_none() && !all {
                // Interactive mode
                println!(
                    "{}",
                    "Creating new changeset (interactive mode)..."
                        .green()
                        .bold()
                );
                verset::interactive::add_changeset()?;
            } else {
                // Non-interactive mode
                println!("{}", "Creating new changeset...".green().bold());
                verset::changeset::add_non_interactive(package, r#type, message, all)?;
            }
        }
        Commands::Status { package } => {
            println!("{}", "Checking changeset status...".green().bold());
            verset::changeset::status(package)?;
        }
        Commands::Apply {
            package,
            dry_run,
            no_changelog,
        } => {
            if dry_run {
                println!("{}", "Previewing changes (dry run)...".yellow().bold());
            } else {
                println!("{}", "Applying changesets...".green().bold());
            }
            verset::changeset::apply(package, dry_run, no_changelog)?;
        }
        Commands::Publish {
            package,
            dry_run,
            registry,
        } => {
            if dry_run {
                println!("{}", "Previewing publish (dry run)...".yellow().bold());
            } else {
                println!("{}", "Publishing packages...".green().bold());
            }
            verset::publish::publish(package, dry_run, registry)?;
        }
        Commands::List => {
            println!("{}", "Listing packages...".green().bold());
            verset::package::list()?;
        }
        Commands::Validate => {
            println!("{}", "Validating configuration...".green().bold());
            verset::config::validate()?;
        }
    }

    Ok(())
}
