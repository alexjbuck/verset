use semver::Version;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

// Helper function to create a test directory
fn setup_test_dir() -> TempDir {
    TempDir::new().unwrap()
}

#[test]
#[serial]
fn test_rust_package_detection_and_update() {
    let temp_dir = setup_test_dir();
    let project_dir = temp_dir.path();

    // Create a Rust project
    let cargo_toml = r#"[package]
name = "test-rust-package"
version = "1.2.3"
edition = "2021"

[dependencies]
serde = "1.0"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // Save and change directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    // Initialize verset
    verset::config::init().unwrap();

    // Load config and detect packages
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();

    // Verify package detection
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "test-rust-package");
    assert_eq!(
        packages[0].current_version,
        Version::parse("1.2.3").unwrap()
    );
    assert!(matches!(
        packages[0].package_type,
        verset::PackageType::Rust
    ));

    // Test version update
    let new_version = Version::parse("2.0.0").unwrap();
    verset::version::update_version(&packages[0], &new_version).unwrap();

    // Verify the update
    let updated_content = fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(updated_content.contains("version = \"2.0.0\""));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_nodejs_package_detection_and_update() {
    let temp_dir = setup_test_dir();
    let project_dir = temp_dir.path();

    // Create a Node.js project
    let package_json = r#"{
  "name": "test-node-package",
  "version": "0.1.0",
  "description": "Test package",
  "main": "index.js",
  "dependencies": {
    "express": "^4.18.0"
  }
}"#;
    fs::write(project_dir.join("package.json"), package_json).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();

    // Verify package detection
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "test-node-package");
    assert_eq!(
        packages[0].current_version,
        Version::parse("0.1.0").unwrap()
    );
    assert!(matches!(
        packages[0].package_type,
        verset::PackageType::Node
    ));

    // Test version update
    let new_version = Version::parse("1.0.0").unwrap();
    verset::version::update_version(&packages[0], &new_version).unwrap();

    // Verify the update
    let updated_content = fs::read_to_string(project_dir.join("package.json")).unwrap();
    assert!(updated_content.contains("\"version\": \"1.0.0\""));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_python_pyproject_detection_and_update() {
    let temp_dir = setup_test_dir();
    let project_dir = temp_dir.path();

    // Create a Python project with pyproject.toml (PEP 621)
    let pyproject_toml = r#"[project]
name = "test-python-package"
version = "3.2.1"
description = "Test Python package"
requires-python = ">=3.8"

[project.dependencies]
requests = ">=2.28.0"
"#;
    fs::write(project_dir.join("pyproject.toml"), pyproject_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();

    // Verify package detection
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "test-python-package");
    assert_eq!(
        packages[0].current_version,
        Version::parse("3.2.1").unwrap()
    );
    assert!(matches!(
        packages[0].package_type,
        verset::PackageType::Python
    ));
    assert_eq!(packages[0].version_path, "project.version");

    // Test version update
    let new_version = Version::parse("4.0.0").unwrap();
    verset::version::update_version(&packages[0], &new_version).unwrap();

    // Verify the update
    let updated_content = fs::read_to_string(project_dir.join("pyproject.toml")).unwrap();
    assert!(updated_content.contains("version = \"4.0.0\""));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_python_poetry_detection_and_update() {
    let temp_dir = setup_test_dir();
    let project_dir = temp_dir.path();

    // Create a Poetry-based Python project
    let pyproject_toml = r#"[tool.poetry]
name = "test-poetry-package"
version = "2.1.0"
description = "Test Poetry package"
authors = ["Test Author <test@example.com>"]

[tool.poetry.dependencies]
python = "^3.8"
requests = "^2.28.0"
"#;
    fs::write(project_dir.join("pyproject.toml"), pyproject_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();

    // Verify package detection
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "test-poetry-package");
    assert_eq!(
        packages[0].current_version,
        Version::parse("2.1.0").unwrap()
    );
    assert!(matches!(
        packages[0].package_type,
        verset::PackageType::Python
    ));
    assert_eq!(packages[0].version_path, "tool.poetry.version");

    // Test version update
    let new_version = Version::parse("3.0.0").unwrap();
    verset::version::update_version(&packages[0], &new_version).unwrap();

    // Verify the update
    let updated_content = fs::read_to_string(project_dir.join("pyproject.toml")).unwrap();
    assert!(updated_content.contains("version = \"3.0.0\""));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_python_init_py_detection_and_update() {
    let temp_dir = setup_test_dir();
    let project_dir = temp_dir.path();

    // Create a Python package with __init__.py
    let package_dir = project_dir.join("my_package");
    fs::create_dir(&package_dir).unwrap();

    let init_py = r#""""My test package."""

__version__ = "1.5.2"
__author__ = "Test Author"

def hello():
    return "Hello, World!"
"#;
    fs::write(package_dir.join("__init__.py"), init_py).unwrap();

    // Also create a basic pyproject.toml without version
    let pyproject_toml = r#"[project]
name = "my_package"
description = "Test package"
"#;
    fs::write(package_dir.join("pyproject.toml"), pyproject_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&package_dir).unwrap();

    verset::config::init().unwrap();
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();

    // Verify package detection
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "my_package");
    assert_eq!(
        packages[0].current_version,
        Version::parse("1.5.2").unwrap()
    );
    assert!(matches!(
        packages[0].package_type,
        verset::PackageType::Python
    ));
    assert_eq!(packages[0].version_path, "__version__");

    // Test version update
    let new_version = Version::parse("2.0.0").unwrap();
    verset::version::update_version(&packages[0], &new_version).unwrap();

    // Verify the update
    let updated_content = fs::read_to_string(package_dir.join("__init__.py")).unwrap();
    assert!(updated_content.contains("__version__ = \"2.0.0\""));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_go_package_detection() {
    let temp_dir = setup_test_dir();
    let project_dir = temp_dir.path();

    // Create a Go project
    let go_mod = r#"module github.com/test/myapp

go 1.21

require (
    github.com/gin-gonic/gin v1.9.1
)
"#;
    fs::write(project_dir.join("go.mod"), go_mod).unwrap();

    // Create version.go file
    let version_go = r#"package main

const Version = "1.3.0"
const AppName = "MyApp"
"#;
    fs::write(project_dir.join("version.go"), version_go).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();

    // Verify package detection
    assert_eq!(packages.len(), 1);
    // The package name will be the temp directory name or "go-package" fallback
    // We should check for the version instead of the exact name
    assert_eq!(
        packages[0].current_version,
        Version::parse("1.3.0").unwrap()
    );
    assert!(matches!(packages[0].package_type, verset::PackageType::Go));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_monorepo_multi_language_detection() {
    let temp_dir = setup_test_dir();
    let project_dir = temp_dir.path();

    // Create a monorepo structure
    let packages_dir = project_dir.join("packages");
    fs::create_dir(&packages_dir).unwrap();

    // Create Rust package
    let rust_dir = packages_dir.join("rust-lib");
    fs::create_dir(&rust_dir).unwrap();
    let cargo_toml = r#"[package]
name = "my-rust-lib"
version = "1.0.0"
edition = "2021"
"#;
    fs::write(rust_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // Create Node.js package
    let node_dir = packages_dir.join("node-app");
    fs::create_dir(&node_dir).unwrap();
    let package_json = r#"{
  "name": "my-node-app",
  "version": "2.0.0",
  "main": "index.js"
}"#;
    fs::write(node_dir.join("package.json"), package_json).unwrap();

    // Create Python package
    let python_dir = packages_dir.join("python-lib");
    fs::create_dir(&python_dir).unwrap();
    let pyproject_toml = r#"[project]
name = "my-python-lib"
version = "3.0.0"
"#;
    fs::write(python_dir.join("pyproject.toml"), pyproject_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();

    // Verify all packages are detected
    assert_eq!(packages.len(), 3);

    // Find each package and verify
    let rust_pkg = packages.iter().find(|p| p.name == "my-rust-lib").unwrap();
    assert_eq!(rust_pkg.current_version, Version::parse("1.0.0").unwrap());
    assert!(matches!(rust_pkg.package_type, verset::PackageType::Rust));

    let node_pkg = packages.iter().find(|p| p.name == "my-node-app").unwrap();
    assert_eq!(node_pkg.current_version, Version::parse("2.0.0").unwrap());
    assert!(matches!(node_pkg.package_type, verset::PackageType::Node));

    let python_pkg = packages.iter().find(|p| p.name == "my-python-lib").unwrap();
    assert_eq!(python_pkg.current_version, Version::parse("3.0.0").unwrap());
    assert!(matches!(
        python_pkg.package_type,
        verset::PackageType::Python
    ));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_version_increment_logic() {
    let temp_dir = setup_test_dir();
    let project_dir = temp_dir.path();

    // Create a simple Rust project
    let cargo_toml = r#"[package]
name = "version-test"
version = "1.2.3"
edition = "2021"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();

    // Test patch increment
    let changeset = verset::Changeset {
        packages: vec!["version-test".to_string()],
        change_type: verset::ChangeType::Patch,
        description: "Fix bug".to_string(),
        created_at: chrono::Utc::now(),
    };
    verset::changeset::save_changeset(&changeset).unwrap();

    // Apply and check
    verset::changeset::apply(None, false, true).unwrap();
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();
    assert_eq!(
        packages[0].current_version,
        Version::parse("1.2.4").unwrap()
    );

    // Test minor increment
    let changeset = verset::Changeset {
        packages: vec!["version-test".to_string()],
        change_type: verset::ChangeType::Minor,
        description: "Add feature".to_string(),
        created_at: chrono::Utc::now(),
    };
    verset::changeset::save_changeset(&changeset).unwrap();

    verset::changeset::apply(None, false, true).unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();
    assert_eq!(
        packages[0].current_version,
        Version::parse("1.3.0").unwrap()
    );

    // Test major increment
    let changeset = verset::Changeset {
        packages: vec!["version-test".to_string()],
        change_type: verset::ChangeType::Major,
        description: "Breaking change".to_string(),
        created_at: chrono::Utc::now(),
    };
    verset::changeset::save_changeset(&changeset).unwrap();

    verset::changeset::apply(None, false, true).unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();
    assert_eq!(
        packages[0].current_version,
        Version::parse("2.0.0").unwrap()
    );

    std::env::set_current_dir(original_dir).unwrap();
}
