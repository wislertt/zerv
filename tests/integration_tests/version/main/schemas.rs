use rstest::rstest;
use zerv::test_utils::{
    ZervFixture,
    ZervSchemaFixture,
};
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

// Test constants
const DEV_TIMESTAMP: u64 = 1234567890;
const TEST_BRANCH: &str = "feature.branch";
const TEST_COMMIT_HASH: &str = "abc123def456";
const DIRTY_BRANCH: &str = "main.branch";
const DIRTY_COMMIT_HASH: &str = "def456abc789";

// Helper functions

/// Creates a ZervFixture with standard tier 1 setup
fn create_standard_tier_1_fixture(version: (u64, u64, u64)) -> ZervFixture {
    ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .with_standard_tier_1()
}

/// Creates a ZervFixture with standard tier 2 setup and distance
fn create_standard_tier_2_fixture(version: (u64, u64, u64), distance: u64) -> ZervFixture {
    ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .with_standard_tier_2()
        .with_vcs_data(Some(distance), Some(false), None, None, None, None, None)
}

/// Creates a ZervFixture with standard tier 3 setup and deterministic git data
fn create_standard_tier_3_fixture(version: (u64, u64, u64), distance: u64) -> ZervFixture {
    ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .with_standard_tier_3()
        .with_vcs_data(
            Some(distance),
            Some(true),
            Some(TEST_BRANCH.to_string()),
            Some(TEST_COMMIT_HASH.to_string()),
            None,
            None,
            None,
        )
        .with_dev(DEV_TIMESTAMP)
}

/// Creates a dirty ZervFixture for tier selection tests
fn create_dirty_tier_3_fixture(version: (u64, u64, u64)) -> ZervFixture {
    ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .with_standard_tier_3()
        .with_vcs_data(
            Some(0),
            Some(true),
            Some(DIRTY_BRANCH.to_string()),
            Some(DIRTY_COMMIT_HASH.to_string()),
            None,
            None,
            None,
        )
        .with_dev(DEV_TIMESTAMP)
}

/// Helper functions for different schema setups
fn setup_schema_major_minor() -> ZervSchemaFixture {
    ZervSchemaFixture::empty().with_major_minor()
}

fn setup_schema_with_epoch() -> ZervSchemaFixture {
    ZervSchemaFixture::empty()
        .with_major_minor_patch()
        .with_epoch_in_extra_core()
}

fn setup_schema_with_prerelease() -> ZervSchemaFixture {
    ZervSchemaFixture::empty()
        .with_major_minor_patch()
        .with_prerelease_in_extra_core()
}

mod schema_preset_standard {
    use super::*;

    #[rstest]
    #[case::tier_1_clean((1, 2, 3), None, None, "1.2.3")]
    #[case::tier_1_with_epoch((1, 0, 0), Some(2), None, "1.0.0-epoch.2")]
    #[case::tier_1_with_post((2, 5, 1), None, Some(3), "2.5.1-post.3")]
    fn test_schema_standard_tier_1(
        #[case] version: (u64, u64, u64),
        #[case] epoch: Option<u64>,
        #[case] post: Option<u64>,
        #[case] expected: &str,
    ) {
        let mut fixture = create_standard_tier_1_fixture(version);

        if let Some(e) = epoch {
            fixture = fixture.with_epoch(e);
        }

        if let Some(p) = post {
            fixture = fixture.with_post(p);
        }

        let zerv_ron = fixture.build().to_string();
        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(output, expected);
    }

    #[rstest]
    #[case::tier_2_with_distance((1, 0, 0), 5, "1.0.0+5")]
    #[case::tier_2_with_branch_metadata((2, 3, 1), 10, "2.3.1+10")]
    fn test_schema_standard_tier_2(
        #[case] version: (u64, u64, u64),
        #[case] distance: u64,
        #[case] expected: &str,
    ) {
        let zerv_ron = create_standard_tier_2_fixture(version, distance)
            .build()
            .to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(output, expected);
    }

    #[rstest]
    #[case::tier_3_dirty((1, 0, 0), "1.0.0-dev.1234567890+feature.branch.5.abc123d")]
    #[case::tier_3_with_distance((2, 1, 0), "2.1.0-dev.1234567890+feature.branch.5.abc123d")]
    fn test_schema_standard_tier_3(#[case] version: (u64, u64, u64), #[case] expected: &str) {
        let zerv_ron = create_standard_tier_3_fixture(version, 5)
            .build()
            .to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(output, expected);
    }
}

mod schema_preset_calver {
    use super::*;

    #[test]
    fn test_schema_calver_accepted() {
        let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-calver")
            .stdin(zerv_ron)
            .assert_success();

        assert!(
            !output.stdout().trim().is_empty(),
            "CalVer schema should produce output"
        );
    }

    #[test]
    fn test_schema_calver_with_distance() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_vcs_data(Some(3), Some(true), None, None, None, None, None)
            .build()
            .to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-calver")
            .stdin(zerv_ron)
            .assert_success();

        assert_eq!(
            output.stdout().trim(),
            "2025.10.22-0+3",
            "CalVer tier 2 should match expected format"
        );
    }

    #[test]
    fn test_schema_calver_with_dirty() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_vcs_data(Some(0), Some(true), None, None, None, None, None)
            .with_dev(1234567890)
            .build()
            .to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-calver")
            .stdin(zerv_ron)
            .assert_success();

        assert!(
            output.stdout().contains("dev"),
            "CalVer tier 3 should include dev notation when dirty, got: {}",
            output.stdout().trim()
        );
    }
}

mod schema_defaults {
    use super::*;

    #[test]
    fn test_schema_default_is_standard() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_standard_tier_1()
            .build()
            .to_string();

        let output_explicit = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-standard")
            .stdin(zerv_ron.clone())
            .assert_success();

        let output_default = TestCommand::new()
            .args_from_str("version --source stdin")
            .stdin(zerv_ron)
            .assert_success();

        assert_eq!(
            output_explicit.stdout().trim(),
            output_default.stdout().trim(),
            "Default schema should be zerv-standard"
        );
    }
}

mod schema_ron_custom {
    use super::*;

    #[test]
    fn test_schema_ron_basic() {
        let zerv_ron = ZervFixture::new().with_version(3, 2, 1).build().to_string();
        let schema_ron = ZervSchemaFixture::standard_tier_1().build().to_string();

        let args = format!("version --source stdin --schema-ron '{schema_ron}'");
        let output = TestCommand::run_with_stdin(&args, zerv_ron);

        assert_eq!(output, "3.2.1");
    }

    #[rstest]
    #[case::major_minor(
        (1, 2, 3),
        setup_schema_major_minor(),
        ZervFixture::new(),
        "version --source stdin --schema-ron",
        "1.2.0")]
    #[case::epoch_in_extra_core(
        (1, 2, 3),
        setup_schema_with_epoch(),
        ZervFixture::new().with_epoch(5),
        "version --source stdin --output-format pep440 --schema-ron",
        "5!1.2.3")]
    #[case::prerelease_in_extra_core(
        (2, 0, 0),
        setup_schema_with_prerelease(),
        ZervFixture::new().with_pre_release(PreReleaseLabel::Beta, Some(3)),
        "version --source stdin --output-format semver --schema-ron",
        "2.0.0-beta.3")]
    fn test_schema_ron_custom_components(
        #[case] version: (u64, u64, u64),
        #[case] schema_fixture: ZervSchemaFixture,
        #[case] zerv_fixture: ZervFixture,
        #[case] args: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = zerv_fixture
            .with_version(version.0, version.1, version.2)
            .build()
            .to_string();
        let custom_schema = schema_fixture.build().to_string();
        let full_args = format!("{} '{}'", args, custom_schema);

        let output = TestCommand::run_with_stdin(&full_args, zerv_ron);

        assert_eq!(output, expected);
    }
}

mod schema_validation {
    use super::*;

    #[test]
    fn test_schema_unknown_preset_error() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema invalid-schema")
            .stdin(zerv_ron)
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("Unknown schema") || stderr.contains("invalid-schema"),
            "Should show unknown schema error, got: {stderr}"
        );
    }

    #[test]
    fn test_schema_and_schema_ron_conflict() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let schema_ron = ZervSchemaFixture::standard_tier_1().build().to_string();

        let output = TestCommand::new()
            .args_from_str(format!(
                "version --source stdin --schema zerv-standard --schema-ron '{schema_ron}'"
            ))
            .stdin(zerv_ron)
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("Conflicting") || stderr.contains("both"),
            "Should show conflict error when both schema and schema-ron are specified, got: {stderr}"
        );
    }

    #[test]
    fn test_schema_ron_invalid_syntax() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let invalid_schema = r#"(core:[var(Invalid),extra_core:[]"#;

        let output = TestCommand::new()
            .args_from_str(format!(
                "version --source stdin --schema-ron '{invalid_schema}'"
            ))
            .stdin(zerv_ron)
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("parse") || stderr.contains("invalid") || stderr.contains("RON"),
            "Should show RON parse error, got: {stderr}"
        );
    }
}

mod schema_output_formats {
    use super::*;

    #[rstest]
    #[case::standard_semver("zerv-standard", "semver", "1.2.3")]
    #[case::standard_pep440("zerv-standard", "pep440", "1.2.3")]
    fn test_schema_with_output_format(
        #[case] schema: &str,
        #[case] output_format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();

        let output = TestCommand::new()
            .args_from_str(format!(
                "version --source stdin --schema {schema} --output-format {output_format}"
            ))
            .stdin(zerv_ron)
            .assert_success();

        assert_eq!(output.stdout().trim(), expected);
    }

    #[test]
    fn test_schema_calver_with_output_formats() {
        let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();

        let output_semver = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-calver --output-format semver")
            .stdin(zerv_ron.clone())
            .assert_success();

        let output_pep440 = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-calver --output-format pep440")
            .stdin(zerv_ron)
            .assert_success();

        assert!(
            !output_semver.stdout().trim().is_empty(),
            "CalVer with semver format should produce output"
        );
        assert!(
            !output_pep440.stdout().trim().is_empty(),
            "CalVer with pep440 format should produce output"
        );
    }

    #[test]
    fn test_schema_with_zerv_output_format_preserves_structure() {
        let original_zerv = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_standard_tier_1()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .build();

        let zerv_ron = original_zerv.to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-standard --output-format zerv")
            .stdin(zerv_ron)
            .assert_success();

        let parsed_zerv: zerv::version::Zerv =
            ron::from_str(output.stdout().trim()).expect("Failed to parse output as Zerv RON");

        assert_eq!(
            parsed_zerv.schema, original_zerv.schema,
            "Schema should be preserved through zerv output format"
        );
    }
}

mod schema_prerelease {
    use super::*;

    #[rstest]
    #[case::alpha(PreReleaseLabel::Alpha, Some(1), "1.0.0-alpha.1")]
    #[case::beta(PreReleaseLabel::Beta, Some(2), "1.0.0-beta.2")]
    #[case::rc(PreReleaseLabel::Rc, Some(3), "1.0.0-rc.3")]
    fn test_schema_standard_with_prerelease(
        #[case] label: PreReleaseLabel,
        #[case] number: Option<u64>,
        #[case] expected: &str,
    ) {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_pre_release(label, number)
            .build()
            .to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-standard")
            .stdin(zerv_ron)
            .assert_success();

        assert_eq!(output.stdout().trim(), expected);
    }

    #[test]
    fn test_schema_calver_with_prerelease() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .build()
            .to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-calver")
            .stdin(zerv_ron)
            .assert_success();

        assert!(
            output.stdout().contains("alpha"),
            "CalVer with alpha prerelease should include 'alpha' in output, got: {}",
            output.stdout().trim()
        );
    }
}

mod schema_tier_behavior {
    use super::*;

    #[test]
    fn test_schema_standard_tier_selection_tagged_clean() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_standard_tier_1()
            .with_vcs_data(Some(0), Some(false), None, None, None, None, None)
            .build()
            .to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-standard")
            .stdin(zerv_ron)
            .assert_success();

        assert_eq!(output.stdout().trim(), "1.2.3");
    }

    #[test]
    fn test_schema_standard_tier_selection_distance_clean() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_standard_tier_2()
            .with_vcs_data(Some(5), Some(false), None, None, None, None, None)
            .build()
            .to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-standard")
            .stdin(zerv_ron)
            .assert_success();

        assert_eq!(output.stdout().trim(), "1.2.3+5");
    }

    #[test]
    fn test_schema_standard_tier_selection_dirty() {
        let zerv_ron = create_dirty_tier_3_fixture((1, 2, 3)).build().to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(
            output, "1.2.3-dev.1234567890+main.branch.0.def456a",
            "Dirty state should trigger tier 3 with dev component"
        );
    }
}

mod schema_consistency {
    use super::*;

    #[test]
    fn test_schema_applied_consistently_across_commands() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_standard_tier_1()
            .build()
            .to_string();

        let output1 = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-standard")
            .stdin(zerv_ron.clone())
            .assert_success();

        let output2 = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-standard")
            .stdin(zerv_ron)
            .assert_success();

        assert_eq!(
            output1.stdout().trim(),
            output2.stdout().trim(),
            "Same schema should produce identical output for same input"
        );
    }

    #[test]
    fn test_different_schemas_can_be_applied() {
        let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();

        let output_standard = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-standard")
            .stdin(zerv_ron.clone())
            .assert_success();

        let output_calver = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-calver")
            .stdin(zerv_ron)
            .assert_success();

        assert!(
            !output_standard.stdout().trim().is_empty(),
            "Standard schema should produce output"
        );
        assert!(
            !output_calver.stdout().trim().is_empty(),
            "CalVer schema should produce output"
        );
    }
}
