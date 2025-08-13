mod util;

use rstest::rstest;
use util::{TestCommand, TestDir};

#[rstest]
#[case("1.2.3")]
#[case("Debug: PEP440Version")]
fn test_default_output_contains(#[case] expected_text: &str) {
    TestCommand::new()
        .assert_success()
        .assert_stdout_contains(expected_text);
}

#[rstest]
#[case("-V")]
#[case("--version")]
#[case("-h")]
#[case("--help")]
fn test_help_and_version_flags(#[case] flag: &str) {
    TestCommand::new().arg(flag).assert_success();
}

#[rstest]
#[case("test.txt", "hello world")]
#[case("subdir/nested.txt", "nested content")]
#[case("deep/path/file.txt", "deep content")]
fn test_dir_create_file_variations(#[case] path: &str, #[case] content: &str) {
    let dir = TestDir::new().expect("Failed to create test dir");
    dir.create_file(path, content).unwrap();
    let file_path = dir.path().join(path);
    assert!(file_path.exists());
    assert_eq!(std::fs::read_to_string(&file_path).unwrap(), content);
}

#[test]
fn test_dir_utility_demo() {
    let dir = TestDir::new().expect("Failed to create test dir");
    dir.init_git().unwrap();
    assert!(dir.path().join(".git").exists());
    assert!(dir.path().join(".git/HEAD").exists());
}
