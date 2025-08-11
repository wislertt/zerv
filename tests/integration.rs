mod util;

use rstest::rstest;
use util::{TestCommand, TestDir};

#[test]
fn test_default_output() {
    TestCommand::new()
        .assert_success()
        .assert_stdout_contains("1.2.3")
        .assert_stdout_contains("Debug: Version");
}

#[rstest]
#[case("-V")]
#[case("--version")]
fn test_version_flags(#[case] flag: &str) {
    TestCommand::new()
        .arg(flag)
        .assert_success()
        .assert_stdout_contains("zerv 0.0.0");
}

#[test]
fn test_dir_utility_demo() {
    let dir = TestDir::new().expect("Failed to create test dir");

    // Create some test files
    dir.create_file("test.txt", "hello world").unwrap();
    dir.create_dir("subdir").unwrap();
    dir.create_file("subdir/nested.txt", "nested content")
        .unwrap();

    // Initialize dummy git repo
    dir.init_git().unwrap();

    // Verify files were created
    assert!(dir.path().join("test.txt").exists());
    assert!(dir.path().join("subdir").exists());
    assert!(dir.path().join("subdir/nested.txt").exists());
    assert!(dir.path().join(".git").exists());
    assert!(dir.path().join(".git/HEAD").exists());

    // Directory is automatically cleaned up when dropped
}
