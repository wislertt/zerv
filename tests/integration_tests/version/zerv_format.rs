use zerv::cli::VersionArgs;
use zerv::constants::{FORMAT_ZERV, SCHEMA_ZERV_STANDARD};
use zerv::test_utils::{GitRepoFixture, should_run_docker_tests};

#[test]
fn test_zerv_format_output() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::tagged("v1.2.3").expect("Failed to create tagged repo");

    let args = VersionArgs {
        version: None,
        source: "git".to_string(),
        schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
        schema_ron: None,
        output_format: FORMAT_ZERV.to_string(),
    };

    let result = zerv::cli::run_version_pipeline(args, Some(fixture.path().to_str().unwrap()));

    let output = result.expect("Pipeline should succeed");

    // Verify it's valid RON format
    assert!(output.contains("schema"));
    assert!(output.contains("vars"));
    assert!(output.contains("major"));
    assert!(output.contains("minor"));
    assert!(output.contains("patch"));

    // Verify it can be parsed back
    let parsed: zerv::version::zerv::Zerv = output.parse().expect("Should parse back to Zerv");
    assert_eq!(parsed.vars.major, Some(1));
    assert_eq!(parsed.vars.minor, Some(2));
    assert_eq!(parsed.vars.patch, Some(3));

    // Verify schema structure
    use zerv::version::zerv::Component;
    assert_eq!(parsed.schema.core.len(), 3);
    assert_eq!(
        parsed.schema.core[0],
        Component::VarField("major".to_string())
    );
    assert_eq!(
        parsed.schema.core[1],
        Component::VarField("minor".to_string())
    );
    assert_eq!(
        parsed.schema.core[2],
        Component::VarField("patch".to_string())
    );
}

#[test]
fn test_zerv_format_schema_structure() {
    // Test without Docker dependency
    use zerv::schema::create_zerv_version;
    use zerv::version::zerv::{Component, ZervVars};

    let vars = ZervVars {
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        dirty: Some(false),
        distance: Some(0),
        ..Default::default()
    };

    let zerv = create_zerv_version(vars, Some(SCHEMA_ZERV_STANDARD), None).unwrap();
    let ron_output = zerv.to_string();

    // Parse back and verify schema
    let parsed: zerv::version::zerv::Zerv = ron_output.parse().expect("Should parse back to Zerv");

    // Verify schema structure
    assert_eq!(parsed.schema.core.len(), 3);
    assert_eq!(
        parsed.schema.core[0],
        Component::VarField("major".to_string())
    );
    assert_eq!(
        parsed.schema.core[1],
        Component::VarField("minor".to_string())
    );
    assert_eq!(
        parsed.schema.core[2],
        Component::VarField("patch".to_string())
    );

    // Verify vars
    assert_eq!(parsed.vars.major, Some(1));
    assert_eq!(parsed.vars.minor, Some(2));
    assert_eq!(parsed.vars.patch, Some(3));
}

#[test]
fn test_zerv_format_roundtrip() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::tagged("v2.0.1").expect("Failed to create tagged repo");

    // Generate Zerv format
    let args1 = VersionArgs {
        version: None,
        source: "git".to_string(),
        schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
        schema_ron: None,
        output_format: FORMAT_ZERV.to_string(),
    };

    let zerv_output =
        zerv::cli::run_version_pipeline(args1, Some(fixture.path().to_str().unwrap()))
            .expect("First pipeline should succeed");

    // Parse it back and convert to PEP440
    let parsed: zerv::version::zerv::Zerv = zerv_output.parse().expect("Should parse Zerv format");
    let pep440_output = zerv::version::pep440::PEP440::from(parsed).to_string();

    assert_eq!(pep440_output, "2.0.1");
}
