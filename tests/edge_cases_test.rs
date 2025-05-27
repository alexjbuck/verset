use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_malformed_version_handling() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    // Create a Rust project with invalid version
    let cargo_toml = r#"[package]
name = "bad-version"
version = "not-a-version"
edition = "2021"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    // Initialize verset - this should succeed even with bad version
    // The init will show a warning but continue
    let init_result = verset::config::init();
    assert!(init_result.is_ok()); // init should succeed

    // But detecting packages should fail
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config);
    assert!(packages.is_err());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_missing_version_field() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    // Create a package.json without version field
    let package_json = r#"{
  "name": "no-version-package",
  "description": "Package without version",
  "main": "index.js"
}"#;
    fs::write(project_dir.join("package.json"), package_json).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    // Initialize verset
    let init_result = verset::config::init();
    assert!(init_result.is_ok());

    // Detecting packages should fail due to missing version
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config);
    assert!(packages.is_err());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_empty_changeset_directory() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    let cargo_toml = r#"[package]
name = "test-package"
version = "1.0.0"
edition = "2021"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();

    // Try to apply with no changesets
    let result = verset::changeset::apply(None, false, false);
    assert!(result.is_ok()); // Should succeed but do nothing

    // Status should show no changesets
    let changesets = verset::changeset::load_changesets().unwrap();
    assert_eq!(changesets.len(), 0);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_duplicate_package_names() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    // Create two packages with the same name in different directories
    let packages_dir = project_dir.join("packages");
    fs::create_dir(&packages_dir).unwrap();

    let pkg1_dir = packages_dir.join("package1");
    fs::create_dir(&pkg1_dir).unwrap();
    let package_json1 = r#"{
  "name": "duplicate-name",
  "version": "1.0.0"
}"#;
    fs::write(pkg1_dir.join("package.json"), package_json1).unwrap();

    let pkg2_dir = packages_dir.join("package2");
    fs::create_dir(&pkg2_dir).unwrap();
    let package_json2 = r#"{
  "name": "duplicate-name",
  "version": "2.0.0"
}"#;
    fs::write(pkg2_dir.join("package.json"), package_json2).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();

    // Both packages should be detected despite having the same name
    assert_eq!(packages.len(), 2);
    assert!(packages.iter().all(|p| p.name == "duplicate-name"));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_changeset_with_nonexistent_package() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    let cargo_toml = r#"[package]
name = "real-package"
version = "1.0.0"
edition = "2021"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();

    // Create changeset for non-existent package
    let changeset = verset::Changeset {
        packages: vec!["nonexistent-package".to_string()],
        change_type: verset::ChangeType::Minor,
        description: "Update nonexistent".to_string(),
        created_at: chrono::Utc::now(),
    };
    verset::changeset::save_changeset(&changeset).unwrap();

    // Apply should handle this gracefully
    let result = verset::changeset::apply(None, false, true);
    assert!(result.is_ok());

    // The changeset should still exist since it wasn't applied
    let changesets = verset::changeset::load_changesets().unwrap();
    assert_eq!(changesets.len(), 1);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_pre_release_version_handling() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    // Create a project with pre-release version
    let cargo_toml = r#"[package]
name = "prerelease-test"
version = "1.0.0-alpha.1"
edition = "2021"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();

    // Create and apply a minor changeset
    let changeset = verset::Changeset {
        packages: vec!["prerelease-test".to_string()],
        change_type: verset::ChangeType::Minor,
        description: "Minor update".to_string(),
        created_at: chrono::Utc::now(),
    };
    verset::changeset::save_changeset(&changeset).unwrap();
    verset::changeset::apply(None, false, true).unwrap();

    // Version should be 1.1.0 (pre-release removed)
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();
    assert_eq!(packages[0].current_version.to_string(), "1.1.0");

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_unicode_in_descriptions() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    let cargo_toml = r#"[package]
name = "unicode-test"
version = "1.0.0"
edition = "2021"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();

    // Create changeset with unicode characters
    let changeset = verset::Changeset {
        packages: vec!["unicode-test".to_string()],
        change_type: verset::ChangeType::Patch,
        description: "Fix bug 🐛 and add emoji support 🎉".to_string(),
        created_at: chrono::Utc::now(),
    };
    verset::changeset::save_changeset(&changeset).unwrap();

    // Load and verify
    let changesets = verset::changeset::load_changesets().unwrap();
    assert_eq!(changesets.len(), 1);
    assert_eq!(
        changesets[0].1.description,
        "Fix bug 🐛 and add emoji support 🎉"
    );

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
#[serial]
fn test_multiple_changesets_same_package() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    let cargo_toml = r#"[package]
name = "multi-change"
version = "1.0.0"
edition = "2021"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    verset::config::init().unwrap();

    // Create multiple changesets
    let changeset1 = verset::Changeset {
        packages: vec!["multi-change".to_string()],
        change_type: verset::ChangeType::Patch,
        description: "Fix bug 1".to_string(),
        created_at: chrono::Utc::now(),
    };
    verset::changeset::save_changeset(&changeset1).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps

    let changeset2 = verset::Changeset {
        packages: vec!["multi-change".to_string()],
        change_type: verset::ChangeType::Minor,
        description: "Add feature".to_string(),
        created_at: chrono::Utc::now(),
    };
    verset::changeset::save_changeset(&changeset2).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    let changeset3 = verset::Changeset {
        packages: vec!["multi-change".to_string()],
        change_type: verset::ChangeType::Patch,
        description: "Fix bug 2".to_string(),
        created_at: chrono::Utc::now(),
    };
    verset::changeset::save_changeset(&changeset3).unwrap();

    // Apply all changesets
    verset::changeset::apply(None, false, true).unwrap();

    // Should apply the highest change type (minor)
    let config = verset::config::load().unwrap();
    let packages = verset::detect::detect_packages(&config).unwrap();
    assert_eq!(packages[0].current_version.to_string(), "1.1.0");

    // All changesets should be removed
    let changesets = verset::changeset::load_changesets().unwrap();
    assert_eq!(changesets.len(), 0);

    std::env::set_current_dir(original_dir).unwrap();
}
