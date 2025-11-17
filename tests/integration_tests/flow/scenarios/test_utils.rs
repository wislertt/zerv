// Test utilities for flow scenario integration tests
// Reuses utilities from src/cli/flow/test_utils.rs

use zerv::cli::flow::test_utils::SchemaTestCase;
use zerv::test_utils::{
    GitRepoFixture,
    assert_version_expectation,
};

use crate::integration_tests::util::command::TestCommand;

/// Flow integration test scenario builder pattern for CLI testing
/// Same API as FlowTestScenario but uses CLI commands internally
pub struct FlowIntegrationTestScenario {
    fixture: GitRepoFixture,
}

impl FlowIntegrationTestScenario {
    /// Create an empty git repository for flow integration testing
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let fixture = GitRepoFixture::empty()
            .map_err(|e| format!("Failed to create empty git fixture: {}", e))?;

        Ok(Self { fixture })
    }

    /// Create a tag in the current git repository
    pub fn create_tag(self, tag: &str) -> Self {
        Self {
            fixture: self.fixture.create_tag(tag),
        }
    }

    /// Create a new branch without checking it out
    pub fn create_branch(self, branch_name: &str) -> Self {
        Self {
            fixture: self.fixture.with_branch(branch_name),
        }
    }

    /// Checkout to an existing branch
    pub fn checkout(self, branch_name: &str) -> Self {
        Self {
            fixture: self.fixture.with_checkout(branch_name),
        }
    }

    /// Make a commit
    pub fn commit(self) -> Self {
        Self {
            fixture: self.fixture.commit("Test commit"),
        }
    }

    /// Make working directory dirty
    pub fn make_dirty(self) -> Self {
        Self {
            fixture: self.fixture.with_dirty(),
        }
    }

    /// Merge a branch
    pub fn merge_branch(self, branch_name: &str) -> Self {
        Self {
            fixture: self.fixture.merge_branch(branch_name),
        }
    }

    /// Expect version output with semver and pep440 formats
    /// Uses CLI command instead of direct function call
    pub fn expect_version(self, semver: &str, pep440: &str) -> Self {
        test_flow_pipeline_with_fixture(&self.test_dir_path(), semver, pep440);
        self
    }

    /// Expect schema variants with the same API as FlowTestScenario
    /// Uses CLI command for each test case instead of direct function call
    pub fn expect_schema_variants(self, test_cases: Vec<SchemaTestCase>) -> Self {
        test_flow_pipeline_with_schema_test_cases(&self.test_dir_path(), test_cases);
        self
    }

    /// Run flow command with custom arguments
    pub fn run_flow_command(self, args: &[&str]) -> FlowTestResult {
        let mut cmd = TestCommand::new();
        cmd.arg("flow").args(args);
        cmd.current_dir(self.test_dir_path());

        let output = cmd.output().expect("Failed to execute flow command");

        FlowTestResult {
            scenario: self,
            output,
            command_args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Get the test directory path
    pub fn test_dir_path(&self) -> String {
        self.fixture.path().to_string_lossy().to_string()
    }
}

/// Result of running a flow command test
pub struct FlowTestResult {
    scenario: FlowIntegrationTestScenario,
    output: std::process::Output,
    command_args: Vec<String>,
}

impl FlowTestResult {
    /// Assert that the command succeeded and return the scenario for chaining
    pub fn assert_success(self) -> FlowIntegrationTestScenario {
        assert!(
            self.output.status.success(),
            "Flow command failed with exit code: {:?}\nstderr: {}\nargs: {}",
            self.output.status.code(),
            String::from_utf8_lossy(&self.output.stderr),
            self.command_args.join(" ")
        );
        self.scenario
    }

    /// Assert that the command failed and return the scenario for chaining
    pub fn assert_failure(self) -> FlowIntegrationTestScenario {
        assert!(
            !self.output.status.success(),
            "Expected flow command to fail but it succeeded\nargs: {}",
            self.command_args.join(" ")
        );
        self.scenario
    }

    /// Assert that the command succeeded and check stdout content
    pub fn assert_stdout_contains(self, expected: &str) -> FlowIntegrationTestScenario {
        assert!(
            self.output.status.success(),
            "Flow command failed with exit code: {:?}\nstderr: {}\nargs: {}",
            self.output.status.code(),
            String::from_utf8_lossy(&self.output.stderr),
            self.command_args.join(" ")
        );

        let stdout = String::from_utf8_lossy(&self.output.stdout);
        assert!(
            stdout.contains(expected),
            "Expected stdout to contain '{}', but got: '{}'",
            expected,
            stdout
        );
        self.scenario
    }

    /// Assert that the command succeeded and check exact stdout
    pub fn assert_stdout_eq(self, expected: &str) -> FlowIntegrationTestScenario {
        assert!(
            self.output.status.success(),
            "Flow command failed with exit code: {:?}\nstderr: {}\nargs: {}",
            self.output.status.code(),
            String::from_utf8_lossy(&self.output.stderr),
            self.command_args.join(" ")
        );

        let stdout = String::from_utf8_lossy(&self.output.stdout);
        let trimmed_stdout = stdout.trim();
        let trimmed_expected = expected.trim();

        assert_eq!(
            trimmed_stdout, trimmed_expected,
            "Expected stdout to be exactly '{}', but got: '{}'",
            trimmed_expected, trimmed_stdout
        );
        self.scenario
    }

    /// Assert that the command failed and check stderr content
    pub fn assert_stderr_contains(self, expected: &str) -> FlowIntegrationTestScenario {
        assert!(
            !self.output.status.success(),
            "Expected flow command to fail but it succeeded\nargs: {}",
            self.command_args.join(" ")
        );

        let stderr = String::from_utf8_lossy(&self.output.stderr);
        assert!(
            stderr.contains(expected),
            "Expected stderr to contain '{}', but got: '{}'",
            expected,
            stderr
        );
        self.scenario
    }

    /// Get stdout as string
    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).to_string()
    }

    /// Get stderr as string
    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).to_string()
    }

    /// Get exit code
    pub fn exit_code(&self) -> Option<i32> {
        self.output.status.code()
    }
}

pub fn test_flow_pipeline_with_fixture(
    fixture_path: &str,
    semver_expectation: &str,
    pep440_expectation: &str,
) {
    test_flow_pipeline_with_fixture_and_schema_opt(
        fixture_path,
        None,
        semver_expectation,
        pep440_expectation,
    )
}

pub fn test_flow_pipeline_with_fixture_and_schema(
    fixture_path: &str,
    schema: &str,
    semver_expectation: &str,
    pep440_expectation: &str,
) {
    test_flow_pipeline_with_fixture_and_schema_opt(
        fixture_path,
        Some(schema),
        semver_expectation,
        pep440_expectation,
    )
}

pub fn test_flow_pipeline_with_fixture_and_schema_opt(
    fixture_path: &str,
    schema: Option<&str>,
    semver_expectation: &str,
    pep440_expectation: &str,
) {
    let test_cases = vec![
        ("semver", semver_expectation),
        ("pep440", pep440_expectation),
    ];

    for (format_name, expectation) in test_cases {
        let mut cmd_args = vec!["flow", "--output-format", format_name];

        // Set schema if provided
        if let Some(schema_value) = schema {
            cmd_args.push("--schema");
            cmd_args.push(schema_value);
        }

        let mut cmd = TestCommand::new();
        cmd.args_from_str(cmd_args.join(" "));
        cmd.current_dir(fixture_path);

        let output = cmd.output().expect("Failed to execute flow command");
        let schema_desc = match schema {
            Some(s) => format!(" and {} schema", s),
            None => String::new(),
        };

        assert!(
            output.status.success(),
            "Flow command should succeed with {} format{} at {}: {}",
            format_name,
            schema_desc,
            fixture_path,
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "Flow pipeline should produce output for {} format{}",
            format_name,
            schema_desc
        );

        assert_version_expectation(expectation, stdout.trim());
    }
}

pub fn test_flow_pipeline_with_schema_test_cases(
    fixture_path: &str,
    schema_test_cases: Vec<SchemaTestCase>,
) {
    for test_case in schema_test_cases {
        test_flow_pipeline_with_fixture_and_schema(
            fixture_path,
            test_case.name,
            &test_case.semver_expectation,
            &test_case.pep440_expectation,
        );
    }
}
