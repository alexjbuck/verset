# Verset

Universal changeset management tool for all programming languages and ecosystems.

[![Crates.io](https://img.shields.io/crates/v/verset.svg)](https://crates.io/crates/verset)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Verset brings the developer experience of JavaScript's changesets to all programming languages. It supports both single repositories and monorepos, with first-class support for Rust, Python, Node.js, Go, Java, and .NET projects.

## Features

- 🌍 **Universal Language Support** - Works with any language that stores version numbers in parseable files
- 📦 **Monorepo & Single-Repo** - Handle complex monorepos with cross-package dependencies
- 🎯 **Interactive CLI** - User-friendly interface for creating changesets
- 📝 **Automatic Changelog** - Generate changelogs in Keep a Changelog format
- 🚀 **Publishing Support** - Publish packages to their respective registries
- ⚡ **Fast & Reliable** - Written in Rust for performance and reliability

## Installation

### From Crates.io

```bash
cargo install verset
```

### From Source

```bash
git clone https://github.com/yourusername/verset
cd verset
cargo install --path .
```

## Quick Start

### Initialize Verset in your project

```bash
verset init
```

This will:
- Create `.verset/` directory
- Generate `config.toml` with sensible defaults
- Auto-detect packages in your repository

### Create a changeset

```bash
# Interactive mode (recommended)
verset add

# Non-interactive mode
verset add --package my-package --type minor --message "Add new feature"
```

### Check status

```bash
verset status
```

### Apply changesets

```bash
# Preview changes
verset apply --dry-run

# Apply changes
verset apply
```

### Publish packages

```bash
verset publish
```

## Usage Examples

### Single Package Repository

```bash
cd my-rust-project
verset init
verset add --type minor --message "Add OAuth support"
verset apply
verset publish
```

### Monorepo Workflow

```bash
cd my-monorepo
verset init

# Add changeset for multiple packages
verset add --package "core,api" --type patch --message "Fix memory leak"

# Add changeset for all packages
verset add --all --type major --message "Breaking: Update API"

# Apply changes to specific package
verset apply --package core

# Publish specific packages
verset publish --package "core,api"
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Apply changesets
  run: |
    verset apply
    
- name: Publish packages
  if: github.ref == 'refs/heads/main'
  run: |
    verset publish
  env:
    CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}
    NPM_TOKEN: ${{ secrets.NPM_TOKEN }}
```

## Configuration

Verset uses a TOML configuration file located at `.verset/config.toml`.

### Basic Configuration

```toml
[packages]
# Glob patterns for package discovery
patterns = ["packages/*", "apps/*", "."]

[changelog]
enabled = true
file = "CHANGELOG.md"
format = "keep-a-changelog"

[post_apply]
# Commands to run after applying changesets
commands = [
  "cargo check",
  "npm run build"
]
```

### Manual Package Configuration

```toml
[[packages.manual]]
name = "my-package"
path = "packages/core"
type = "rust"
version_file = "Cargo.toml"
version_path = "package.version"
```

### Publishing Configuration

```toml
[publish.rust]
registry = "https://crates.io"
command = "cargo publish"

[publish.node]
registry = "https://registry.npmjs.org"
command = "npm publish"

[[publish.packages]]
name = "my-private-package"
commands = ["npm publish --registry https://npm.company.com"]
```

## Supported Languages

### Rust
- Version file: `Cargo.toml`
- Version path: `package.version`
- Publish: `cargo publish`

### Node.js
- Version file: `package.json`
- Version path: `version`
- Publish: `npm publish`

### Python
- Version files: `pyproject.toml`, `__init__.py`
- Version paths: `project.version`, `tool.poetry.version`, `__version__`
- Publish: `poetry publish`

### Go
- Version file: `go.mod`, `version.go`
- Version detection: Looks for `const Version = "x.y.z"`
- Publish: `go mod publish`

### Java
- Maven: `pom.xml`
- Gradle: `build.gradle`, `build.gradle.kts`
- Publish: `mvn deploy`

### .NET
- Version file: `*.csproj`
- Version path: `Version`
- Publish: `dotnet nuget push`

## Changeset Format

Changesets are stored as Markdown files in `.verset/changesets/`:

```markdown
---
packages:
- my-package
- another-package
type: minor
---

Add OAuth2 authentication support

- Add Google and GitHub providers
- Update authentication middleware
- Add configuration options
```

## Commands Reference

### `verset init`
Initialize verset in the current repository.

### `verset add [OPTIONS]`
Create a new changeset.

Options:
- `--package <PACKAGES>` - Target specific package(s), comma-separated
- `--type <TYPE>` - Change type (major|minor|patch)
- `--message <MESSAGE>` - Change description
- `--all` - Apply to all packages

### `verset status [OPTIONS]`
Show pending changesets and version impacts.

Options:
- `--package <PACKAGE>` - Show status for specific package

### `verset apply [OPTIONS]`
Apply pending changesets and update versions.

Options:
- `--package <PACKAGE>` - Apply only to specific package(s)
- `--dry-run` - Preview changes without applying
- `--no-changelog` - Skip changelog generation

### `verset publish [OPTIONS]`
Publish packages to their registries.

Options:
- `--package <PACKAGES>` - Publish specific package(s)
- `--dry-run` - Preview what would be published
- `--registry <URL>` - Override default registry

### `verset list`
List all detected packages with their versions.

### `verset validate`
Validate configuration and package definitions.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 