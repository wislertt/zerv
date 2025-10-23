use rstest::rstest;
use zerv::test_utils::ZervFixture;
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

mod output_format_basic {
    //! Tests for basic format conversions (semver ↔ pep440 ↔ zerv)
    use super::*;

    #[rstest]
    #[case::semver_basic((1, 2, 3), "semver", "1.2.3")]
    #[case::pep440_basic((1, 2, 3), "pep440", "1.2.3")]
    fn test_basic(#[case] version: (u64, u64, u64), #[case] format: &str, #[case] expected: &str) {
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

    #[test]
    fn test_zerv_roundtrip() {
        let original_zerv = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .build();

        let zerv_ron = original_zerv.to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --output-format zerv", zerv_ron);

        let parsed_zerv: zerv::version::Zerv =
            ron::from_str(&output).expect("Failed to parse output as Zerv RON");

        assert_eq!(
            parsed_zerv, original_zerv,
            "Zerv output format should preserve structure"
        );
    }
}

mod output_format_prerelease {
    //! Tests for prerelease version format conversions
    use super::*;

    #[rstest]
    #[case::alpha_semver(PreReleaseLabel::Alpha, Some(1), "semver", "1.0.0-alpha.1")]
    #[case::alpha_pep440(PreReleaseLabel::Alpha, Some(1), "pep440", "1.0.0a1")]
    #[case::beta_semver(PreReleaseLabel::Beta, Some(2), "semver", "1.0.0-beta.2")]
    #[case::beta_pep440(PreReleaseLabel::Beta, Some(2), "pep440", "1.0.0b2")]
    #[case::rc_semver(PreReleaseLabel::Rc, Some(3), "semver", "1.0.0-rc.3")]
    #[case::rc_pep440(PreReleaseLabel::Rc, Some(3), "pep440", "1.0.0rc3")]
    fn test_conversion(
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
}

mod output_format_extended {
    //! Tests for extended version features (epoch, post, dev)
    use super::*;

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
    #[case::epoch_post_dev_semver(
        Some(1),
        Some(2),
        Some(3),
        "semver",
        "1.0.0-epoch.1.post.2.dev.3"
    )]
    fn test_features(
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

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --output-format {format}"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }
}

mod format_combinations {
    //! Tests for format conversions with different input formats
    use super::*;

    #[rstest]
    #[case::semver_to_pep440(
        ZervFixture::from_semver_str("1.2.3-alpha.1+some.build"),
        "pep440",
        "1.2.3a1+some.build"
    )]
    #[case::semver_to_semver(
        ZervFixture::from_semver_str("1.2.3-alpha.1+some.build"),
        "semver",
        "1.2.3-alpha.1+some.build"
    )]
    #[case::pep440_to_semver(
        ZervFixture::from_pep440_str("1.2.3a1+some.build"),
        "semver",
        "1.2.3-alpha.1+some.build"
    )]
    #[case::pep440_to_pep440(
        ZervFixture::from_pep440_str("1.2.3a1+some.build"),
        "pep440",
        "1.2.3a1+some.build"
    )]
    fn test_with_different_inputs(
        #[case] zerv_fixture: ZervFixture,
        #[case] output_format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = zerv_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --output-format {output_format}"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }
}

mod format_validation {
    //! Tests for format validation and error handling
    use super::*;

    #[test]
    fn test_invalid_output_format_rejected_by_clap() {
        let output = TestCommand::new()
            .args_from_str("version --output-format invalid")
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("invalid value 'invalid'") || stderr.contains("--output-format"),
            "Should show clap validation error for invalid output format, got: {stderr}"
        );
    }
}

mod format_roundtrip {
    //! Tests for symmetric format conversions
    use super::*;

    #[rstest]
    #[case::basic_version("1.2.3")]
    #[case::prerelease_alpha("1.0.0-alpha.1")]
    #[case::prerelease_beta("2.0.0-beta.2")]
    #[case::prerelease_rc("3.0.0-rc.1")]
    fn test_semver_pep440_roundtrip(#[case] original_version: &str) {
        let semver_zerv = ZervFixture::from_semver_str(original_version)
            .build()
            .to_string();

        let pep440_output = TestCommand::run_with_stdin(
            "version --source stdin --output-format pep440",
            semver_zerv,
        );

        let pep440_zerv = ZervFixture::from_pep440_str(&pep440_output)
            .build()
            .to_string();

        let semver_output = TestCommand::run_with_stdin(
            "version --source stdin --output-format semver",
            pep440_zerv,
        );

        assert_eq!(
            semver_output, original_version,
            "Roundtrip conversion should preserve version"
        );
    }

    #[test]
    fn test_zerv_with_complex_structure() {
        let original_zerv = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(1)
            .build();

        let zerv_ron = original_zerv.to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --output-format zerv", zerv_ron);

        let parsed_zerv: zerv::version::Zerv =
            ron::from_str(&output).expect("Failed to parse complex Zerv RON");

        assert_eq!(parsed_zerv, original_zerv);
    }
}
