use super::TestCommand;

#[test]
fn test_version_source_git_default() {
    let test_output = TestCommand::new()
        .arg("version")
        .arg("--source")
        .arg("git")
        .assert_success();

    let output = test_output.stdout();

    // Should generate some version output
    assert!(
        !output.trim().is_empty(),
        "Git source should generate version output"
    );
}

// TODO: Implement --source string option
// #[test]
// fn test_version_source_string() {
//     let test_output = TestCommand::new()
//         .arg("version")
//         .arg("--source")
//         .arg("string")
//         .arg("1.2.3")
//         .assert_success();
//
//     let output = test_output.stdout();
//
//     // Should contain the provided version string
//     assert!(
//         output.contains("1.2.3"),
//         "String source should use provided version: {output}"
//     );
// }
