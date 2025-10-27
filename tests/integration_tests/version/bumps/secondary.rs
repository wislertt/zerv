//! Secondary component bump tests
//!
//! Tests for --bump-epoch, --bump-post, --bump-dev, --bump-pre-release-num,
//! and --bump-pre-release-label options. These tests verify that secondary
//! component bumps work correctly and interact properly with other version data.

use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;
use zerv::version::PreReleaseLabel;

use super::base_zerv_fixture;
use crate::util::TestCommand;

/// Zerv fixture with secondary components for bump tests
#[fixture]
fn secondary_bump_fixture() -> ZervFixture {
    base_zerv_fixture().with_epoch(1).with_post(2).with_dev(3)
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
    secondary_bump_fixture().with_pre_release(PreReleaseLabel::Beta, Some(2))
}

mod epoch_bump {
    use super::*;

    #[rstest]
    fn test_bump_epoch_simple(base_zerv_fixture: ZervFixture) {
        let input = base_zerv_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --output-format pep440 --bump-epoch",
            input,
        );

        assert_eq!(output.trim(), "1!0.0.0");
    }

    #[rstest]
    #[case("2!0.0.0", "2")]
    #[case("4!0.0.0", "4")]
    fn test_bump_epoch_with_value(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let input = base_zerv_fixture.build().to_string();
        let args = format!(
            "version --source stdin --bump-epoch {} --output-format pep440",
            bump_value
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    fn test_bump_epoch_preserve_secondary_data(full_secondary_fixture: ZervFixture) {
        let input = full_secondary_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-epoch --output-format pep440",
            input,
        );

        assert_eq!(output.trim(), "2!0.0.0");
    }
}

mod post_bump {
    use super::*;

    #[rstest]
    fn test_bump_post_simple(base_zerv_fixture: ZervFixture) {
        let input = base_zerv_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --output-format pep440 --bump-post",
            input,
        );

        assert_eq!(output.trim(), "1.2.3a1.post1");
    }

    #[rstest]
    #[case("1.2.3a1.post2", "2")]
    #[case("1.2.3a1.post4", "4")]
    fn test_bump_post_with_value(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let input = base_zerv_fixture.build().to_string();
        let args = format!(
            "version --source stdin --bump-post {} --output-format pep440",
            bump_value
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    fn test_bump_post_with_existing_data(full_secondary_fixture: ZervFixture) {
        let input = full_secondary_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-post --output-format pep440",
            input,
        );

        assert_eq!(output.trim(), "1!1.2.3b2.post3");
    }

    #[rstest]
    fn test_bump_post_on_stable_version() {
        let fixture = ZervFixture::new().with_version(1, 2, 3);
        let input = fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-post --output-format pep440",
            input,
        );

        assert_eq!(output.trim(), "1.2.3.post1");
    }
}

mod dev_bump {
    use super::*;

    #[rstest]
    fn test_bump_dev_simple(base_zerv_fixture: ZervFixture) {
        let input = base_zerv_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --output-format zerv --bump-dev",
            input,
        );

        assert!(
            output.contains("dev: Some(1)"),
            "Expected dev: Some(1) in output: {}",
            output
        );
    }

    #[rstest]
    #[case(2, "2")]
    #[case(4, "4")]
    fn test_bump_dev_with_value(
        base_zerv_fixture: ZervFixture,
        #[case] expected_dev: u64,
        #[case] bump_value: &str,
    ) {
        let input = base_zerv_fixture.build().to_string();
        let args = format!(
            "version --source stdin --bump-dev {} --output-format zerv",
            bump_value
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert!(
            output.contains(&format!("dev: Some({})", expected_dev)),
            "Expected dev: Some({}) in output: {}",
            expected_dev,
            output
        );
    }

    #[rstest]
    fn test_bump_dev_with_existing_data(full_secondary_fixture: ZervFixture) {
        let input = full_secondary_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-dev --output-format zerv",
            input,
        );

        assert!(
            output.contains("dev: Some(4)"),
            "Expected dev: Some(4) in output: {}",
            output
        );
    }
}

mod pre_release_num_bump {
    use super::*;

    #[rstest]
    fn test_bump_pre_release_num_simple(prerelease_bump_fixture: ZervFixture) {
        let input = prerelease_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --output-format pep440 --bump-pre-release-num",
            input,
        );

        assert_eq!(output.trim(), "1.2.3a2");
    }

    #[rstest]
    #[case("1.2.3a3", "2")]
    #[case("1.2.3a5", "4")]
    fn test_bump_pre_release_num_with_value(
        prerelease_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_value: &str,
    ) {
        let input = prerelease_bump_fixture.build().to_string();
        let args = format!(
            "version --source stdin --bump-pre-release-num {} --output-format pep440",
            bump_value
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    fn test_bump_pre_release_num_with_existing_data(full_secondary_fixture: ZervFixture) {
        let input = full_secondary_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-pre-release-num --output-format pep440",
            input,
        );

        assert_eq!(output.trim(), "1!1.2.3b3");
    }

    #[rstest]
    fn test_bump_pre_release_num_without_existing_number() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, None);
        let input = fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-pre-release-num --output-format pep440",
            input,
        );

        assert_eq!(output.trim(), "1.2.3a1");
    }
}

mod pre_release_label_bump {
    use super::*;

    #[rstest]
    #[case("1.2.3b0", "beta")]
    #[case("1.2.3rc0", "rc")]
    fn test_bump_pre_release_label_simple(
        prerelease_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] new_label: &str,
    ) {
        let input = prerelease_bump_fixture.build().to_string();
        let args = format!(
            "version --source stdin --output-format pep440 --bump-pre-release-label {}",
            new_label
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3b0", PreReleaseLabel::Alpha, "beta")]
    #[case("1.2.3rc0", PreReleaseLabel::Beta, "rc")]
    #[case("1.2.3a0", PreReleaseLabel::Rc, "alpha")]
    fn test_bump_pre_release_label_transitions(
        #[case] expected: &str,
        #[case] start_label: PreReleaseLabel,
        #[case] end_label: &str,
    ) {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(start_label, Some(1));
        let input = fixture.build().to_string();
        let args = format!(
            "version --source stdin --bump-pre-release-label {} --output-format pep440",
            end_label
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    fn test_bump_pre_release_label_preserve_existing_data(full_secondary_fixture: ZervFixture) {
        let input = full_secondary_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-pre-release-label rc --output-format pep440",
            input,
        );

        assert_eq!(output.trim(), "1!1.2.3rc0");
    }

    #[rstest]
    fn test_bump_pre_release_label_on_stable_version() {
        let fixture = ZervFixture::new().with_version(1, 2, 3);
        let input = fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-pre-release-label alpha --output-format pep440",
            input,
        );

        assert_eq!(output.trim(), "1.2.3a0");
    }
}

mod secondary_combinations {
    use super::*;

    #[rstest]
    #[case(
        "2!0.0.0a1",
        "version --source stdin --bump-epoch --bump-pre-release-num --bump-dev --output-format pep440"
    )]
    #[case(
        "2!0.0.0rc0.post1",
        "version --source stdin --bump-epoch --bump-pre-release-label rc --bump-post --output-format pep440"
    )]
    #[case(
        "dev: Some(1)",
        "version --source stdin --bump-pre-release-label rc --bump-dev --output-format zerv"
    )]
    #[case(
        "2!0.0.0rc0.post1",
        "version --source stdin --bump-epoch --bump-pre-release-label rc --bump-post --bump-dev --output-format pep440"
    )]
    fn test_multiple_secondary_bumps(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] args: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(args, input);

        if args.contains("zerv") {
            assert!(
                output.contains(expected),
                "Expected '{}' in output: {}",
                expected,
                output
            );
        } else {
            assert_eq!(output.trim(), expected);
        }
    }

    #[rstest]
    #[case("3!0.0.0", "2", "0")]
    fn test_secondary_bumps_with_custom_values(
        full_secondary_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] epoch_value: &str,
        #[case] prerelease_value: &str,
    ) {
        let input = full_secondary_fixture.build().to_string();
        let args = format!(
            "version --source stdin --bump-epoch {} --bump-pre-release-num {} --output-format pep440",
            epoch_value, prerelease_value
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    fn test_complex_secondary_bump_order(full_secondary_fixture: ZervFixture) {
        let input = full_secondary_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-epoch --bump-pre-release-label rc --bump-post --bump-dev --output-format pep440",
            input,
        );

        assert_eq!(output.trim(), "2!0.0.0rc0.post1");
    }
}
