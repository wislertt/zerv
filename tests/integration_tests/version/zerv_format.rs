use zerv::constants::ron_fields;
use zerv::constants::{formats, schema_names};
use zerv::test_utils::{GitRepoFixture, VersionArgsFixture, should_run_docker_tests};

#[test]
fn test_zerv_format_output() {
    if !should_run_docker_tests() {
        return;
    }

    // Use atomic GitRepoFixture creation with better error context
    let fixture = GitRepoFixture::tagged("v1.2.3")
        .expect("Failed to create tagged repo - check Docker availability and Git operations");

    // Verify Git repository state before proceeding
    assert!(
        fixture.path().join(".git").exists(),
        "Git repository should exist at: {}",
        fixture.path().display()
    );
    assert!(
        fixture.path().join("README.md").exists(),
        "Initial commit should create README.md at: {}",
        fixture.path().display()
    );

    let args = VersionArgsFixture::new()
        .with_schema(schema_names::ZERV_STANDARD)
        .with_output_format(formats::ZERV)
        .with_directory(fixture.path().to_str().unwrap())
        .build();

    // Run pipeline with detailed error context
    let result = zerv::cli::run_version_pipeline(args);
    let output = result.unwrap_or_else(|e| {
        panic!(
            "Pipeline should succeed for tagged repo at {}: {}",
            fixture.path().display(),
            e
        );
    });

    // Verify it's valid RON format with detailed assertions
    assert!(
        output.contains("schema"),
        "Output should contain 'schema' field. Got: {output}"
    );
    assert!(
        output.contains("vars"),
        "Output should contain 'vars' field. Got: {output}"
    );
    assert!(
        output.contains("major"),
        "Output should contain 'major' field. Got: {output}"
    );
    assert!(
        output.contains("minor"),
        "Output should contain 'minor' field. Got: {output}"
    );
    assert!(
        output.contains("patch"),
        "Output should contain 'patch' field. Got: {output}"
    );

    // Verify it can be parsed back with detailed error context
    let parsed: zerv::version::zerv::Zerv = output
        .parse()
        .unwrap_or_else(|e| panic!("Should parse back to Zerv. Output: {output}\nError: {e}"));

    // Verify parsed values with detailed assertions
    assert_eq!(
        parsed.vars.major,
        Some(1),
        "Major version should be 1, got: {:?}",
        parsed.vars.major
    );
    assert_eq!(
        parsed.vars.minor,
        Some(2),
        "Minor version should be 2, got: {:?}",
        parsed.vars.minor
    );
    assert_eq!(
        parsed.vars.patch,
        Some(3),
        "Patch version should be 3, got: {:?}",
        parsed.vars.patch
    );

    // Verify schema structure with detailed assertions
    use zerv::version::zerv::Component;
    assert_eq!(
        parsed.schema.core.len(),
        3,
        "Schema should have 3 core components, got: {}",
        parsed.schema.core.len()
    );
    assert_eq!(
        parsed.schema.core[0],
        Component::VarField(ron_fields::MAJOR.to_string()),
        "First schema component should be major field"
    );
    assert_eq!(
        parsed.schema.core[1],
        Component::VarField(ron_fields::MINOR.to_string()),
        "Second schema component should be minor field"
    );
    assert_eq!(
        parsed.schema.core[2],
        Component::VarField(ron_fields::PATCH.to_string()),
        "Third schema component should be patch field"
    );
}

#[test]
fn test_zerv_format_schema_structure() {
    // Test without Docker dependency
    use zerv::cli::version::ZervDraft;
    use zerv::version::zerv::{Component, ZervVars};

    let vars = ZervVars {
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        dirty: Some(false),
        distance: Some(0),
        ..Default::default()
    };

    let zerv = ZervDraft::new(vars, None)
        .create_zerv_version(Some(schema_names::ZERV_STANDARD), None)
        .unwrap();
    let ron_output = zerv.to_string();

    // Parse back and verify schema
    let parsed: zerv::version::zerv::Zerv = ron_output.parse().expect("Should parse back to Zerv");

    // Verify schema structure
    assert_eq!(parsed.schema.core.len(), 3);
    assert_eq!(
        parsed.schema.core[0],
        Component::VarField(ron_fields::MAJOR.to_string())
    );
    assert_eq!(
        parsed.schema.core[1],
        Component::VarField(ron_fields::MINOR.to_string())
    );
    assert_eq!(
        parsed.schema.core[2],
        Component::VarField(ron_fields::PATCH.to_string())
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
    let args1 = VersionArgsFixture::new()
        .with_schema(schema_names::ZERV_STANDARD)
        .with_output_format(formats::ZERV)
        .with_directory(fixture.path().to_str().unwrap())
        .build();

    let zerv_output =
        zerv::cli::run_version_pipeline(args1).expect("First pipeline should succeed");

    // Parse it back and convert to PEP440
    let parsed: zerv::version::zerv::Zerv = zerv_output.parse().expect("Should parse Zerv format");
    let pep440_output = zerv::version::pep440::PEP440::from(parsed).to_string();

    assert_eq!(pep440_output, "2.0.1");
}
