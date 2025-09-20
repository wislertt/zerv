pub mod check;
pub mod help_flags;
pub mod util;
pub mod version;

use util::TestCommand;
use zerv::test_utils::GitRepoFixture;

/// Test a version command with output format
pub fn test_version_output_format(
    fixture: &GitRepoFixture,
    format: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = TestCommand::new()
        .current_dir(fixture.path())
        .arg("version")
        .arg("--output-format")
        .arg(format)
        .assert_success();

    Ok(output.stdout().to_string())
}
