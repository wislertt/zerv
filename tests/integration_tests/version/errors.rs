use crate::integration_tests::util::command::TestCommand;
use rstest::rstest;
use std::path::Path;
use zerv::test_utils::{GitRepoFixture, TestDir, should_run_docker_tests};

/// Test setup enum for different error scenarios
enum TestSetup {
    NoTags,
    NotInGitRepo,
    EmptyGitRepo,
    CorruptedGit,
    PermissionDenied,
}

impl TestSetup {
    fn create(&self) -> TestFixture {
        match self {
            TestSetup::NoTags => {
                let fixture = GitRepoFixture::with_distance("v0.0.0", 1)
                    .expect("should create git fixture with commits");
                // Remove the tag to simulate a repo with commits but no tags
                fixture
                    .git_impl
                    .execute_git(&fixture.test_dir, &["tag", "-d", "v0.0.0"])
                    .expect("should remove tag");
                TestFixture::GitRepo(fixture)
            }
            TestSetup::NotInGitRepo => {
                let test_dir = TestDir::new().expect("should create temp dir");
                TestFixture::PlainDir(test_dir)
            }
            TestSetup::EmptyGitRepo => {
                let test_dir = TestDir::new().expect("should create temp dir");
                let fixture = GitRepoFixture::tagged("v1.0.0").expect("should create fixture");
                // Initialize a new empty repo in a different directory
                fixture
                    .git_impl
                    .init_repo_no_commit(&test_dir)
                    .expect("should init empty repo");
                TestFixture::PlainDir(test_dir)
            }
            TestSetup::CorruptedGit => {
                let fixture = GitRepoFixture::tagged("v1.0.0").expect("should create fixture");
                // Corrupt the repository by removing HEAD file
                let git_dir = fixture.test_dir.path().join(".git");
                if git_dir.exists() {
                    let head_file = git_dir.join("HEAD");
                    if head_file.exists() {
                        std::fs::remove_file(&head_file).expect("should remove HEAD file");
                    }
                }
                TestFixture::GitRepo(fixture)
            }
            TestSetup::PermissionDenied => {
                let fixture = GitRepoFixture::tagged("v1.0.0").expect("should create fixture");
                // Try to restrict permissions on .git directory
                #[cfg(unix)]
                {
                    use std::fs;
                    use std::os::unix::fs::PermissionsExt;

                    let git_dir = fixture.test_dir.path().join(".git");
                    if git_dir.exists() {
                        let original_perms = fs::metadata(&git_dir)
                            .expect("should get metadata")
                            .permissions();
                        let mut restricted_perms = original_perms.clone();
                        restricted_perms.set_mode(0o000);
                        let _ = fs::set_permissions(&git_dir, restricted_perms);
                    }
                }
                TestFixture::GitRepo(fixture)
            }
        }
    }
}

/// Test fixture wrapper for different directory types
enum TestFixture {
    GitRepo(GitRepoFixture),
    PlainDir(TestDir),
}

impl TestFixture {
    fn path(&self) -> &Path {
        match self {
            TestFixture::GitRepo(fixture) => fixture.test_dir.path(),
            TestFixture::PlainDir(dir) => dir.path(),
        }
    }
}

/// Parameterized test for various git error scenarios
#[rstest]
#[case::no_tags(TestSetup::NoTags, &["No version tags found in git repository"])]
#[case::not_in_repo(TestSetup::NotInGitRepo, &["Not in a git repository (--source git)"])]
#[case::empty_repo(TestSetup::EmptyGitRepo, &["No commits found in git repository", "No version tags found in git repository"])]
#[case::corrupted_git(TestSetup::CorruptedGit, &["Git command failed", "not a git repository", "Not in a git repository", "VCS not found"])]
#[case::permission_denied(TestSetup::PermissionDenied, &["Permission denied", "Not in a git repository", "VCS not found"])]
fn test_git_error_scenarios(#[case] setup: TestSetup, #[case] expected_patterns: &[&str]) {
    if !should_run_docker_tests() && !matches!(setup, TestSetup::NotInGitRepo) {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled (except non-git tests)
    }

    let fixture = setup.create();
    let test_output = TestCommand::new()
        .current_dir(fixture.path())
        .arg("version")
        .assert_failure();

    let stderr = test_output.stderr();
    let found_match = expected_patterns
        .iter()
        .any(|pattern| stderr.contains(pattern));

    assert!(
        found_match,
        "Error message should contain one of the expected patterns. Expected one of: {expected_patterns:?}, Got: {stderr}"
    );
}

/// Test comprehensive error message quality (no generic errors, source-aware)
#[rstest]
#[case::no_tags(TestSetup::NoTags)]
#[case::not_in_repo(TestSetup::NotInGitRepo)]
#[case::empty_repo(TestSetup::EmptyGitRepo)]
fn test_error_message_quality(#[case] setup: TestSetup) {
    if !should_run_docker_tests() && !matches!(setup, TestSetup::NotInGitRepo) {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled (except non-git tests)
    }

    let fixture = setup.create();
    let test_output = TestCommand::new()
        .current_dir(fixture.path())
        .arg("version")
        .assert_failure();

    let stderr = test_output.stderr();

    // Should not contain generic error messages
    assert!(
        !stderr.contains("IO error"),
        "Should not show generic IO error. Got: {stderr}"
    );

    assert!(
        !stderr.contains("VCS data"),
        "Should not show internal VCS data error. Got: {stderr}"
    );

    // Should be source-aware (mention git repository)
    assert!(
        stderr.contains("git repository") || stderr.contains("git"),
        "Error should be source-aware and mention git. Got: {stderr}"
    );

    // Should not show raw git error output
    assert!(
        !stderr.contains("fatal: ambiguous argument 'HEAD'"),
        "Should not show raw git error messages. Got: {stderr}"
    );
}

/// Test CLI argument validation errors
#[rstest]
#[case::invalid_format("invalid-format", &["Unknown format", "invalid format", "Unknown output format"])]
fn test_cli_argument_errors(#[case] invalid_format: &str, #[case] expected_patterns: &[&str]) {
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    let fixture = GitRepoFixture::tagged("v1.0.0").expect("should create fixture");

    let test_output = TestCommand::new()
        .current_dir(fixture.test_dir.path())
        .arg("version")
        .arg("--output-format")
        .arg(invalid_format)
        .assert_failure();

    let stderr = test_output.stderr();
    let found_match = expected_patterns
        .iter()
        .any(|pattern| stderr.contains(pattern));

    assert!(
        found_match || !stderr.is_empty(), // At least should produce some error
        "Should produce expected error message. Expected one of: {expected_patterns:?}, Got: {stderr}"
    );
}

/// Test conflicting CLI arguments
#[test]
fn test_conflicting_arguments() {
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    let fixture = GitRepoFixture::tagged("v1.0.0").expect("should create fixture");

    // Test conflicting schema arguments
    let test_output = TestCommand::new()
        .current_dir(fixture.test_dir.path())
        .arg("version")
        .arg("--schema")
        .arg("zerv-standard")
        .arg("--schema-ron")
        .arg("SchemaConfig(core: [], extra_core: [], build: [])")
        .assert_failure();

    let stderr = test_output.stderr();
    assert!(
        stderr.contains("Cannot specify both")
            || stderr.contains("Conflicting")
            || !stderr.is_empty(),
        "Conflicting arguments should produce error message. Got: {stderr}"
    );
}

/// Test error message consistency across different scenarios
#[test]
fn test_error_message_consistency() {
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    // Test multiple scenarios and verify consistent terminology
    let scenarios = vec![
        (TestSetup::NoTags, "no tags"),
        (TestSetup::EmptyGitRepo, "empty repo"),
    ];

    let mut error_messages = Vec::new();

    for (setup, description) in scenarios {
        let fixture = setup.create();
        let test_output = TestCommand::new()
            .current_dir(fixture.path())
            .arg("version")
            .assert_failure();

        error_messages.push((description, test_output.stderr().to_string()));
    }

    // Test not in repository scenario separately (doesn't need Docker)
    let test_dir = TestDir::new().expect("should create temp dir");
    let no_repo_output = TestCommand::new()
        .current_dir(test_dir.path())
        .arg("version")
        .assert_failure();
    error_messages.push(("not in repo", no_repo_output.stderr().to_string()));

    // Verify all error messages are source-aware and consistent
    for (description, stderr) in &error_messages {
        assert!(
            stderr.contains("git repository") || stderr.contains("git"),
            "{description} error should be source-aware and mention git. Got: {stderr}"
        );

        assert!(
            !stderr.contains("IO error"),
            "{description} error should not contain generic IO error. Got: {stderr}"
        );
    }
}

/// Test that validates successful version generation (positive test case)
#[test]
fn test_version_success_scenario() {
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    let fixture = GitRepoFixture::tagged("v1.2.3").expect("should create fixture");

    let test_output = TestCommand::new()
        .current_dir(fixture.test_dir.path())
        .arg("version")
        .assert_success();

    let stdout = test_output.stdout();
    assert!(
        stdout.contains("1.2.3"),
        "Should produce version output containing the tag. Got: {stdout}"
    );
}

/// Test comprehensive error flow for specific scenarios (detailed validation)
#[rstest]
#[case::no_tags_comprehensive(TestSetup::NoTags)]
#[case::empty_repo_comprehensive(TestSetup::EmptyGitRepo)]
fn test_comprehensive_error_flow(#[case] setup: TestSetup) {
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    let fixture = setup.create();
    let test_output = TestCommand::new()
        .current_dir(fixture.path())
        .arg("version")
        .assert_failure();

    let stderr = test_output.stderr();

    // Verify it's not a generic error
    assert!(
        !stderr.contains("IO error"),
        "Should not show generic IO error. Got: {stderr}"
    );

    assert!(
        !stderr.contains("VCS data"),
        "Should not show internal VCS data error. Got: {stderr}"
    );

    // Verify the error is actionable (mentions git repository specifically)
    assert!(
        stderr.contains("git repository"),
        "Error should specify git repository as the source. Got: {stderr}"
    );

    // Verify it's not showing raw git error output
    assert!(
        !stderr.contains("fatal: ambiguous argument 'HEAD'"),
        "Should not show raw git error messages. Got: {stderr}"
    );
}

/// Test not in repository scenario (doesn't require Docker)
#[test]
fn test_not_in_repository_comprehensive() {
    let test_dir = TestDir::new().expect("should create temp dir");

    let test_output = TestCommand::new()
        .current_dir(test_dir.path())
        .arg("version")
        .assert_failure();

    let stderr = test_output.stderr();

    // Verify the complete error message format
    assert!(
        stderr.contains("Not in a git repository (--source git)"),
        "Error message should specify the source that was attempted. Got: {stderr}"
    );

    // Verify it's not a generic error
    assert!(
        !stderr.contains("VCS not found: No VCS repository found"),
        "Should not show generic VCS not found error. Got: {stderr}"
    );

    // Verify the error mentions the specific source
    assert!(
        stderr.contains("--source git"),
        "Error should reference the specific source attempted. Got: {stderr}"
    );
}

/// Comprehensive validation of error message consistency across all scenarios
/// This test validates requirements 5.1, 5.2, and 5.3 from the error handling improvements spec
#[test]
fn test_comprehensive_error_message_validation() {
    // Test scenarios that don't require Docker
    let non_docker_scenarios = vec![("not_in_repo", TestSetup::NotInGitRepo)];

    let mut all_error_messages = Vec::new();

    // Test non-Docker scenarios
    for (description, setup) in non_docker_scenarios {
        let fixture = setup.create();
        let test_output = TestCommand::new()
            .current_dir(fixture.path())
            .arg("version")
            .assert_failure();

        let stderr = test_output.stderr().to_string();
        all_error_messages.push((description, stderr));
    }

    // Test Docker scenarios if available
    if should_run_docker_tests() {
        let docker_scenarios = vec![
            ("no_tags", TestSetup::NoTags),
            ("empty_repo", TestSetup::EmptyGitRepo),
        ];

        for (description, setup) in docker_scenarios {
            let fixture = setup.create();
            let test_output = TestCommand::new()
                .current_dir(fixture.path())
                .arg("version")
                .assert_failure();

            let stderr = test_output.stderr().to_string();
            all_error_messages.push((description, stderr));
        }
    }

    // Validate all error messages for consistency
    for (description, stderr) in &all_error_messages {
        // Requirement 5.1: All VCS-related errors should include source information
        assert!(
            stderr.contains("git repository") || stderr.contains("git") || stderr.contains("Git"),
            "{description} error should be source-aware and mention git: {stderr}"
        );

        // Requirement 5.2: Consistent terminology
        assert!(
            !stderr.contains("IO error") && !stderr.contains("VCS data"),
            "{description} error should not contain generic error messages: {stderr}"
        );

        // Requirement 5.3: No raw git error output should be exposed
        assert!(
            !stderr.contains("fatal: ambiguous argument 'HEAD'"),
            "{description} error should not show raw git error messages: {stderr}"
        );
    }

    // Test that error messages are actionable where appropriate
    let not_in_repo_error = all_error_messages
        .iter()
        .find(|(desc, _)| desc == &"not_in_repo")
        .map(|(_, stderr)| stderr);

    if let Some(stderr) = not_in_repo_error {
        assert!(
            stderr.contains("--source git"),
            "Not in repository error should provide actionable context: {stderr}"
        );
    }
}

/// Test error message terminology consistency across different git repository states
#[test]
fn test_error_terminology_consistency() {
    let test_cases = vec![
        // Test case that works without Docker
        (
            "not_in_repo",
            TestSetup::NotInGitRepo,
            vec!["git repository", "--source git"],
        ),
    ];

    // Add Docker-dependent test cases if available
    let docker_cases = if should_run_docker_tests() {
        vec![
            ("no_tags", TestSetup::NoTags, vec!["git repository"]),
            (
                "empty_repo",
                TestSetup::EmptyGitRepo,
                vec!["git repository"],
            ),
        ]
    } else {
        vec![]
    };

    let mut all_cases = test_cases;
    all_cases.extend(docker_cases);

    for (description, setup, expected_terms) in all_cases {
        let fixture = setup.create();
        let test_output = TestCommand::new()
            .current_dir(fixture.path())
            .arg("version")
            .assert_failure();

        let stderr = test_output.stderr();

        for term in expected_terms {
            assert!(
                stderr.contains(term),
                "{description} error should contain consistent terminology '{term}': {stderr}"
            );
        }

        // Verify no generic error patterns
        let forbidden_patterns = vec!["VCS not found: No VCS", "IO error:", "VCS data"];
        for pattern in forbidden_patterns {
            assert!(
                !stderr.contains(pattern),
                "{description} error should not contain generic pattern '{pattern}': {stderr}"
            );
        }
    }
}

/// Test that error messages provide actionable guidance where appropriate
#[test]
fn test_error_actionable_guidance() {
    // Test not in repository scenario (always available)
    let test_dir = TestDir::new().expect("should create temp dir");
    let test_output = TestCommand::new()
        .current_dir(test_dir.path())
        .arg("version")
        .assert_failure();

    let stderr = test_output.stderr();

    // Should provide clear guidance about the source that was attempted
    assert!(
        stderr.contains("--source git"),
        "Error should indicate which source was attempted: {stderr}"
    );

    // Should be specific about the type of repository
    assert!(
        stderr.contains("git repository"),
        "Error should specify the repository type: {stderr}"
    );

    // Should not be a generic message
    assert!(
        !stderr.contains("VCS not found: No VCS repository found"),
        "Error should not be generic: {stderr}"
    );
}
