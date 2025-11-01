use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::{
    ZervFixture,
    ZervSchemaFixture,
};
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

const DEV_TIMESTAMP: u64 = 1234567890;
const TEST_BRANCH: &str = "feature.branch";
const TEST_COMMIT_HASH: &str = "abc123def456";
const DIRTY_BRANCH: &str = "main.branch";
const DIRTY_COMMIT_HASH: &str = "def456abc789";

#[fixture]
fn tier_1_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 0, 0)
        .with_standard_tier_1()
}

#[fixture]
fn tier_2_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 0, 0)
        .with_standard_tier_2()
        .with_vcs_data(Some(5), Some(false), None, None, None, None, None)
}

#[fixture]
fn tier_3_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 0, 0)
        .with_standard_tier_3()
        .with_vcs_data(
            Some(5),
            Some(true),
            Some(TEST_BRANCH.to_string()),
            Some(TEST_COMMIT_HASH.to_string()),
            None,
            None,
            None,
        )
        .with_dev(DEV_TIMESTAMP)
}

#[fixture]
fn dirty_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
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

mod schema_preset_standard {
    //! Tests for the built-in zerv-standard schema preset
    use super::*;

    #[rstest]
    #[case::tier_1_clean((1, 2, 3), None, None, "1.2.3")]
    #[case::tier_1_with_epoch((1, 0, 0), Some(2), None, "1.0.0-epoch.2")]
    #[case::tier_1_with_post((2, 5, 1), None, Some(3), "2.5.1-post.3")]
    fn test_tier_1(
        tier_1_fixture: ZervFixture,
        #[case] version: (u64, u64, u64),
        #[case] epoch: Option<u64>,
        #[case] post: Option<u64>,
        #[case] expected: &str,
    ) {
        let mut fixture = tier_1_fixture.with_version(version.0, version.1, version.2);

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
    #[case::with_distance((1, 0, 0), 5, "1.0.0+5")]
    #[case::with_branch_metadata((2, 3, 1), 10, "2.3.1+10")]
    fn test_tier_2(
        tier_2_fixture: ZervFixture,
        #[case] version: (u64, u64, u64),
        #[case] distance: u64,
        #[case] expected: &str,
    ) {
        let zerv_ron = tier_2_fixture
            .with_version(version.0, version.1, version.2)
            .with_vcs_data(Some(distance), Some(false), None, None, None, None, None)
            .build()
            .to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(output, expected);
    }

    #[rstest]
    #[case::dirty((1, 0, 0), "1.0.0-dev.1234567890+feature.branch.5.abc123d")]
    #[case::with_distance((2, 1, 0), "2.1.0-dev.1234567890+feature.branch.5.abc123d")]
    fn test_tier_3(
        tier_3_fixture: ZervFixture,
        #[case] version: (u64, u64, u64),
        #[case] expected: &str,
    ) {
        let zerv_ron = tier_3_fixture
            .with_version(version.0, version.1, version.2)
            .build()
            .to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(output, expected);
    }
}

mod schema_preset_calver {
    //! Tests for the built-in zerv-calver schema preset
    use super::*;

    #[rstest]
    fn test_basic(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture.with_version(1, 2, 3).build().to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-calver", zerv_ron);

        assert!(!output.is_empty(), "CalVer schema should produce output");
    }

    #[rstest]
    fn test_with_distance(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture
            .with_version(1, 0, 0)
            .with_vcs_data(Some(3), Some(true), None, None, None, None, None)
            .build()
            .to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-calver", zerv_ron);

        let today_date = chrono::Utc::now().format("%Y.%-m.%-d").to_string();
        let expected = format!("{}-0+3", today_date);

        assert_eq!(output, expected, "CalVer tier 2 format");
    }

    #[rstest]
    fn test_with_dirty(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture
            .with_version(1, 0, 0)
            .with_vcs_data(Some(0), Some(true), None, None, None, None, None)
            .with_dev(DEV_TIMESTAMP)
            .build()
            .to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-calver", zerv_ron);

        assert!(
            output.contains("dev"),
            "CalVer tier 3 should include dev notation when dirty, got: {output}"
        );
    }
}

mod schema_defaults {
    //! Tests for default schema behavior
    use super::*;

    #[rstest]
    fn test_default_is_standard(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture.with_version(1, 2, 3).build().to_string();

        let output_explicit = TestCommand::run_with_stdin(
            "version --source stdin --schema zerv-standard",
            zerv_ron.clone(),
        );

        let output_default = TestCommand::run_with_stdin("version --source stdin", zerv_ron);

        assert_eq!(
            output_explicit, output_default,
            "Default schema should be zerv-standard"
        );
    }
}

mod schema_ron_custom {
    //! Tests for custom RON schema functionality
    use super::*;

    #[rstest]
    fn test_basic(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture.with_version(3, 2, 1).build().to_string();
        let schema_ron = ZervSchemaFixture::standard_tier_1().build().to_string();

        let args = format!("version --source stdin --schema-ron '{schema_ron}'");
        let output = TestCommand::run_with_stdin(&args, zerv_ron);

        assert_eq!(output, "3.2.1");
    }

    #[rstest]
    #[case::major_minor(
        (1, 2, 3),
        ZervSchemaFixture::empty().with_major_minor(),
        ZervFixture::new(),
        "version --source stdin --schema-ron",
        "1.2.0"
    )]
    #[case::epoch_in_extra_core(
        (1, 2, 3),
        ZervSchemaFixture::empty().with_major_minor_patch().with_epoch_in_extra_core(),
        ZervFixture::new().with_epoch(5),
        "version --source stdin --output-format pep440 --schema-ron",
        "5!1.2.3"
    )]
    #[case::prerelease_in_extra_core(
        (2, 0, 0),
        ZervSchemaFixture::empty().with_major_minor_patch().with_prerelease_in_extra_core(),
        ZervFixture::new().with_pre_release(PreReleaseLabel::Beta, Some(3)),
        "version --source stdin --output-format semver --schema-ron",
        "2.0.0-beta.3"
    )]
    fn test_custom_components(
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
    //! Tests for schema validation and error handling
    use super::*;

    #[rstest]
    fn test_unknown_preset_error(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture.build().to_string();

        let result = TestCommand::run_with_stdin_expect_fail(
            "version --source stdin --schema invalid-schema",
            zerv_ron,
        );
        assert!(
            result.contains("Unknown schema") || result.contains("invalid-schema"),
            "Should show unknown schema error, got: {result}"
        );
    }

    #[rstest]
    fn test_schema_and_schema_ron_conflict(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture.build().to_string();
        let schema_ron = ZervSchemaFixture::standard_tier_1().build().to_string();

        let result = TestCommand::run_with_stdin_expect_fail(
            &format!("version --source stdin --schema zerv-standard --schema-ron '{schema_ron}'"),
            zerv_ron,
        );
        assert!(
            result.contains("Conflicting") || result.contains("both"),
            "Should show conflict error when both schema \
             and schema-ron are specified, got: {result}"
        );
    }

    #[rstest]
    fn test_ron_invalid_syntax(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture.build().to_string();
        let invalid_schema = r#"(core:[var(Invalid),extra_core:[]"#;

        let result = TestCommand::run_with_stdin_expect_fail(
            &format!("version --source stdin --schema-ron '{invalid_schema}'"),
            zerv_ron,
        );
        assert!(
            result.contains("parse") || result.contains("invalid") || result.contains("RON"),
            "Should show RON parse error, got: {result}"
        );
    }
}

mod schema_output_formats {
    //! Tests for schema interaction with output formats
    use super::*;

    #[rstest]
    #[case::standard_semver("zerv-standard", "semver", "1.2.3")]
    #[case::standard_pep440("zerv-standard", "pep440", "1.2.3")]
    fn test_with_output_format(
        tier_1_fixture: ZervFixture,
        #[case] schema: &str,
        #[case] output_format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = tier_1_fixture.with_version(1, 2, 3).build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --schema {schema} --output-format {output_format}"),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_calver_with_output_formats(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture.with_version(1, 2, 3).build().to_string();

        let output_semver = TestCommand::run_with_stdin(
            "version --source stdin --schema zerv-calver --output-format semver",
            zerv_ron.clone(),
        );

        let output_pep440 = TestCommand::run_with_stdin(
            "version --source stdin --schema zerv-calver --output-format pep440",
            zerv_ron,
        );

        assert!(
            !output_semver.is_empty(),
            "CalVer with semver format should produce output"
        );
        assert!(
            !output_pep440.is_empty(),
            "CalVer with pep440 format should produce output"
        );
    }

    #[rstest]
    fn test_preserves_structure(tier_1_fixture: ZervFixture) {
        let original_zerv = tier_1_fixture
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .build();

        let zerv_ron = original_zerv.to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --schema zerv-standard --output-format zerv",
            zerv_ron,
        );

        let parsed_zerv: zerv::version::Zerv =
            ron::from_str(&output).expect("Failed to parse output as Zerv RON");

        assert_eq!(
            parsed_zerv.schema, original_zerv.schema,
            "Schema should be preserved through zerv output format"
        );
    }
}

mod schema_prerelease {
    //! Tests for schema behavior with prerelease versions
    use super::*;

    #[rstest]
    #[case::alpha(PreReleaseLabel::Alpha, Some(1), "1.0.0-alpha.1")]
    #[case::beta(PreReleaseLabel::Beta, Some(2), "1.0.0-beta.2")]
    #[case::rc(PreReleaseLabel::Rc, Some(3), "1.0.0-rc.3")]
    fn test_standard_with_prerelease(
        tier_1_fixture: ZervFixture,
        #[case] label: PreReleaseLabel,
        #[case] number: Option<u64>,
        #[case] expected: &str,
    ) {
        let zerv_ron = tier_1_fixture
            .with_version(1, 0, 0)
            .with_pre_release(label, number)
            .build()
            .to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_calver_with_prerelease(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture
            .with_version(1, 0, 0)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .build()
            .to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-calver", zerv_ron);

        assert!(
            output.contains("alpha"),
            "CalVer with alpha prerelease should include 'alpha' in output, got: {output}"
        );
    }
}

mod schema_tier_behavior {
    //! Tests for schema tier selection behavior
    use super::*;

    #[rstest]
    fn test_tier_selection_tagged_clean(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture
            .with_version(1, 2, 3)
            .with_vcs_data(Some(0), Some(false), None, None, None, None, None)
            .build()
            .to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(output, "1.2.3");
    }

    #[rstest]
    fn test_tier_selection_distance_clean(tier_2_fixture: ZervFixture) {
        let zerv_ron = tier_2_fixture.with_version(1, 2, 3).build().to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(output, "1.2.3+5");
    }

    #[rstest]
    fn test_tier_selection_dirty(dirty_fixture: ZervFixture) {
        let zerv_ron = dirty_fixture.build().to_string();

        let output =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(
            output, "1.2.3-dev.1234567890+main.branch.0.def456a",
            "Dirty state should trigger tier 3 with dev component"
        );
    }
}

mod schema_consistency {
    //! Tests for schema consistency across invocations
    use super::*;

    #[rstest]
    fn test_applied_consistently(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture.with_version(1, 2, 3).build().to_string();

        let output1 = TestCommand::run_with_stdin(
            "version --source stdin --schema zerv-standard",
            zerv_ron.clone(),
        );

        let output2 =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-standard", zerv_ron);

        assert_eq!(
            output1, output2,
            "Same schema should produce identical output for same input"
        );
    }

    #[rstest]
    fn test_different_schemas(tier_1_fixture: ZervFixture) {
        let zerv_ron = tier_1_fixture.with_version(1, 2, 3).build().to_string();

        let output_standard = TestCommand::run_with_stdin(
            "version --source stdin --schema zerv-standard",
            zerv_ron.clone(),
        );

        let output_calver =
            TestCommand::run_with_stdin("version --source stdin --schema zerv-calver", zerv_ron);

        assert!(
            !output_standard.is_empty(),
            "Standard schema should produce output"
        );
        assert!(
            !output_calver.is_empty(),
            "CalVer schema should produce output"
        );
    }
}
