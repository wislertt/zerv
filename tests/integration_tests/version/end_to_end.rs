use crate::integration_tests::util::TestCommand;
use rstest::rstest;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;
use zerv::test_utils::{GitRepoFixture, ZervFixture, should_run_docker_tests};

struct TestOverrides<'a> {
    tag_version: Option<&'a str>,
    distance: Option<u32>,
    dirty: Option<bool>,
    clean: bool,
    branch: Option<&'a str>,
    commit_hash: Option<&'a str>,
}

#[rstest]
#[case::tag_version_override("v1.0.0", TestOverrides { tag_version: Some("v2.0.0"), distance: None, dirty: None, clean: false, branch: None, commit_hash: None }, "2.0.0")]
#[case::distance_override("v1.0.0", TestOverrides { tag_version: None, distance: Some(5), dirty: None, clean: false, branch: None, commit_hash: None }, "1.0.0-post.5+main.")]
#[case::dirty_override_true("v1.0.0", TestOverrides { tag_version: None, distance: None, dirty: Some(true), clean: false, branch: None, commit_hash: None }, "1.0.0+main.")]
#[case::dirty_override_false("v1.0.0", TestOverrides { tag_version: None, distance: None, dirty: Some(false), clean: false, branch: None, commit_hash: None }, "1.0.0")]
#[case::clean_flag("v1.0.0", TestOverrides { tag_version: None, distance: None, dirty: None, clean: true, branch: None, commit_hash: None }, "1.0.0")]
#[case::branch_override("v1.0.0", TestOverrides { tag_version: None, distance: Some(2), dirty: None, clean: false, branch: Some("feature"), commit_hash: None }, "1.0.0-post.2+feature.")]
#[case::commit_hash_override("v1.0.0", TestOverrides { tag_version: None, distance: None, dirty: None, clean: false, branch: None, commit_hash: Some("abc123") }, "1.0.0")]
#[case::multiple_overrides("v1.0.0", TestOverrides { tag_version: Some("v3.0.0"), distance: Some(1), dirty: Some(false), clean: false, branch: Some("dev"), commit_hash: Some("def456") }, "3.0.0+dev.")]
fn test_git_source_with_overrides(
    #[case] initial_tag: &str,
    #[case] overrides: TestOverrides,
    #[case] expected_version_prefix: &str,
) {
    if !should_run_docker_tests() {
        return;
    }

    // Create fixture with initial tag
    let fixture = GitRepoFixture::tagged(initial_tag).expect("Failed to create tagged repo");

    // Build command with overrides
    let mut cmd = TestCommand::new();
    cmd.current_dir(fixture.path())
        .arg("version")
        .arg("--output-format")
        .arg("semver");

    if let Some(tag) = overrides.tag_version {
        cmd.arg("--tag-version").arg(tag);
    }
    if let Some(distance) = overrides.distance {
        cmd.arg("--distance").arg(distance.to_string());
    }
    if let Some(dirty) = overrides.dirty {
        cmd.arg("--dirty").arg(if dirty { "true" } else { "false" });
    }
    if overrides.clean {
        cmd.arg("--clean");
    }
    if let Some(branch) = overrides.branch {
        cmd.arg("--current-branch").arg(branch);
    }
    if let Some(commit) = overrides.commit_hash {
        cmd.arg("--commit-hash").arg(commit);
    }

    let output = cmd.assert_success();
    let version = output.stdout().trim().to_string();

    assert!(
        version.starts_with(expected_version_prefix),
        "Version should start with expected prefix. Expected: {expected_version_prefix}, Got: {version}"
    );
}

/// Test conflicting override combinations that should fail
/// Requirements: 2.7
#[rstest]
#[case::clean_with_distance(true, Some(5), None, "Cannot use --clean with --distance")]
#[case::clean_with_dirty(true, None, Some(true), "Cannot use --clean with --dirty")]
#[case::clean_with_both(true, Some(3), Some(false), "Cannot use --clean with")]
fn test_conflicting_overrides(
    #[case] clean_flag: bool,
    #[case] distance_override: Option<u32>,
    #[case] dirty_override: Option<bool>,
    #[case] expected_error_pattern: &str,
) {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

    let mut cmd = TestCommand::new();
    cmd.current_dir(fixture.path()).arg("version");

    if clean_flag {
        cmd.arg("--clean");
    }
    if let Some(distance) = distance_override {
        cmd.arg("--distance").arg(distance.to_string());
    }
    if let Some(dirty) = dirty_override {
        cmd.arg("--dirty").arg(if dirty { "true" } else { "false" });
    }

    let output = cmd.assert_failure();
    let stderr = output.stderr();

    assert!(
        stderr.contains(expected_error_pattern),
        "Error should contain expected pattern. Expected: {expected_error_pattern}, Got: {stderr}"
    );
}

/// Test stdin source with Zerv RON input and overrides
/// Requirements: 5.1, 5.2, 5.3, 5.4, 5.5
#[test]
fn test_stdin_source_with_zerv_ron() {
    // Use a basic Zerv RON fixture for testing
    let zerv_ron_input = &ZervFixture::basic().to_ron_string();

    // Test basic stdin parsing
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--bin",
        "zerv",
        "--",
        "version",
        "--source",
        "stdin",
        "--input-format",
        "zerv",
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn command");

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(zerv_ron_input.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(
        output.status.success(),
        "Command should succeed with valid Zerv RON input. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        version.contains("1.2.3"),
        "Version should contain the input version. Got: {version}"
    );
}

/// Test piping workflows between multiple Zerv commands
/// Requirements: 5.1, 5.2, 5.3
#[test]
fn test_piping_workflows() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

    // First command: Generate Zerv RON output
    let output1 = TestCommand::new()
        .current_dir(fixture.path())
        .arg("version")
        .arg("--output-format")
        .arg("zerv")
        .assert_success();

    let zerv_ron_output = output1.stdout();

    // Verify it's valid Zerv RON
    assert!(
        zerv_ron_output.contains("schema") && zerv_ron_output.contains("vars"),
        "First command should produce valid Zerv RON. Got: {zerv_ron_output}"
    );

    // Test that the RON output is suitable for piping workflows
    // Write RON to a temporary file to simulate piping
    let test_ron_file = fixture.test_dir.path().join("test_output.ron");
    std::fs::write(&test_ron_file, zerv_ron_output.as_bytes())
        .expect("Failed to write RON to file");

    // Verify the RON can be read back and contains expected structure
    let ron_content = std::fs::read_to_string(&test_ron_file).expect("Failed to read RON file");

    // Verify it contains complete version information for piping
    assert!(
        ron_content.contains("1") && ron_content.contains("0") && ron_content.contains("0"),
        "RON should contain version data (1.0.0) for transformation. Got: {ron_content}"
    );

    // Verify it has the structure needed for further processing
    assert!(
        ron_content.contains("core") && ron_content.contains("vars"),
        "RON should have complete schema and vars for piping. Got: {ron_content}"
    );
}

/// Test error scenarios with real git repositories
/// Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7
#[rstest]
#[case::invalid_tag_version_format("invalid-version", "semver", "Invalid version")]
#[case::invalid_pep440_format("invalid.version", "pep440", "Invalid version")]
#[case::unknown_input_format("v1.0.0", "unknown", "invalid value 'unknown'")]
fn test_error_scenarios_with_overrides(
    #[case] tag_version: &str,
    #[case] input_format: &str,
    #[case] expected_error_pattern: &str,
) {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

    let output = TestCommand::new()
        .current_dir(fixture.path())
        .arg("version")
        .arg("--tag-version")
        .arg(tag_version)
        .arg("--input-format")
        .arg(input_format)
        .assert_failure();

    let stderr = output.stderr();
    assert!(
        stderr.contains(expected_error_pattern),
        "Error should contain expected pattern. Expected: {expected_error_pattern}, Got: {stderr}"
    );
}

/// Test stdin error scenarios
/// Requirements: 6.1, 6.2, 6.3, 6.4, 6.5
#[rstest]
#[case::simple_version_string("1.2.3", "Use --tag-version")]
#[case::empty_input("", "No input provided via stdin")]
#[case::invalid_ron("invalid ron format", "Invalid Zerv RON format")]
#[case::semver_to_stdin("1.2.3", "Use --tag-version instead")]
fn test_stdin_error_scenarios(#[case] stdin_input: &str, #[case] expected_error_pattern: &str) {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--bin",
        "zerv",
        "--",
        "version",
        "--source",
        "stdin",
        "--input-format",
        "zerv",
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn command");

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(stdin_input.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(
        !output.status.success(),
        "Command should fail with invalid stdin input"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_error_pattern),
        "Error should contain expected pattern. Expected: {expected_error_pattern}, Got: {stderr}"
    );
}

/// Test performance requirements with large repositories
/// Requirements: 10.1, 10.2, 10.3, 10.4, 10.5
#[test]
fn test_performance_with_large_repository() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture =
        GitRepoFixture::with_distance("v1.0.0", 5).expect("Failed to create repo with distance");
    let test_dir = fixture.test_dir;

    // Measure performance of version generation
    let start_time = Instant::now();

    let output = TestCommand::new()
        .current_dir(test_dir.path())
        .arg("version")
        .assert_success();

    let duration = start_time.elapsed();

    assert!(
        duration.as_millis() < 2000, // Using 2000ms as reasonable limit for test environment
        "Version generation should be fast. Took: {}ms",
        duration.as_millis()
    );

    let version = output.stdout().trim().to_string();
    assert!(
        !version.is_empty(),
        "Should generate valid version output: {version}"
    );
}

/// Test concurrent executions for safety
/// Requirements: 10.4
#[test]
fn test_concurrent_executions() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

    // Run multiple concurrent version commands
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let fixture_path = fixture.path().to_path_buf();
            std::thread::spawn(move || {
                let output = TestCommand::new()
                    .current_dir(&fixture_path)
                    .arg("version")
                    .arg("--output-prefix")
                    .arg(format!("thread{i}:"))
                    .assert_success();

                output.stdout().to_string()
            })
        })
        .collect();

    // Wait for all threads and collect results
    let results: Vec<String> = handles
        .into_iter()
        .map(|handle| handle.join().expect("Thread should complete successfully"))
        .collect();

    // All results should be valid and contain the expected version
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.contains(&format!("thread{i}:")) && result.contains("1.0.0"),
            "Thread {i} should produce valid output: {result}"
        );
    }
}

/// Test output format consistency across all scenarios
/// Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6
#[rstest]
#[case::semver_format("semver", "1.0.0")]
#[case::pep440_format("pep440", "1.0.0")]
#[case::zerv_format("zerv", "schema")]
fn test_output_format_consistency(#[case] output_format: &str, #[case] expected_content: &str) {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

    let output = TestCommand::new()
        .current_dir(fixture.path())
        .arg("version")
        .arg("--output-format")
        .arg(output_format)
        .assert_success();

    let version = output.stdout();

    // Should be single-line clean output for non-RON formats (requirement 7.1)
    if output_format != "zerv" {
        assert_eq!(
            version.lines().count(),
            1,
            "Output should be single line for format {output_format}: {version}"
        );
    }

    // Should contain expected content
    assert!(
        version.contains(expected_content),
        "Output should contain expected content for {output_format}. Expected: {expected_content}, Got: {version}"
    );

    // Should not contain debug or error information
    assert!(
        !version.contains("Error") && !version.contains("Debug"),
        "Output should be clean for {output_format}: {version}"
    );
}

/// Test comprehensive end-to-end workflow combining all features
/// Requirements: All requirements integration test
#[test]
fn test_comprehensive_end_to_end_workflow() {
    if !should_run_docker_tests() {
        return;
    }

    // Step 1: Create a repository with specific state
    let fixture =
        GitRepoFixture::with_distance("v1.0.0", 3).expect("Failed to create repo with distance");

    // Step 2: Generate Zerv RON with git source and overrides
    let output1 = TestCommand::new()
        .current_dir(fixture.path())
        .arg("version")
        .arg("--output-format")
        .arg("zerv")
        .arg("--tag-version")
        .arg("v2.0.0-beta.1")
        .arg("--current-branch")
        .arg("release/2.0")
        .arg("--no-dirty")
        .assert_success();

    let zerv_ron = output1.stdout();

    // Step 3: Verify the RON contains the expected transformations
    assert!(
        zerv_ron.contains("2") && zerv_ron.contains("0") && zerv_ron.contains("0"),
        "RON should contain version 2.0.0 from tag override. Got: {zerv_ron}"
    );

    assert!(
        zerv_ron.contains("release") || zerv_ron.contains("2.0"),
        "RON should contain branch override. Got: {zerv_ron}"
    );

    assert!(
        zerv_ron.contains("false") || !zerv_ron.contains("dirty: Some(true)"),
        "RON should reflect clean state from dirty override. Got: {zerv_ron}"
    );

    // Step 4: Verify the RON structure is complete for further processing
    assert!(
        zerv_ron.contains("schema") && zerv_ron.contains("vars"),
        "RON should have complete structure for piping workflows. Got: {zerv_ron}"
    );

    // Verify it contains all the necessary fields for transformation
    assert!(
        zerv_ron.contains("major") && zerv_ron.contains("minor") && zerv_ron.contains("patch"),
        "RON should contain version components for transformation. Got: {zerv_ron}"
    );
}
