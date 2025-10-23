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

mod major_override {
    use super::*;

    #[rstest]
    #[case::zero(0, "0.2.3")]
    #[case::same(1, "1.2.3")]
    #[case::increment(2, "2.2.3")]
    #[case::large(99, "99.2.3")]
    fn test_major_override_basic(
        base_fixture: ZervFixture,
        #[case] major: u32,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --major {major} --output-format semver"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_major_override_with_pep440(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 5 --output-format pep440",
            zerv_ron,
        );

        assert_eq!(output, "5.2.3");
    }

    #[rstest]
    fn test_major_override_with_zerv_format(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 10 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("major: Some(10)"),
            "Expected major: Some(10) in output: {}",
            output
        );
    }

    #[rstest]
    fn test_major_override_replaces_stdin_value(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 7 --output-format semver",
            zerv_ron,
        );

        assert_eq!(
            output, "7.2.3",
            "Expected major override to replace stdin value 1"
        );
    }
}

mod minor_override {
    use super::*;

    #[rstest]
    #[case::zero(0, "1.0.3")]
    #[case::same(2, "1.2.3")]
    #[case::increment(3, "1.3.3")]
    #[case::large(99, "1.99.3")]
    fn test_minor_override_basic(
        base_fixture: ZervFixture,
        #[case] minor: u32,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --minor {minor} --output-format semver"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_minor_override_with_pep440(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --minor 8 --output-format pep440",
            zerv_ron,
        );

        assert_eq!(output, "1.8.3");
    }

    #[rstest]
    fn test_minor_override_with_zerv_format(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --minor 15 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("minor: Some(15)"),
            "Expected minor: Some(15) in output: {}",
            output
        );
    }

    #[rstest]
    fn test_minor_override_replaces_stdin_value(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --minor 20 --output-format semver",
            zerv_ron,
        );

        assert_eq!(
            output, "1.20.3",
            "Expected minor override to replace stdin value 2"
        );
    }
}

mod patch_override {
    use super::*;

    #[rstest]
    #[case::zero(0, "1.2.0")]
    #[case::same(3, "1.2.3")]
    #[case::increment(4, "1.2.4")]
    #[case::large(99, "1.2.99")]
    fn test_patch_override_basic(
        base_fixture: ZervFixture,
        #[case] patch: u32,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --patch {patch} --output-format semver"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_patch_override_with_pep440(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --patch 12 --output-format pep440",
            zerv_ron,
        );

        assert_eq!(output, "1.2.12");
    }

    #[rstest]
    fn test_patch_override_with_zerv_format(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --patch 42 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("patch: Some(42)"),
            "Expected patch: Some(42) in output: {}",
            output
        );
    }

    #[rstest]
    fn test_patch_override_replaces_stdin_value(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --patch 99 --output-format semver",
            zerv_ron,
        );

        assert_eq!(
            output, "1.2.99",
            "Expected patch override to replace stdin value 3"
        );
    }
}

mod component_combinations {
    use super::*;

    #[rstest]
    fn test_major_and_minor_override(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 5 --minor 10 --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "5.10.3");
    }

    #[rstest]
    fn test_major_and_patch_override(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 3 --patch 7 --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "3.2.7");
    }

    #[rstest]
    fn test_minor_and_patch_override(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --minor 4 --patch 8 --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "1.4.8");
    }

    #[rstest]
    fn test_all_three_components_override(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 10 --minor 20 --patch 30 --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "10.20.30");
    }

    #[rstest]
    fn test_all_components_with_pep440(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 2 --minor 3 --patch 4 --output-format pep440",
            zerv_ron,
        );

        assert_eq!(output, "2.3.4");
    }

    #[rstest]
    fn test_all_components_with_zerv_format(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 6 --minor 7 --patch 8 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("major: Some(6)")
                && output.contains("minor: Some(7)")
                && output.contains("patch: Some(8)"),
            "Expected all components overridden in output: {}",
            output
        );
    }
}

mod component_with_prerelease {
    use super::*;

    #[fixture]
    fn prerelease_fixture() -> ZervFixture {
        ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
    }

    #[rstest]
    fn test_major_override_preserves_prerelease(prerelease_fixture: ZervFixture) {
        let zerv_ron = prerelease_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 5 --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "5.2.3-alpha.1");
    }

    #[rstest]
    fn test_minor_override_preserves_prerelease(prerelease_fixture: ZervFixture) {
        let zerv_ron = prerelease_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --minor 10 --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "1.10.3-alpha.1");
    }

    #[rstest]
    fn test_patch_override_preserves_prerelease(prerelease_fixture: ZervFixture) {
        let zerv_ron = prerelease_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --patch 15 --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "1.2.15-alpha.1");
    }

    #[rstest]
    fn test_all_components_preserve_prerelease(prerelease_fixture: ZervFixture) {
        let zerv_ron = prerelease_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 2 --minor 3 --patch 4 --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "2.3.4-alpha.1");
    }
}

mod component_with_vcs_data {
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
    fn test_major_override_preserves_vcs_data(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 10 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("major: Some(10)")
                && output.contains("distance: Some(5)")
                && output.contains("dirty: Some(true)"),
            "Expected major override with preserved VCS data in output: {}",
            output
        );
    }

    #[rstest]
    fn test_minor_override_preserves_vcs_data(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --minor 20 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("minor: Some(20)")
                && output.contains("distance: Some(5)")
                && output.contains("dirty: Some(true)"),
            "Expected minor override with preserved VCS data in output: {}",
            output
        );
    }

    #[rstest]
    fn test_patch_override_preserves_vcs_data(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --patch 30 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("patch: Some(30)")
                && output.contains("distance: Some(5)")
                && output.contains("dirty: Some(true)"),
            "Expected patch override with preserved VCS data in output: {}",
            output
        );
    }
}
