use rstest::rstest;
use zerv::test_utils::ZervFixture;
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

// ============================================================================
// Input Format Tests
// ============================================================================

// ============================================================================
// Output Format Tests - Basic Conversions
// ============================================================================

#[rstest]
#[case::semver_basic((1, 2, 3), "semver", "1.2.3")]
#[case::pep440_basic((1, 2, 3), "pep440", "1.2.3")]
fn test_output_format_basic(
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

#[test]
fn test_output_format_zerv_roundtrip() {
    let original_zerv = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .build();

    let zerv_ron = original_zerv.to_string();

    let output = TestCommand::new()
        .args_from_str("version --source stdin --output-format zerv")
        .stdin(zerv_ron)
        .assert_success();

    let parsed_zerv: zerv::version::Zerv =
        ron::from_str(output.stdout().trim()).expect("Failed to parse output as Zerv RON");

    assert_eq!(
        parsed_zerv, original_zerv,
        "Zerv output format should preserve structure"
    );
}

// ============================================================================
// Output Format Tests - Pre-release Versions
// ============================================================================

#[rstest]
#[case::alpha_semver(PreReleaseLabel::Alpha, Some(1), "semver", "1.0.0-alpha.1")]
#[case::alpha_pep440(PreReleaseLabel::Alpha, Some(1), "pep440", "1.0.0a1")]
#[case::beta_semver(PreReleaseLabel::Beta, Some(2), "semver", "1.0.0-beta.2")]
#[case::beta_pep440(PreReleaseLabel::Beta, Some(2), "pep440", "1.0.0b2")]
#[case::rc_semver(PreReleaseLabel::Rc, Some(3), "semver", "1.0.0-rc.3")]
#[case::rc_pep440(PreReleaseLabel::Rc, Some(3), "pep440", "1.0.0rc3")]
fn test_output_format_prerelease_conversion(
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

// ============================================================================
// Output Format Tests - Extended Version Features (Epoch, Post, Dev)
// ============================================================================

#[rstest]
#[case::epoch_only_pep440(Some(2), None, None, "pep440", "2!1.0.0")]
#[case::epoch_only_semver(Some(2), None, None, "semver", "1.0.0-epoch.2")]
#[case::post_only_pep440(None, Some(5), None, "pep440", "1.0.0.post5")]
#[case::post_only_semver(None, Some(5), None, "semver", "1.0.0-post.5")]
#[case::dev_only_pep440(None, None, Some(3), "pep440", "1.0.0.dev3")]
#[case::dev_only_semver(None, None, Some(3), "semver", "1.0.0-dev.3")]
#[case::epoch_post_pep440(Some(1), Some(2), None, "pep440", "1!1.0.0.post2")]
#[case::epoch_post_semver(Some(1), Some(2), None, "semver", "1.0.0-epoch.1.post.2")]
#[case::epoch_dev_pep440(Some(1), None, Some(4), "pep440", "1!1.0.0.dev4")]
#[case::epoch_dev_semver(Some(1), None, Some(4), "semver", "1.0.0-epoch.1.dev.4")]
#[case::post_dev_pep440(None, Some(2), Some(5), "pep440", "1.0.0.post2.dev5")]
#[case::post_dev_semver(None, Some(2), Some(5), "semver", "1.0.0-post.2.dev.5")]
#[case::epoch_post_dev_pep440(Some(1), Some(2), Some(3), "pep440", "1!1.0.0.post2.dev3")]
#[case::epoch_post_dev_semver(Some(1), Some(2), Some(3), "semver", "1.0.0-epoch.1.post.2.dev.3")]
fn test_output_format_extended_features(
    #[case] epoch: Option<u64>,
    #[case] post: Option<u64>,
    #[case] dev: Option<u64>,
    #[case] format: &str,
    #[case] expected: &str,
) {
    let mut fixture = ZervFixture::new()
        .with_version(1, 0, 0)
        .with_standard_tier_3();

    if let Some(e) = epoch {
        fixture = fixture.with_epoch(e);
    }

    if let Some(p) = post {
        fixture = fixture.with_post(p);
    }

    if let Some(d) = dev {
        fixture = fixture.with_dev(d)
    }

    let zerv_ron = fixture.build().to_string();

    let output = TestCommand::new()
        .args_from_str(format!("version --source stdin --output-format {format}"))
        .stdin(zerv_ron)
        .assert_success();

    assert_eq!(output.stdout().trim(), expected);
}

// ============================================================================
// Format Combination Tests
// ============================================================================

/// Test that different output formats work correctly with stdin source
#[rstest]
#[case::semver_to_pep440("1.2.3-alpha.1", "pep440", "1.2.3a1")]
#[case::semver_to_semver("1.2.3-alpha.1", "semver", "1.2.3-alpha.1")]
#[case::semver_with_build_to_pep440("1.2.3-alpha.1+some.build", "pep440", "1.2.3a1+some.build")]
#[case::semver_with_build_to_semver(
    "1.2.3-alpha.1+some.build",
    "semver",
    "1.2.3-alpha.1+some.build"
)]
#[case::pep440_to_semver("1.2.3a1", "semver", "1.2.3-alpha.1")]
#[case::pep440_to_pep440("1.2.3a1", "pep440", "1.2.3a1")]
fn test_output_format_with_different_inputs(
    #[case] input_version: &str,
    #[case] output_format: &str,
    #[case] expected: &str,
) {
    let zerv_ron = if input_version.contains('-') || input_version.contains('+') {
        // SemVer format
        ZervFixture::from_semver_str(input_version)
            .build()
            .to_string()
    } else if input_version.contains('a')
        || input_version.contains('b')
        || input_version.contains("rc")
        || input_version.contains('!')
    {
        // PEP440 format
        ZervFixture::from_pep440_str(input_version)
            .build()
            .to_string()
    } else {
        // Basic version
        let parts: Vec<&str> = input_version.split('.').collect();
        ZervFixture::new()
            .with_version(
                parts[0].parse().unwrap(),
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
            )
            .build()
            .to_string()
    };

    let output = TestCommand::new()
        .args_from_str(format!(
            "version --source stdin --output-format {output_format}"
        ))
        .stdin(zerv_ron)
        .assert_success();

    assert_eq!(output.stdout().trim(), expected);
}

// ============================================================================
// Format Validation and Error Handling
// ============================================================================

#[test]
fn test_invalid_output_format_rejected_by_clap() {
    // Invalid output format should be rejected by clap's value_parser
    let output = TestCommand::new()
        .args_from_str("version --output-format invalid")
        .assert_failure();

    let stderr = output.stderr();
    assert!(
        stderr.contains("invalid value 'invalid'") || stderr.contains("--output-format"),
        "Should show clap validation error for invalid output format, got: {stderr}"
    );
}

// ============================================================================
// Format Consistency Tests
// ============================================================================

/// Test that format conversions are symmetric where possible
#[rstest]
#[case::basic_version("1.2.3")]
#[case::prerelease_alpha("1.0.0-alpha.1")]
#[case::prerelease_beta("2.0.0-beta.2")]
#[case::prerelease_rc("3.0.0-rc.1")]
fn test_format_roundtrip_semver_pep440(#[case] original_version: &str) {
    // SemVer -> PEP440 -> SemVer roundtrip using stdin
    let semver_zerv = ZervFixture::from_semver_str(original_version)
        .build()
        .to_string();

    let pep440_output = TestCommand::new()
        .args_from_str("version --source stdin --output-format pep440")
        .stdin(semver_zerv)
        .assert_success();

    let pep440_zerv = ZervFixture::from_pep440_str(pep440_output.stdout().trim())
        .build()
        .to_string();

    let semver_output = TestCommand::new()
        .args_from_str("version --source stdin --output-format semver")
        .stdin(pep440_zerv)
        .assert_success();

    assert_eq!(
        semver_output.stdout().trim(),
        original_version,
        "Roundtrip conversion should preserve version"
    );
}

#[test]
fn test_output_format_zerv_with_complex_structure() {
    // Test that complex Zerv structures are preserved through zerv output format
    let original_zerv = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_epoch(2)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_post(1)
        .build();

    let zerv_ron = original_zerv.to_string();

    let output = TestCommand::new()
        .args_from_str("version --source stdin --output-format zerv")
        .stdin(zerv_ron)
        .assert_success();

    let parsed_zerv: zerv::version::Zerv =
        ron::from_str(output.stdout().trim()).expect("Failed to parse complex Zerv RON");

    assert_eq!(parsed_zerv, original_zerv);
}
