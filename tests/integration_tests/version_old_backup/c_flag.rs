use super::TestCommand;
use rstest::rstest;

/// Test the -C flag behavior for changing directory before running commands
mod c_flag_tests {
    use super::*;

    #[rstest]
    #[case(".", true)] // Current directory should work
    #[case("..", false)] // Parent directory should fail
    #[case("src", false)] // src directory should fail
    #[case("nonexistent", false)] // Nonexistent directory should fail
    fn test_c_flag_with_various_directories(#[case] directory: &str, #[case] should_succeed: bool) {
        let mut cmd = TestCommand::new();
        cmd.args(["version", "-C", directory]);

        if should_succeed {
            let result = cmd.assert_success();
            let stdout = result.stdout();
            assert!(
                !stdout.is_empty(),
                "Should produce version output for directory: '{directory}'"
            );
            assert!(
                stdout.len() > 5,
                "Version output should be substantial for directory: '{directory}'"
            );
        } else {
            let result = cmd.assert_failure();
            let stderr = result.stderr();

            if directory == "nonexistent" {
                // Should fail with either "Not in a git repository" or a path error
                assert!(
                    stderr.contains("Not in a git repository") || stderr.contains("No such file"),
                    "Should fail with appropriate error for directory '{directory}', got: {stderr}"
                );
            } else {
                assert!(
                    stderr.contains("Not in a git repository"),
                    "Should fail with 'Not in a git repository' error for directory '{directory}', got: {stderr}"
                );
            }
        }
    }

    #[rstest]
    #[case(".")]
    #[case("./")]
    #[case("")]
    fn test_c_flag_variations(#[case] path: &str) {
        // Test various ways to specify current directory
        let mut cmd = TestCommand::new();
        if path.is_empty() {
            cmd.args(["version"]);
        } else {
            cmd.args(["version", "-C", path]);
        }

        let result = cmd.assert_success();
        let stdout = result.stdout();
        assert!(
            !stdout.is_empty(),
            "Should produce version output for path: '{path}'"
        );
    }

    #[test]
    fn test_c_flag_with_absolute_path() {
        // Test -C with absolute path to current directory should work
        let current_dir = std::env::current_dir().unwrap();
        let current_dir_str = current_dir.to_string_lossy();

        let mut cmd = TestCommand::new();
        cmd.args(["version", "-C", &current_dir_str]);

        let result = cmd.assert_success();
        let stdout = result.stdout();
        assert!(!stdout.is_empty(), "Should produce version output");
    }

    #[test]
    fn test_c_flag_with_absolute_path_parent_fails() {
        // Test -C with absolute path to parent directory should fail
        let current_dir = std::env::current_dir().unwrap();
        let parent_dir = current_dir.parent().unwrap();
        let parent_dir_str = parent_dir.to_string_lossy();

        let mut cmd = TestCommand::new();
        cmd.args(["version", "-C", &parent_dir_str]);

        let result = cmd.assert_failure();
        let stderr = result.stderr();
        assert!(
            stderr.contains("Not in a git repository"),
            "Should fail with 'Not in a git repository' error, got: {stderr}"
        );
    }

    #[test]
    fn test_no_c_flag_works() {
        // Test without -C flag should work (default behavior)
        let mut cmd = TestCommand::new();
        cmd.args(["version"]);

        let result = cmd.assert_success();
        let stdout = result.stdout();
        assert!(!stdout.is_empty(), "Should produce version output");
    }

    #[test]
    fn test_c_flag_consistency() {
        // Test that -C . produces the same output as no -C flag
        let mut cmd_no_c = TestCommand::new();
        cmd_no_c.args(["version"]);
        let result_no_c = cmd_no_c.assert_success();

        let mut cmd_with_c = TestCommand::new();
        cmd_with_c.args(["version", "-C", "."]);
        let result_with_c = cmd_with_c.assert_success();

        assert_eq!(
            result_no_c.stdout(),
            result_with_c.stdout(),
            "Output should be identical with and without -C ."
        );
    }

    #[rstest]
    #[case("..", "Not in a git repository")]
    #[case("src", "Not in a git repository")]
    #[case("nonexistent", "Not in a git repository")]
    fn test_c_flag_error_message_quality(#[case] directory: &str, #[case] expected_error: &str) {
        // Test that error messages are helpful and consistent
        let mut cmd = TestCommand::new();
        cmd.args(["version", "-C", directory]);

        let result = cmd.assert_failure();
        let stderr = result.stderr();

        // Should contain the expected error message
        if directory == "nonexistent" {
            // For nonexistent directory, might get either error message
            assert!(
                stderr.contains(expected_error) || stderr.contains("No such file"),
                "Error should mention '{expected_error}' or 'No such file' for directory '{directory}', got: {stderr}"
            );
        } else {
            assert!(
                stderr.contains(expected_error),
                "Error should mention '{expected_error}' for directory '{directory}', got: {stderr}"
            );
        }

        // Should mention --source git
        assert!(
            stderr.contains("--source git"),
            "Error should mention '--source git' for directory '{directory}', got: {stderr}"
        );

        // Should not contain misleading "Git command not found" message
        assert!(
            !stderr.contains("Git command not found"),
            "Error should not mention 'Git command not found' for directory '{directory}', got: {stderr}"
        );
    }

    #[rstest]
    #[case("..", "parent directory")]
    #[case("src", "src directory")]
    #[case("nonexistent", "nonexistent directory")]
    fn test_c_flag_error_context(#[case] directory: &str, #[case] description: &str) {
        // Test that error messages provide good context
        let mut cmd = TestCommand::new();
        cmd.args(["version", "-C", directory]);

        let result = cmd.assert_failure();
        let stderr = result.stderr();

        // Error should be clear and actionable
        assert!(
            stderr.contains("Not in a git repository") || stderr.contains("No such file"),
            "Error should be clear for {description} ({directory}), got: {stderr}"
        );
    }

    #[rstest]
    #[case("..", "parent")]
    #[case("src", "subdirectory")]
    #[case("nonexistent", "missing")]
    fn test_c_flag_error_consistency(#[case] directory: &str, #[case] _type: &str) {
        // Test that error messages are consistent across different directory types
        let mut cmd = TestCommand::new();
        cmd.args(["version", "-C", directory]);

        let result = cmd.assert_failure();
        let stderr = result.stderr();

        // All should mention the same core error
        assert!(
            stderr.contains("Not in a git repository") || stderr.contains("No such file"),
            "Error should be consistent for {_type} directory '{directory}', got: {stderr}"
        );
    }
}
