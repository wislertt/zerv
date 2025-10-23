//! Secondary component bump tests
//!
//! Tests for --bump-epoch, --bump-post, --bump-dev, --bump-pre-release-num,
//! and --bump-pre-release-label options. These tests verify that secondary
//! component bumps work correctly and interact properly with other version data.

use rstest::{fixture, rstest};
use crate::util::TestCommand;
use zerv::test_utils::ZervFixture;
use zerv::utils::constants::{
    sources::STDIN,
    formats::SEMVER,
    formats::PEP440,
    formats::ZERV,
};
use zerv::version::PreReleaseLabel;
use super::{base_zerv_fixture};

/// Zerv fixture with secondary components for bump tests
#[fixture]
fn secondary_bump_fixture() -> ZervFixture {
    base_zerv_fixture()
        .with_epoch(1)
        .with_post(2)
        .with_dev(3)
}

/// Zerv fixture with prerelease for label bump tests
#[fixture]
fn prerelease_bump_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
}

/// Zerv fixture with all secondary components
#[fixture]
fn full_secondary_fixture() -> ZervFixture {
    secondary_bump_fixture()
        .with_pre_release(PreReleaseLabel::Beta, Some(2))
}

mod epoch_bump {
    use super::*;

    #[rstest]
    #[case("2!1.2.3-alpha.1", ZERV)]  // Simple epoch bump
    fn test_bump_epoch_simple(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let input = base_zerv_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!("version --source {} --output-format {} --bump-epoch", STDIN, format),
            input
        );

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3!1.2.3-alpha.1", "2")]  // Custom epoch bump value
    #[case("5!1.2.3-alpha.1", "4")]  // Larger epoch bump value
    fn test_bump_epoch_with_value(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let input = base_zerv_fixture.build().to_string();
        let args = format!("version --source {} --bump-epoch {} --output-format {}", STDIN, bump_value, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2!1.2.3-alpha.1")]  // Epoch bump preserves other data
    fn test_bump_epoch_preserve_all_data(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let args = format!("version --source {} --bump-epoch --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}

mod post_bump {
    use super::*;

    #[rstest]
    #[case("1.2.3-alpha.1.post.3", ZERV)]  // Simple post bump
    fn test_bump_post_simple(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let input = base_zerv_fixture.build().to_string();
        let args = format!("version --source {} --output-format {} --bump-post", STDIN, format);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3-alpha.1.post.4", "2")]  // Custom post bump value
    #[case("1.2.3-alpha.1.post.6", "4")]  // Larger post bump value
    fn test_bump_post_with_value(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let input = base_zerv_fixture.build().to_string();
        let args = format!("version --source {} --bump-post {} --output-format {}", STDIN, bump_value, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2!1.2.3-beta.2.post.3")]  // Post bump with existing epoch and prerelease
    fn test_bump_post_with_existing_data(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let args = format!("version --source {} --bump-post --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3.post.3")]  // Post bump on stable version
    fn test_bump_post_on_stable_version(
        #[case] expected: &str,
    ) {
        // Create stable version (no prerelease)
        let fixture = ZervFixture::new().with_version(1, 2, 3);
        let input = fixture.build().to_string();
        let args = format!("version --source {} --bump-post --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}

mod dev_bump {
    use super::*;

    #[rstest]
    #[case("1.2.3-alpha.1.dev.3", ZERV)]  // Simple dev bump
    fn test_bump_dev_simple(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let input = base_zerv_fixture.build().to_string();
        let args = format!("version --source {} --output-format {} --bump-dev", STDIN, format);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3-alpha.1.dev.4", "2")]  // Custom dev bump value
    #[case("1.2.3-alpha.1.dev.6", "4")]  // Larger dev bump value
    fn test_bump_dev_with_value(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let input = base_zerv_fixture.build().to_string();
        let args = format!("version --source {} --bump-dev {} --output-format {}", STDIN, bump_value, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2!1.2.3-beta.2.dev.3")]  // Dev bump with existing epoch and prerelease
    fn test_bump_dev_with_existing_data(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let args = format!("version --source {} --bump-dev --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}

mod pre_release_num_bump {
    use super::*;

    #[rstest]
    #[case("1.2.3-alpha.2", ZERV)]  // Simple prerelease number bump
    fn test_bump_pre_release_num_simple(
        prerelease_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let input = prerelease_bump_fixture.build().to_string();
        let args = format!("version --source {} --output-format {} --bump-pre-release-num", STDIN, format);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3-alpha.3", "2")]  // Custom prerelease number bump value
    #[case("1.2.3-alpha.5", "4")]  // Larger prerelease number bump value
    fn test_bump_pre_release_num_with_value(
        prerelease_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let input = prerelease_bump_fixture.build().to_string();
        let args = format!("version --source {} --bump-pre-release-num {} --output-format {}", STDIN, bump_value, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2!1.2.3-beta.3")]  // Prerelease number bump with existing epoch
    fn test_bump_pre_release_num_with_existing_data(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let args = format!("version --source {} --bump-pre-release-num --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3-alpha.1")]  // Prerelease number bump on version without existing number
    fn test_bump_pre_release_num_without_existing_number(
        #[case] expected: &str,
    ) {
        // Create version with prerelease label but no number
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, None);
        let input = fixture.build().to_string();
        let args = format!("version --source {} --bump-pre-release-num --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}

mod pre_release_label_bump {
    use super::*;

    #[rstest]
    #[case("1.2.3-beta.0", ZERV)]  // alpha -> beta
    #[case("1.2.3-rc.0", ZERV)]    // alpha -> rc
    fn test_bump_pre_release_label_simple(
        prerelease_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let new_label = if expected.contains("beta") { "beta" } else { "rc" };

        let input = prerelease_bump_fixture.build().to_string();
        let args = format!("version --source {} --output-format {} --bump-pre-release-label {}", STDIN, format, new_label);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3-beta.0")]  // beta -> rc
    #[case("1.2.3-alpha.0")]  // rc -> alpha (backwards)
    fn test_bump_pre_release_label_transitions(
        #[case] expected: &str,
    ) {
        let (start_label, end_label) = if expected.contains("beta") {
            ("alpha", "beta")
        } else if expected.contains("rc") {
            ("beta", "rc")
        } else {
            ("rc", "alpha")
        };

        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(start_label, Some(1));
        let input = fixture.build().to_string();
        let args = format!("version --source {} --bump-pre-release-label {} --output-format {}", STDIN, end_label, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2!1.2.3-rc.0")]  // Label bump preserves existing epoch
    fn test_bump_pre_release_label_preserve_existing_data(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let args = format!("version --source {} --bump-pre-release-label rc --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3-alpha.0")]  // Label bump on stable version creates prerelease
    fn test_bump_pre_release_label_on_stable_version(
        #[case] expected: &str,
    ) {
        // Create stable version (no prerelease)
        let fixture = ZervFixture::new().with_version(1, 2, 3);
        let input = fixture.build().to_string();
        let args = format!("version --source {} --bump-pre-release-label alpha --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}

mod secondary_combinations {
    use super::*;

    #[rstest]
    #[case("2!1.2.3-alpha.2.dev.3")]  // epoch + prerelease num + dev
    #[case("2!1.2.3-beta.0.post.3")]  // epoch + label + post
    #[case("1.2.3-beta.0.dev.3")]    // label + dev
    #[case("2!1.2.3-beta.1.dev.3.post.3")]  // All secondary bumps
    fn test_multiple_secondary_bumps(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let args = match expected {
            s if s.contains("epoch") && s.contains("alpha.2") && s.contains("dev.3") => {
                format!("version --source {} --bump-epoch --bump-pre-release-num --bump-dev --output-format {}", STDIN, ZERV)
            }
            s if s.contains("epoch") && s.contains("beta.0") && s.contains("post.3") => {
                format!("version --source {} --bump-epoch --bump-pre-release-label beta --bump-post --output-format {}", STDIN, ZERV)
            }
            s if s.contains("beta.0") && s.contains("dev.3") && !s.contains("epoch") => {
                format!("version --source {} --bump-pre-release-label beta --bump-dev --output-format {}", STDIN, ZERV)
            }
            s if s.contains("epoch") && s.contains("beta.1") && s.contains("dev.3") && s.contains("post.3") => {
                format!("version --source {} --bump-epoch --bump-pre-release-label beta --bump-pre-release-num --bump-dev --bump-post --output-format {}", STDIN, ZERV)
            }
            _ => unreachable!("Unexpected expected value pattern"),
        };

        let output = TestCommand::run_with_stdin(&args, input);
        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3!1.2.3-beta.2", "2", "1")]  // epoch=2 + prerelease num=1
    fn test_secondary_bumps_with_custom_values(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] epoch_value: &str,
        #[case] prerelease_value: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let args = format!("version --source {} --bump-epoch {} --bump-pre-release-num {} --output-format {}", STDIN, epoch_value, prerelease_value, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2!1.2.3-beta.0.post.3.dev.4")]  // Complex combination preserves order
    fn test_complex_secondary_bump_order(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let args = format!("version --source {} --bump-epoch --bump-pre-release-label beta --bump-post --bump-dev --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }
}
