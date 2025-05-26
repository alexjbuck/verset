# Verset: Universal Changeset Management Tool

## Project Overview

Verset is a universal changeset management tool written in Rust that brings the developer experience of JavaScript's changesets to all programming languages and ecosystems. It supports both single repositories and monorepos, with first-class support for Rust, Python, and Node.js, and extensible support for any language that stores version information in parseable files.

## Core Objectives

- **Universal Language Support**: Work with any programming language that stores version numbers in JSON, TOML, YAML, Python, Rust, JavaScript, or TypeScript files
- **Monorepo & Single-Repo Friendly**: Handle complex monorepos with cross-package dependencies while also working perfectly for single-package repositories
- **Developer-Friendly CLI**: Interactive interface for easy changeset creation with non-interactive mode for automation
- **Standards Compliant**: Follow semantic versioning (major.minor.patch-pre+meta) across all languages
- **Automation Ready**: Configurable post-apply hooks and publishing commands
- **Just Works**: Auto-detect common project structures and provide sensible defaults

## Architecture & Technical Stack

### Core Technology
- **Language**: Rust for performance, reliability, and single binary distribution
- **CLI Framework**: Clap for command-line interface
- **Configuration**: TOML for configuration files
- **Serialization**: Serde for data handling
- **Version Management**: SemVer crate for semantic versioning

### Key Dependencies
```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
semver = { version = "1.0", features = ["serde"] }
glob = "0.3"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
dialoguer = "0.11"  # Interactive prompts
indicatif = "0.17"  # Progress bars
colored = "2.0"     # Colored output
```

## Command-Line Interface

### Core Commands

#### `verset init`
Initialize verset in current repository. Auto-detects packages and creates `.verset/config.toml`.

#### `verset add [OPTIONS]`
Add a new changeset.

**Interactive Mode (default):**
```bash
verset add
# Prompts for package selection, change type, and description
```

**Non-Interactive Mode:**
```bash
verset add --package core --type minor --message "Add OAuth support"
verset add --package "web,api" --type patch --message "Fix CORS headers"
verset add --all --type major --message "Breaking API changes"
```

**Options:**
- `--package <PACKAGE>`: Target specific package(s), comma-separated
- `--type <TYPE>`: Change type (major|minor|patch)
- `--message <MESSAGE>`: Change description
- `--all`: Apply to all packages

#### `verset status [OPTIONS]`
Show pending changesets and version impacts.

```bash
verset status
verset status --package core
```

#### `verset apply [OPTIONS]`
Apply pending changesets, update versions, and generate changelog.

```bash
verset apply
verset apply --package core
verset apply --dry-run
verset apply --no-changelog
```

**Options:**
- `--package <PACKAGE>`: Apply only to specific package(s)
- `--dry-run`: Preview changes without applying
- `--no-changelog`: Skip changelog generation

#### `verset publish [OPTIONS]`
Publish packages with new versions to registries.

```bash
verset publish
verset publish --package core
verset publish --dry-run
verset publish --registry custom-registry.com
```

**Options:**
- `--package <PACKAGE>`: Publish only specific package(s)  
- `--dry-run`: Preview what would be published
- `--registry <URL>`: Override default registry

#### `verset list`
List all detected packages with their current versions and metadata.

#### `verset validate`
Validate configuration and package definitions.

## Configuration System

### Main Configuration (`.verset/config.toml`)

```toml
[packages]
# Glob patterns for automatic package discovery
patterns = ["packages/*", "apps/*", "libs/*", "."]

# Manual package definitions (override auto-detection)
[[packages.manual]]
name = "core"
path = "packages/core"
type = "rust"
version_file = "Cargo.toml"
version_path = "package.version"

[[packages.manual]]
name = "web-client"
path = "packages/web"
type = "node"
version_file = "package.json"
version_path = "version"

[changelog]
enabled = true
file = "CHANGELOG.md"
format = "keep-a-changelog"  # or "custom"
template_path = ".verset/changelog.template"

[post_apply]
# Shell commands to run after applying changesets
commands = [
  "cargo check",
  "npm run build",
  "git add ."
]

[publish]
# Default publish settings
dry_run = false

# Per-language defaults
[publish.rust]
registry = "https://crates.io"
command = "cargo publish"

[publish.node]
registry = "https://registry.npmjs.org"
command = "npm publish"

[publish.python]
registry = "https://pypi.org"
command = "poetry publish"

# Custom per-package publish settings
[[publish.packages]]
name = "core"
commands = ["cargo publish --registry custom"]

[[publish.packages]]
name = "web-client"
commands = [
  "npm run build",
  "npm publish --registry https://npm.company.com"
]
```

## Changeset Format

Changesets are stored as Markdown files in `.verset/changesets/` with frontmatter metadata.

**Filename Pattern**: `{timestamp}-{random}.md`  
**Example**: `.verset/changesets/20241201-abc123.md`

```markdown
---
packages: ["core", "web-client"]
type: "minor"
---

Add OAuth2 authentication support

- Add OAuth2 providers (Google, GitHub)
- Update authentication middleware
- Add configuration options for OAuth2
```

## Package Auto-Detection

### Language Detection Logic
Verset automatically detects package types based on files present:

- **Rust**: `Cargo.toml` present
- **Node.js**: `package.json` present
- **Python**: `pyproject.toml` present
- **Go**: `go.mod` present
- **Java/Maven**: `pom.xml` present
- **Java/Gradle**: `build.gradle` or `build.gradle.kts` present
- **C#/.NET**: `*.csproj` present

### Supported Version File Formats

#### Rust (`Cargo.toml`)
```toml
[package]
version = "1.2.3"
```

#### Node.js (`package.json`)
```json
{
  "version": "1.2.3"
}
```

#### Python (`pyproject.toml`)
```toml
[project]
version = "1.2.3"
```

#### Python (`__init__.py`)
```python
__version__ = "1.2.3"
```

#### Custom paths supported via configuration for any JSON/TOML/YAML file.

## Core Data Structures

### Rust Type Definitions

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Major,
    Minor,
    Patch,
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
```

## Module Structure

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Core exports and types
├── config.rs         # Configuration management
├── package.rs        # Package detection and management
├── changeset.rs      # Changeset creation and management
├── version.rs        # Version file parsing and updating
├── changelog.rs      # Changelog generation
├── publish.rs        # Package publishing
├── detect.rs         # Language and package detection
└── interactive.rs    # Interactive CLI prompts
```

## Example Usage Workflows

### Single Package Repository
```bash
cd my-rust-project
verset init
verset add --type minor --message "Add new API endpoint"
verset apply
verset publish
```

### Monorepo Workflow
```bash
cd my-monorepo
verset init

# Interactive changeset creation
verset add
# Prompts for package selection and change details

# Non-interactive for automation
verset add --package "core,shared" --type minor --message "Add OAuth support"

# Preview changes
verset status
verset apply --dry-run

# Apply changes
verset apply
# Updates versions, generates changelog, runs post-apply commands

# Publish specific packages
verset publish --package core
```

### CI/CD Integration
```bash
# In CI pipeline after merge to main
verset apply --dry-run  # Verify changes
verset apply           # Apply if changes exist
verset publish         # Publish new versions
```

## Development Phases

### Phase 1: Core MVP
- [ ] Basic CLI structure with clap
- [ ] Configuration system (TOML parsing)
- [ ] Package detection for Rust, Node.js, Python
- [ ] Changeset creation and storage
- [ ] Version file parsing and updating
- [ ] Basic apply command functionality

### Phase 2: User Experience
- [ ] Interactive prompts with dialoguer
- [ ] Colored output and progress bars
- [ ] Comprehensive error handling
- [ ] Validation system
- [ ] Status command with detailed output

### Phase 3: Advanced Features
- [ ] Changelog generation
- [ ] Post-apply command execution
- [ ] Dry-run functionality
- [ ] Package filtering and selection

### Phase 4: Publishing
- [ ] Registry publishing for major languages
- [ ] Custom publish commands
- [ ] Publishing dry-run
- [ ] Registry configuration

### Phase 5: Distribution
- [ ] Package verset into node package for distribution through npm
- [ ] Package verset into python package for distribution through pypi
- [ ] Publish verset to crates.io for distribution through cargo
- [ ] Publish verset documentation on github pages using mdbook 

### Phase 6: Extensions
- [ ] Additional language support (Go, Java, C#)
- [ ] Custom file format support
- [ ] Advanced changelog templating
- [ ] CI/CD integrations and examples


## Success Metrics

- **Ease of Use**: Single command to get started (`verset init`)
- **Language Coverage**: Support for top 5 programming languages
- **Performance**: Sub-second response times for common operations
- **Reliability**: Comprehensive error handling and validation
- **Adoption**: Clear documentation and examples for common workflows

## Competitive Advantages

1. **Universal**: Works across all programming languages and ecosystems
2. **Rust Performance**: Fast, reliable, single binary distribution
3. **Familiar UX**: JavaScript changesets users feel immediately at home
4. **Monorepo Native**: Handles complex cross-package dependencies
5. **Configurable**: Adapts to any workflow or project structure
6. **Standards Compliant**: Proper semantic versioning everywhere

## Getting Started for Developers

This project is designed to be implemented in Rust with a focus on developer experience and universal compatibility. The goal is to create the definitive tool for changeset management across all programming ecosystems.

Key implementation priorities:
1. **Robust package detection** that works reliably across languages
2. **Intuitive CLI** that feels natural to developers from any background
3. **Comprehensive error handling** with helpful error messages
4. **Extensive testing** to ensure reliability across different project structures
5. **Clear documentation** with examples for common use cases

The tool should feel like a natural extension of existing development workflows while providing the power and flexibility needed for complex monorepo scenarios.
