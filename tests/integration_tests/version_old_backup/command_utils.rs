use super::{GitRepoFixture, TestCommand};

/// Command execution utilities for integration tests
pub struct VersionCommandUtils;

impl VersionCommandUtils {
    /// Run version command and return output
    pub fn run_version_command(fixture: &GitRepoFixture) -> String {
        let test_output = TestCommand::new()
            .current_dir(fixture.path())
            .arg("version")
            .assert_success();

        test_output.stdout().trim().to_string()
    }

    /// Run version command with specific format
    pub fn run_version_command_with_format(fixture: &GitRepoFixture, format: &str) -> String {
        let test_output = TestCommand::new()
            .current_dir(fixture.path())
            .arg("version")
            .arg("--output-format")
            .arg(format)
            .assert_success();

        test_output.stdout().trim().to_string()
    }
}
