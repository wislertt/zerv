use rstest::rstest;
use zerv::schema::ZervSchemaPreset;
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

    let output = TestCommand::run_with_stdin(
        &format!("version --source stdin --output-format {format}"),
        zerv_ron,
    );

    assert_eq!(output, expected);
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

    let output = TestCommand::run_with_stdin(
        &format!("version --source stdin --output-format {format}"),
        zerv_ron,
    );

    assert_eq!(output, expected);
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

    let output =
        TestCommand::run_with_stdin("version --source stdin --output-format pep440", zerv_ron);

    assert_eq!(output, expected);
}

#[rstest]
#[case::standard_tier_1(ZervFixture::new().with_schema_preset(ZervSchemaPreset::StandardBasePrerelease).with_version(3, 1, 4))]
#[case::standard_tier_2(ZervFixture::new().with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostContext).with_version(2, 0, 0))]
#[case::standard_tier_3(ZervFixture::new().with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostDevContext).with_version(1, 5, 2))]
#[case::calver_tier_1(ZervFixture::new().with_schema_preset(ZervSchemaPreset::CalverBasePrerelease).with_version(2024, 12, 1))]
#[case::calver_tier_2(ZervFixture::new().with_schema_preset(ZervSchemaPreset::CalverBasePrereleasePostContext).with_version(2024, 1, 0))]
#[case::calver_tier_3(ZervFixture::new().with_schema_preset(ZervSchemaPreset::CalverBasePrereleasePostDevContext).with_version(2023, 8, 15))]
fn test_stdin_zerv_roundtrip(#[case] fixture: ZervFixture) {
    let original_zerv = fixture.build();

    let zerv_ron = original_zerv.to_string();

    let output =
        TestCommand::run_with_stdin("version --source stdin --output-format zerv", zerv_ron);

    let parsed_zerv: Zerv = ron::from_str(&output).expect("Failed to parse output as Zerv RON");

    assert_eq!(
        parsed_zerv, original_zerv,
        "Stdin roundtrip should preserve Zerv structure"
    );
}

#[test]
fn test_stdin_without_input_returns_error() {
    // Test that running with --source stdin but without providing stdin input
    // returns an error immediately instead of hanging
    // This verifies the terminal detection is working correctly

    let output = TestCommand::new()
        .args_from_str("version --source stdin --output-format semver")
        // Note: No .stdin() call here - stdin is not provided
        .assert_failure();

    // Verify we get a helpful error message about stdin being required
    let stderr = output.stderr();
    assert!(
        stderr.contains("No input provided via stdin") || stderr.contains("stdin"),
        "Error message should mention stdin requirement. Got: {}",
        stderr
    );
}
