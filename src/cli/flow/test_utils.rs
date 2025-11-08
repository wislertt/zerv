// Test utilities for flow pipeline tests

use crate::cli::flow::args::FlowArgs;
use crate::cli::flow::pipeline::run_flow_pipeline;
use crate::cli::utils::template::{
    Template,
    TemplateExtGeneric,
};
use crate::schema::schema_preset_names::*;
use crate::test_info;
use crate::test_utils::{
    GitRepoFixture,
    assert_version_expectation,
};
use crate::version::pep440::utils::pre_release_label_to_pep440_string;
use crate::version::zerv::PreReleaseLabel;

/// Generates a branch hash and asserts it matches the expected value
pub fn expect_branch_hash(branch_name: &str, length: usize, expected_hash: &str) -> String {
    let hash = Template::<u32>::new(format!(
        "{{{{ hash_int(value='{}', length={}) }}}}",
        branch_name, length
    ))
    .render_unwrap(None);
    let hash_str = hash.to_string();
    assert_eq!(
        hash_str, expected_hash,
        "Hash generation for branch '{}' with length {} failed",
        branch_name, length
    );
    hash_str
}

// Test case structure for better readability and type safety
#[derive(Debug, Clone)]
pub struct SchemaTestCase {
    pub name: &'static str,
    pub semver_expectation: String,
    pub pep440_expectation: String,
}

// Flow test scenario builder pattern
pub struct FlowTestScenario {
    fixture: GitRepoFixture,
    fixture_path: String,
}

impl FlowTestScenario {
    /// Create an empty git repository without any tags
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let fixture = GitRepoFixture::empty()
            .map_err(|e| format!("Failed to create empty git fixture: {}", e))?;
        let fixture_path = fixture.path().to_string_lossy().to_string();

        Ok(Self {
            fixture,
            fixture_path,
        })
    }

    /// Create a tag in the current git repository
    pub fn create_tag(self, tag: &str) -> Self {
        test_info!("Creating tag: {}", tag);
        self.fixture
            .git_impl
            .create_tag(&self.fixture.test_dir, tag)
            .unwrap_or_else(|e| panic!("Failed to create tag '{}': {}", tag, e));
        self
    }

    pub fn expect_version(self, semver: &str, pep440: &str) -> Self {
        test_info!("Expecting version: semver={}, pep440={}", semver, pep440);
        test_flow_pipeline_with_fixture(&self.fixture_path, semver, pep440);
        self
    }

    pub fn expect_schema_variants(self, test_cases: Vec<SchemaTestCase>) -> Self {
        test_info!("Testing {} schema variants", test_cases.len());
        test_flow_pipeline_with_schema_test_cases(&self.fixture_path, test_cases);
        self
    }

    /// Create a new branch without checking it out
    pub fn create_branch(self, branch_name: &str) -> Self {
        test_info!("Creating branch: {}", branch_name);
        self.fixture
            .create_branch(branch_name)
            .unwrap_or_else(|e| panic!("Failed to create branch '{}': {}", branch_name, e));
        self
    }

    /// Checkout to an existing branch
    pub fn checkout(self, branch_name: &str) -> Self {
        test_info!("Switching to branch: {}", branch_name);
        self.fixture
            .checkout_branch(branch_name)
            .unwrap_or_else(|e| panic!("Failed to checkout branch '{}': {}", branch_name, e));
        self
    }

    pub fn commit(self) -> Self {
        test_info!("Making commit");
        // Note: GitRepoFixture doesn't have commit_empty method
        // This would need to be implemented or use the Git operations trait
        self
    }

    pub fn make_dirty(self) -> Self {
        test_info!("Making working directory dirty");
        self.fixture.make_dirty().expect("Failed to make dirty");
        self
    }
}

/// Creates comprehensive test cases for ALL standard-related schema constants
pub fn create_all_standard_schema_test_cases(
    base_version: &str,
    pre_release_label: PreReleaseLabel,
    pre_release_num: &str,
    post: u32,
    sanitized_branch_name: &str,
    distance: u32,
) -> Vec<SchemaTestCase> {
    let semver_label = pre_release_label.label_str();
    let pep440_label = pre_release_label_to_pep440_string(&pre_release_label);

    vec![
        // Base schemas
        SchemaTestCase {
            name: STANDARD_BASE,
            semver_expectation: base_version.to_string(),
            pep440_expectation: base_version.to_string(),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE,
            semver_expectation: format!("{}-{}.{}", base_version, semver_label, pre_release_num),
            pep440_expectation: format!("{}{}{}", base_version, pep440_label, pre_release_num),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST,
            semver_expectation: format!(
                "{}-{}.{}.post.{}",
                base_version, semver_label, pre_release_num, post
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}",
                base_version, pep440_label, pre_release_num, post
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_DEV,
            semver_expectation: format!(
                "{}-{}.{}.post.{}.dev.{{timestamp:now}}",
                base_version, semver_label, pre_release_num, post
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}.dev{{timestamp:now}}",
                base_version, pep440_label, pre_release_num, post
            ),
        },
        // Context schemas
        SchemaTestCase {
            name: STANDARD_BASE_CONTEXT,
            semver_expectation: format!(
                "{}+{}.{}.{{hex:7}}",
                base_version, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}+{}.{}.{{hex:7}}",
                base_version, sanitized_branch_name, distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}+{}.{}.{{hex:7}}",
                base_version, semver_label, pre_release_num, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}{}{}+{}.{}.{{hex:7}}",
                base_version, pep440_label, pre_release_num, sanitized_branch_name, distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}+{}.{}.{{hex:7}}",
                base_version, semver_label, pre_release_num, post, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}+{}.{}.{{hex:7}}",
                base_version, pep440_label, pre_release_num, post, sanitized_branch_name, distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}.dev.{{timestamp:now}}+{}.{}.{{hex:7}}",
                base_version, semver_label, pre_release_num, post, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}.dev{{timestamp:now}}+{}.{}.{{hex:7}}",
                base_version, pep440_label, pre_release_num, post, sanitized_branch_name, distance
            ),
        },
        // Complete schemas
        SchemaTestCase {
            name: STANDARD_NO_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}.dev.{{timestamp:now}}",
                base_version, semver_label, pre_release_num, post
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}.dev{{timestamp:now}}",
                base_version, pep440_label, pre_release_num, post
            ),
        },
        SchemaTestCase {
            name: STANDARD_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}.dev.{{timestamp:now}}+{}.{}.{{hex:7}}",
                base_version, semver_label, pre_release_num, post, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}.dev{{timestamp:now}}+{}.{}.{{hex:7}}",
                base_version, pep440_label, pre_release_num, post, sanitized_branch_name, distance
            ),
        },
        SchemaTestCase {
            name: STANDARD,
            semver_expectation: format!(
                "{}-{}.{}.post.{}.dev.{{timestamp:now}}+{}.{}.{{hex:7}}",
                base_version, semver_label, pre_release_num, post, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}.dev{{timestamp:now}}+{}.{}.{{hex:7}}",
                base_version, pep440_label, pre_release_num, post, sanitized_branch_name, distance
            ),
        },
    ]
}

// Existing test utility functions (moved from pipeline.rs)
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
        let mut args = FlowArgs::default();
        args.input.directory = Some(fixture_path.to_string());
        args.output.output_format = format_name.to_string();

        // Set schema if provided
        if let Some(schema_value) = schema {
            args.schema = Some(schema_value.to_string());
        }

        let result = run_flow_pipeline(args);
        let schema_desc = match schema {
            Some(s) => format!(" and {} schema", s),
            None => String::new(),
        };

        assert!(
            result.is_ok(),
            "Flow pipeline should succeed with {} format{} at {}: {}",
            format_name,
            schema_desc,
            fixture_path,
            result.unwrap_err()
        );

        let output = result.unwrap();
        assert!(
            !output.is_empty(),
            "Flow pipeline should produce output for {} format{}",
            format_name,
            schema_desc
        );

        assert_version_expectation(expectation, &output);

        let log_msg = match schema {
            Some(s) => format!("{} with {} schema", format_name, s),
            None => format_name.to_string(),
        };
        test_info!("Flow pipeline output ({}): {}", log_msg, output);
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
