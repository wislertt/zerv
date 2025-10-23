use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

#[fixture]
fn base_fixture() -> ZervFixture {
    ZervFixture::new().with_version(1, 2, 3)
}

mod epoch_override {
    use super::*;

    #[rstest]
    #[case::one(1, "1!1.2.3")]
    #[case::large(5, "5!1.2.3")]
    fn test_epoch_override_basic(
        base_fixture: ZervFixture,
        #[case] epoch: u32,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --epoch {epoch} --output-format pep440"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_epoch_override_with_zerv_format(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --epoch 2 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("epoch: Some(2)"),
            "Expected epoch: Some(2) in output: {}",
            output
        );
    }

    #[rstest]
    fn test_epoch_zero_normalized(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --epoch 0 --output-format pep440",
            zerv_ron,
        );

        assert_eq!(output, "1.2.3", "Epoch 0 should be normalized away");
    }
}

mod post_override {
    use super::*;

    #[rstest]
    #[case::one(1)]
    #[case::large(99)]
    fn test_post_override_sets_var(base_fixture: ZervFixture, #[case] post: u32) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --post {post} --output-format zerv"),
            zerv_ron,
        );

        assert!(
            output.contains(&format!("post: Some({post})")),
            "Expected post: Some({post}) in vars output: {}",
            output
        );
    }

    #[rstest]
    fn test_post_zero_sets_var(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --post 0 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("post: Some(0)"),
            "Expected post: Some(0) in vars output: {}",
            output
        );
    }
}

mod dev_override {
    use super::*;

    #[rstest]
    #[case::one(1)]
    #[case::large(99)]
    fn test_dev_override_sets_var(base_fixture: ZervFixture, #[case] dev: u32) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --dev {dev} --output-format zerv"),
            zerv_ron,
        );

        assert!(
            output.contains(&format!("dev: Some({dev})")),
            "Expected dev: Some({dev}) in vars output: {}",
            output
        );
    }

    #[rstest]
    fn test_dev_zero_sets_var(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --dev 0 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("dev: Some(0)"),
            "Expected dev: Some(0) in vars output: {}",
            output
        );
    }
}

mod pre_release_label_override {
    use super::*;

    #[rstest]
    #[case::alpha("alpha", "1.2.3-alpha.0")]
    #[case::beta("beta", "1.2.3-beta.0")]
    #[case::rc("rc", "1.2.3-rc.0")]
    fn test_pre_release_label_override_semver(
        base_fixture: ZervFixture,
        #[case] label: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --pre-release-label {label} --output-format semver"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    #[case::alpha("alpha", "1.2.3a0")]
    #[case::beta("beta", "1.2.3b0")]
    #[case::rc("rc", "1.2.3rc0")]
    fn test_pre_release_label_override_pep440(
        base_fixture: ZervFixture,
        #[case] label: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --pre-release-label {label} --output-format pep440"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_pre_release_label_override_with_zerv_format(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --pre-release-label beta --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("label: Beta"),
            "Expected label: Beta in output: {}",
            output
        );
    }

    #[rstest]
    fn test_pre_release_label_replaces_existing(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture
            .with_pre_release(PreReleaseLabel::Alpha, Some(5))
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --pre-release-label rc --output-format semver",
            zerv_ron,
        );

        assert_eq!(
            output, "1.2.3-rc.5",
            "Label should replace existing, preserving number"
        );
    }
}

mod pre_release_num_override {
    use super::*;

    #[rstest]
    #[case::zero(0, "1.2.3-alpha.0")]
    #[case::one(1, "1.2.3-alpha.1")]
    #[case::large(99, "1.2.3-alpha.99")]
    fn test_pre_release_num_override_basic(
        base_fixture: ZervFixture,
        #[case] num: u32,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --pre-release-num {num} --output-format semver"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_pre_release_num_override_with_zerv_format(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --pre-release-num 10 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("number: Some(10)"),
            "Expected number: Some(10) in output: {}",
            output
        );
    }

    #[rstest]
    fn test_pre_release_num_replaces_existing(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture
            .with_pre_release(PreReleaseLabel::Rc, Some(5))
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --pre-release-num 3 --output-format semver",
            zerv_ron,
        );

        assert_eq!(
            output, "1.2.3-rc.3",
            "Number should replace existing, preserving label"
        );
    }
}

mod secondary_component_combinations {
    use super::*;

    #[rstest]
    fn test_epoch_with_vars(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --epoch 2 --post 5 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("epoch: Some(2)") && output.contains("post: Some(5)"),
            "Expected both epoch and post in vars: {}",
            output
        );
    }

    #[rstest]
    fn test_pre_release_label_and_num(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --pre-release-label beta --pre-release-num 7 \
             --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "1.2.3-beta.7");
    }

    #[rstest]
    fn test_multiple_secondary_vars(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --epoch 1 --post 2 --dev 3 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("epoch: Some(1)")
                && output.contains("post: Some(2)")
                && output.contains("dev: Some(3)"),
            "Expected all secondary vars set: {}",
            output
        );
    }
}

mod secondary_with_primary_components {
    use super::*;

    #[rstest]
    fn test_primary_and_epoch(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 2 --minor 5 --patch 1 --epoch 3 --output-format pep440",
            zerv_ron,
        );

        assert_eq!(output, "3!2.5.1");
    }

    #[rstest]
    fn test_primary_and_post_in_vars(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 3 --minor 0 --patch 0 --post 1 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("major: Some(3)")
                && output.contains("minor: Some(0)")
                && output.contains("patch: Some(0)")
                && output.contains("post: Some(1)"),
            "Expected all components in vars: {}",
            output
        );
    }

    #[rstest]
    fn test_primary_and_prerelease(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 2 --minor 0 --patch 0 \
             --pre-release-label beta --pre-release-num 3 --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "2.0.0-beta.3");
    }
}

mod secondary_with_vcs_data {
    use super::*;

    #[fixture]
    fn vcs_fixture() -> ZervFixture {
        ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            Some(5),
            Some(true),
            None,
            None,
            None,
            None,
            Some("main".to_string()),
        )
    }

    #[rstest]
    fn test_epoch_preserves_vcs_data(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --epoch 2 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("epoch: Some(2)")
                && output.contains("distance: Some(5)")
                && output.contains("dirty: Some(true)"),
            "Expected epoch override with preserved VCS data in output: {}",
            output
        );
    }

    #[rstest]
    fn test_post_preserves_vcs_data(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --post 3 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("post: Some(3)")
                && output.contains("distance: Some(5)")
                && output.contains("dirty: Some(true)"),
            "Expected post override with preserved VCS data in output: {}",
            output
        );
    }

    #[rstest]
    fn test_dev_preserves_vcs_data(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --dev 4 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("dev: Some(4)")
                && output.contains("distance: Some(5)")
                && output.contains("dirty: Some(true)"),
            "Expected dev override with preserved VCS data in output: {}",
            output
        );
    }

    #[rstest]
    fn test_prerelease_preserves_vcs_data(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --pre-release-label alpha --pre-release-num 2 \
             --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("label: Alpha")
                && output.contains("number: Some(2)")
                && output.contains("distance: Some(5)")
                && output.contains("dirty: Some(true)"),
            "Expected prerelease override with preserved VCS data in output: {}",
            output
        );
    }
}
