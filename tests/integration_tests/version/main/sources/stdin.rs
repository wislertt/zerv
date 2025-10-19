use rstest::rstest;
use zerv::test_utils::ZervFixture;
use zerv::version::{
    PreReleaseLabel,
    Zerv,
};

use crate::util::TestCommand;

#[rstest]
#[case::basic_semver((1, 2, 3), "semver", "1.2.3")]
#[case::basic_pep440((2, 0, 0), "pep440", "2.0.0")]
fn test_stdin_basic_output(
    #[case] version: (u64, u64, u64),
    #[case] format: &str,
    #[case] expected: &str,
) {
    let zerv_ron = ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .build()
        .to_string();

    let output = TestCommand::new()
        .args_from_str(format!("version --source stdin --output-format {format}"))
        .stdin(zerv_ron)
        .assert_success();

    assert_eq!(output.stdout().trim(), expected);
}

#[rstest]
#[case::alpha_semver(PreReleaseLabel::Alpha, Some(1), "semver", "1.0.0-alpha.1")]
#[case::beta_pep440(PreReleaseLabel::Beta, Some(2), "pep440", "1.0.0b2")]
fn test_stdin_with_prerelease(
    #[case] label: PreReleaseLabel,
    #[case] number: Option<u64>,
    #[case] format: &str,
    #[case] expected: &str,
) {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 0, 0)
        .with_pre_release(label, number)
        .build()
        .to_string();

    let output = TestCommand::new()
        .args_from_str(format!("version --source stdin --output-format {format}"))
        .stdin(zerv_ron)
        .assert_success();

    assert_eq!(output.stdout().trim(), expected);
}

#[rstest]
#[case::epoch(Some(2), None, "2!1.0.0")]
#[case::post(None, Some(5), "1.0.0.post5")]
fn test_stdin_pep440_features(
    #[case] epoch: Option<u64>,
    #[case] post: Option<u64>,
    #[case] expected: &str,
) {
    let mut fixture = ZervFixture::new().with_version(1, 0, 0);

    if let Some(e) = epoch {
        fixture = fixture.with_epoch(e);
    }

    if let Some(p) = post {
        fixture = fixture.with_post(p);
    }

    let zerv_ron = fixture.build().to_string();

    let output = TestCommand::new()
        .args_from_str("version --source stdin --output-format pep440")
        .stdin(zerv_ron)
        .assert_success();

    assert_eq!(output.stdout().trim(), expected);
}

#[test]
fn test_stdin_zerv_roundtrip() {
    let original_zerv = ZervFixture::new().with_version(3, 1, 4).build();

    let zerv_ron = original_zerv.to_string();

    let output = TestCommand::new()
        .args_from_str("version --source stdin --output-format zerv")
        .stdin(zerv_ron)
        .assert_success();

    let parsed_zerv: Zerv =
        ron::from_str(output.stdout().trim()).expect("Failed to parse output as Zerv RON");

    assert_eq!(
        parsed_zerv, original_zerv,
        "Stdin roundtrip should preserve Zerv structure"
    );
}
