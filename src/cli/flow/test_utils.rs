// Test utilities for flow pipeline tests

use crate::cli::flow::args::FlowArgs;
use crate::cli::flow::pipeline::run_flow_pipeline;
use crate::cli::utils::template::{
    Template,
    TemplateExtGeneric,
};
use crate::schema::schema_preset_names::*;
use crate::test_utils::{
    GitRepoFixture,
    assert_version_expectation,
};
use crate::version::pep440::utils::pre_release_label_to_pep440_string;
use crate::version::zerv::PreReleaseLabel;
use crate::{
    test_debug,
    test_info,
};

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
}

impl FlowTestScenario {
    /// Create an empty git repository without any tags
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let fixture = GitRepoFixture::empty()
            .map_err(|e| format!("Failed to create empty git fixture: {}", e))?;

        Ok(Self { fixture })
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
        test_flow_pipeline_with_fixture(&self.test_dir_path(), semver, pep440);
        self
    }

    pub fn expect_schema_variants(self, test_cases: Vec<SchemaTestCase>) -> Self {
        test_info!("Testing {} schema variants", test_cases.len());
        test_flow_pipeline_with_schema_test_cases(&self.test_dir_path(), test_cases);
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
        // Create a unique file with timestamp to ensure it's always a new change
        use std::time::{
            SystemTime,
            UNIX_EPOCH,
        };
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let filename = format!("commit_file_{}.txt", timestamp);
        let content = format!("Test commit content at timestamp {}", timestamp);

        self.fixture
            .test_dir
            .create_file(&filename, &content)
            .unwrap_or_else(|e| panic!("Failed to create file for commit: {}", e));
        self.fixture
            .git_impl
            .create_commit(&self.fixture.test_dir, "Test commit")
            .unwrap_or_else(|e| panic!("Failed to create commit: {}", e));
        self
    }

    pub fn make_dirty(self) -> Self {
        test_info!("Making working directory dirty");
        self.fixture.make_dirty().expect("Failed to make dirty");
        self
    }

    pub fn merge_branch(self, branch_name: &str) -> Self {
        test_info!("Merging branch: {}", branch_name);
        self.fixture
            .git_impl
            .merge_branch(&self.fixture.test_dir, branch_name)
            .unwrap_or_else(|e| panic!("Failed to merge branch '{}': {}", branch_name, e));
        self
    }

    pub fn test_dir_path(&self) -> String {
        self.fixture.path().to_string_lossy().to_string()
    }

    pub fn debug_git_state(self, context: &str) -> Self {
        crate::test_info!("=== DEBUG: {} ===", context);
        let test_dir_path = self.test_dir_path();
        crate::test_info!("Test directory: {}", test_dir_path);
        crate::test_info!(
            "You can investigate with: cd {} && git log --oneline --graph --all",
            test_dir_path
        );

        // Current branch and HEAD info
        match self
            .fixture
            .git_impl
            .execute_git(&self.fixture.test_dir, &["branch", "--show-current"])
        {
            Ok(output) => {
                crate::test_info!("Current branch: {}", output.trim());
            }
            Err(e) => {
                crate::test_info!("Git: Failed to get current branch: {}", e);
            }
        }

        match self
            .fixture
            .git_impl
            .execute_git(&self.fixture.test_dir, &["rev-parse", "HEAD"])
        {
            Ok(output) => {
                crate::test_info!("HEAD commit: {}", output.trim());
            }
            Err(e) => {
                crate::test_info!("Git: Failed to get HEAD: {}", e);
            }
        }

        // Tags on current commit
        match self
            .fixture
            .git_impl
            .execute_git(&self.fixture.test_dir, &["tag", "--points-at", "HEAD"])
        {
            Ok(output) => {
                if output.trim().is_empty() {
                    crate::test_info!("Tags on HEAD: None");
                } else {
                    crate::test_info!("Tags on HEAD: {}", output.trim());
                }
            }
            Err(e) => {
                crate::test_info!("Git: Failed to get tags on HEAD: {}", e);
            }
        }

        // All tags in repo
        match self.fixture.git_impl.execute_git(
            &self.fixture.test_dir,
            &["tag", "--list", "-n", "--sort=-version:refname"],
        ) {
            Ok(output) => {
                crate::test_info!("All tags (sorted):");
                for line in output.lines().take(10) {
                    crate::test_info!("Tag: {}", line);
                }
            }
            Err(e) => {
                crate::test_info!("Git: Failed to get tag list: {}", e);
            }
        }

        // Recent commits with tags
        match self.fixture.git_impl.execute_git(
            &self.fixture.test_dir,
            &["log", "--oneline", "--graph", "--all", "--decorate", "-10"],
        ) {
            Ok(output) => {
                crate::test_info!("Recent commits with decorations:");
                for line in output.lines().take(20) {
                    crate::test_info!("Commit: {}", line);
                }
            }
            Err(e) => {
                crate::test_info!("Git: Failed to get log: {}", e);
            }
        }

        // Describe current commit
        match self.fixture.git_impl.execute_git(
            &self.fixture.test_dir,
            &["describe", "--tags", "--always", "--abbrev=7"],
        ) {
            Ok(output) => {
                crate::test_info!("Git describe: {}", output.trim());
            }
            Err(e) => {
                crate::test_info!("Git: Failed to describe: {}", e);
            }
        }

        crate::test_info!("=== END DEBUG ===");
        self
    }

    /// Copy test directory to .cache/tmp for debugging
    pub fn copy_test_path_to_cache(self, context: &str) -> Self {
        let test_dir_path = self.test_dir_path();
        let cache_dir = std::path::Path::new(".cache/tmp");

        // Create cache directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(cache_dir) {
            crate::test_info!("Failed to create cache directory: {}", e);
            return self;
        }

        // Create unique subdirectory for this debug session
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let target_dir = cache_dir.join(format!("{}-{}", context, timestamp));

        // Copy the test directory
        match std::fs::create_dir_all(&target_dir) {
            Ok(_) => {
                // Use cp command for recursive copy
                match std::process::Command::new("cp")
                    .arg("-r")
                    .arg(&test_dir_path)
                    .arg(&target_dir)
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            crate::test_info!("Copied test directory to: {}", target_dir.display());
                            crate::test_info!(
                                "You can investigate with: cd {}",
                                target_dir.display()
                            );
                        } else {
                            crate::test_info!(
                                "Failed to copy directory: {}",
                                String::from_utf8_lossy(&output.stderr)
                            );
                        }
                    }
                    Err(e) => {
                        crate::test_info!("Failed to run cp command: {}", e);
                    }
                }
            }
            Err(e) => {
                crate::test_info!("Failed to create target directory: {}", e);
            }
        }
        self
    }
}

/// Creates comprehensive test cases for ALL standard-related schema constants
pub fn create_full_schema_test_cases(
    base_version: &str,
    pre_release_label: PreReleaseLabel,
    pre_release_num: &str,
    post: u32,
    dev: Option<&str>,
    sanitized_branch_name: &str,
    distance: u32,
) -> Vec<SchemaTestCase> {
    let semver_label = pre_release_label.label_str();
    let pep440_label = pre_release_label_to_pep440_string(&pre_release_label);

    let semver_dev = match dev {
        Some(dev_str) => format!(".dev.{}", dev_str),
        None => String::new(),
    };
    let pep440_dev = match dev {
        Some(dev_str) => format!(".dev{}", dev_str),
        None => String::new(),
    };

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
                "{}-{}.{}.post.{}{}",
                base_version, semver_label, pre_release_num, post, semver_dev
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}",
                base_version, pep440_label, pre_release_num, post, pep440_dev
            ),
        },
        // Context schemas
        SchemaTestCase {
            name: STANDARD_BASE_CONTEXT,
            semver_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, sanitized_branch_name, distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}+{}.{}.g{{hex:7}}",
                base_version, semver_label, pre_release_num, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}{}{}+{}.{}.g{{hex:7}}",
                base_version, pep440_label, pre_release_num, sanitized_branch_name, distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}+{}.{}.g{{hex:7}}",
                base_version, semver_label, pre_release_num, post, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}+{}.{}.g{{hex:7}}",
                base_version, pep440_label, pre_release_num, post, sanitized_branch_name, distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                pre_release_num,
                post,
                semver_dev,
                sanitized_branch_name,
                distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                pre_release_num,
                post,
                pep440_dev,
                sanitized_branch_name,
                distance
            ),
        },
        // Complete schemas
        SchemaTestCase {
            name: STANDARD_NO_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}",
                base_version, semver_label, pre_release_num, post, semver_dev
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}",
                base_version, pep440_label, pre_release_num, post, pep440_dev
            ),
        },
        SchemaTestCase {
            name: STANDARD_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                pre_release_num,
                post,
                semver_dev,
                sanitized_branch_name,
                distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                pre_release_num,
                post,
                pep440_dev,
                sanitized_branch_name,
                distance
            ),
        },
        SchemaTestCase {
            name: STANDARD,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                pre_release_num,
                post,
                semver_dev,
                sanitized_branch_name,
                distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                pre_release_num,
                post,
                pep440_dev,
                sanitized_branch_name,
                distance
            ),
        },
    ]
}

pub fn create_base_schema_test_cases(
    base_version: &str,
    sanitized_branch_name: &str,
) -> Vec<SchemaTestCase> {
    let distance = 0;
    let mut test_cases = Vec::new();

    let no_context_schemas = vec![
        STANDARD,
        STANDARD_BASE,
        STANDARD_BASE_PRERELEASE,
        STANDARD_BASE_PRERELEASE_POST,
        STANDARD_BASE_PRERELEASE_POST_DEV,
        STANDARD_NO_CONTEXT,
    ];

    let context_schemas = vec![
        STANDARD_BASE_CONTEXT,
        STANDARD_BASE_PRERELEASE_CONTEXT,
        STANDARD_BASE_PRERELEASE_POST_CONTEXT,
        STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT,
        STANDARD_CONTEXT,
    ];

    // Test no context schemas with base version only
    for schema_name in &no_context_schemas {
        test_cases.push(SchemaTestCase {
            name: schema_name,
            semver_expectation: base_version.to_string(),
            pep440_expectation: base_version.to_string(),
        });
    }

    // Test context schemas with context
    for schema_name in &context_schemas {
        test_cases.push(SchemaTestCase {
            name: schema_name,
            semver_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, sanitized_branch_name, distance
            ),
        });
    }

    test_cases
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
        test_debug!("Flow pipeline output ({}): {}", log_msg, output);
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
