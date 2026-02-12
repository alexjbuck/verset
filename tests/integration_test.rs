use serial_test::serial;
use std::fs;
use tempfile::TempDir;

fn setup_test_project() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path();

    // Create a Cargo.toml for a test Rust project
    let cargo_toml = r#"[package]
name = "test-project"
version = "1.0.0"
edition = "2021"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    temp_dir
}

#[test]
#[serial]
fn test_verset_init() {
    let temp_dir = setup_test_project();
    let project_dir = temp_dir.path();

    // Save current directory
    let original_dir = std::env::current_dir().unwrap();

    // Change to the test directory
    std::env::set_current_dir(project_dir).unwrap();

    // Initialize verset
    let result = verset::config::init();

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    result.unwrap();

    // Check that .verset directory was created
    assert!(project_dir.join(".verset").exists());
    assert!(project_dir.join(".verset/config.toml").exists());
    assert!(project_dir.join(".verset/changesets").exists());
}

#[test]
#[serial]
fn test_changeset_workflow() {
    let temp_dir = setup_test_project();
    let project_dir = temp_dir.path();

    // Save current directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(project_dir).unwrap();

    // Initialize verset
    verset::config::init().unwrap();

    // Create a changeset
    let changeset = verset::Changeset {
        packages: vec!["test-project".to_string()],
        change_type: verset::ChangeType::Minor,
        description: "Test changeset".to_string(),
        created_at: chrono::Utc::now(),
    };

    verset::changeset::save_changeset(&changeset).unwrap();

    // Load changesets
    let changesets = verset::changeset::load_changesets().unwrap();

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();

    assert_eq!(changesets.len(), 1);
    assert_eq!(changesets[0].1.packages, vec!["test-project"]);
    assert_eq!(changesets[0].1.change_type, verset::ChangeType::Minor);
    assert_eq!(changesets[0].1.description, "Test changeset");
}

#[test]
fn test_change_type_formatting() {
    // Test the format_changes function indirectly
    assert_eq!(verset::ChangeType::Major.to_string(), "major");
    assert_eq!(verset::ChangeType::Minor.to_string(), "minor");
    assert_eq!(verset::ChangeType::Patch.to_string(), "patch");
}
