//! Primary component bump tests
//!
//! Tests for --bump-major, --bump-minor, and --bump-patch options.
//! These tests verify that primary component bumps work correctly
//! across different output formats and preserve other version data.

use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

/// Zerv fixture starting from version 1.2.3 for primary bump tests
#[fixture]
fn primary_bump_fixture() -> ZervFixture {
    ZervFixture::new().with_version(1, 2, 3)
}

/// Zerv fixture with prerelease data for primary bump tests
#[fixture]
fn primary_bump_with_prerelease_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
}

/// Zerv fixture with VCS data for primary bump tests
#[fixture]
fn primary_bump_with_vcs_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_distance(5)
        .with_dirty(true)
}

mod major_bump {
    use super::*;

    #[rstest]
    #[case::semver("semver", "2.0.0")]
    #[case::pep440("pep440", "2.0.0")]
    fn test_bump_major_simple(
        primary_bump_fixture: ZervFixture,
        #[case] format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --output-format {} --bump-major",
                format
            ),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_bump_major_zerv_format(primary_bump_fixture: ZervFixture) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --output-format zerv --bump-major",
            zerv_ron,
        );
        // ZERV format should contain the bumped major version
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("minor: Some(0)"));
        assert!(output.contains("patch: Some(0)"));
    }

    #[rstest]
    #[case("3.0.0", "2")] // Custom major bump value
    #[case("5.0.0", "4")] // Larger major bump value
    fn test_bump_major_with_value(
        primary_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --bump-major {}", bump_value),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_bump_major_resets_prerelease(primary_bump_with_prerelease_fixture: ZervFixture) {
        let zerv_ron = primary_bump_with_prerelease_fixture.build().to_string();
        let output = TestCommand::run_with_stdin("version --source stdin --bump-major", zerv_ron);
        // Bumps reset prerelease to stable version
        assert_eq!(output, "2.0.0");
    }

    #[rstest]
    fn test_bump_major_preserve_vcs(primary_bump_with_vcs_fixture: ZervFixture) {
        let zerv_ron = primary_bump_with_vcs_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --output-format zerv",
            zerv_ron,
        );
        // Should preserve VCS data but reset prerelease
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("distance: Some(5)"));
        assert!(output.contains("dirty: Some(true)"));
        // Prerelease should be reset (None)
        assert!(output.contains("pre_release: None"));
    }
}

mod minor_bump {
    use super::*;

    #[rstest]
    #[case::semver("semver", "1.3.0")]
    #[case::pep440("pep440", "1.3.0")]
    fn test_bump_minor_simple(
        primary_bump_fixture: ZervFixture,
        #[case] format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --output-format {} --bump-minor",
                format
            ),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_bump_minor_zerv_format(primary_bump_fixture: ZervFixture) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --output-format zerv --bump-minor",
            zerv_ron,
        );
        // ZERV format should contain the bumped minor version
        assert!(output.contains("major: Some(1)"));
        assert!(output.contains("minor: Some(3)"));
        assert!(output.contains("patch: Some(0)"));
    }

    #[rstest]
    #[case("1.4.0", "2")] // Custom minor bump value
    fn test_bump_minor_with_value(
        primary_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --bump-minor {}", bump_value),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_bump_minor_resets_prerelease(primary_bump_with_prerelease_fixture: ZervFixture) {
        let zerv_ron = primary_bump_with_prerelease_fixture.build().to_string();
        let output = TestCommand::run_with_stdin("version --source stdin --bump-minor", zerv_ron);
        // Bumps reset prerelease to stable version
        assert_eq!(output, "1.3.0");
    }
}

mod patch_bump {
    use super::*;

    #[rstest]
    #[case::semver("semver", "1.2.4")]
    #[case::pep440("pep440", "1.2.4")]
    fn test_bump_patch_simple(
        primary_bump_fixture: ZervFixture,
        #[case] format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --output-format {} --bump-patch",
                format
            ),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_bump_patch_zerv_format(primary_bump_fixture: ZervFixture) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --output-format zerv --bump-patch",
            zerv_ron,
        );
        // ZERV format should contain the bumped patch version
        assert!(output.contains("major: Some(1)"));
        assert!(output.contains("minor: Some(2)"));
        assert!(output.contains("patch: Some(4)"));
    }

    #[rstest]
    #[case("1.2.5", "2")] // Custom patch bump value
    fn test_bump_patch_with_value(
        primary_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --bump-patch {}", bump_value),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_bump_patch_resets_prerelease(primary_bump_with_prerelease_fixture: ZervFixture) {
        let zerv_ron = primary_bump_with_prerelease_fixture.build().to_string();
        let output = TestCommand::run_with_stdin("version --source stdin --bump-patch", zerv_ron);
        // Bumps reset prerelease to stable version
        assert_eq!(output, "1.2.4");
    }
}

mod primary_combinations {
    use super::*;

    #[rstest]
    fn test_multiple_primary_bumps(primary_bump_fixture: ZervFixture) {
        let zerv_ron = primary_bump_fixture.build().to_string();

        // Test major + minor (major bumps reset minor to 0, then minor bump adds 1)
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --bump-minor",
            zerv_ron.clone(),
        );
        assert_eq!(output, "2.1.0");

        // Test major + minor + patch
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --bump-minor --bump-patch",
            zerv_ron,
        );
        assert_eq!(output, "2.1.1");
    }

    #[rstest]
    fn test_primary_bumps_with_custom_values(primary_bump_fixture: ZervFixture) {
        let zerv_ron = primary_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major 2 --bump-minor 3",
            zerv_ron,
        );
        // major bump by 2 (1->3), which resets minor to 0, then minor bump by 3 (0->3)
        assert_eq!(output, "3.3.0");
    }

    #[rstest]
    fn test_primary_bumps_preserve_vcs_reset_prerelease(
        primary_bump_with_vcs_fixture: ZervFixture,
    ) {
        let zerv_ron = primary_bump_with_vcs_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --bump-minor --output-format zerv",
            zerv_ron,
        );
        // Should preserve VCS data but reset prerelease
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("minor: Some(1)")); // major bump resets minor to 0, then minor bump adds 1
        assert!(output.contains("patch: Some(0)"));
        assert!(output.contains("distance: Some(5)"));
        assert!(output.contains("dirty: Some(true)"));
        // Prerelease should be reset (None)
        assert!(output.contains("pre_release: None"));
    }
}
