pub mod git;
pub mod none;
pub mod smart_default;
pub mod stdin;

use rstest::rstest;

use crate::util::TestCommand;

#[rstest]
#[case("unknown")]
#[case("invalid")]
#[case("xyz")]
fn test_invalid_source_error(#[case] invalid_source: &str) {
    let output = TestCommand::new()
        .args_from_str(format!("version --source {invalid_source}"))
        .assert_failure();

    let stderr = output.stderr();
    assert!(stderr.contains(&format!("invalid value '{invalid_source}'")));
    assert!(stderr.contains("possible values: git, stdin, none"));
    assert!(stderr.contains("For more information, try '--help'"));
}
