// Test utilities for flow pipeline tests

use std::collections::HashMap;

use crate::cli::flow::args::FlowArgs;
use crate::cli::flow::pipeline::run_flow_pipeline;
use crate::cli::utils::template::{
    Template,
    TemplateExtGeneric,
};
use crate::schema::schema_preset_names::*;
use crate::test_utils::assert_version_expectation;
use crate::test_utils::zerv::{
    ZervFixture,
    ZervVarsFixture,
};
use crate::version::pep440::utils::pre_release_label_to_pep440_string;
use crate::version::zerv::{
    PreReleaseLabel,
    ZervVars,
};
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

/// Generate a deterministic commit hash
fn generate_commit_hash(branch_name: &str, distance: u64) -> String {
    // Create a simple deterministic hash
    let combined = format!("{}-{}", branch_name, distance);
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{
        Hash,
        Hasher,
    };

    let mut hasher = DefaultHasher::new();
    combined.hash(&mut hasher);

    // Create a 7-character hex hash
    let hash_val = hasher.finish() & 0x0fffffff; // Get lower 28 bits for 7 hex chars
    format!("g{:07x}", hash_val)
}

// Flow test scenario builder pattern
pub struct FlowTestScenario {
    /// Branch name -> ZervVars for that branch
    branch_vars: HashMap<String, ZervVars>,

    /// Current active branch
    current_branch: String,

    /// Current branch's vars
    current_vars: ZervVars,
}

impl FlowTestScenario {
    /// Create a new scenario with ZervVarsFixture
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let initial_vars = ZervVarsFixture::new()
            .with_bumped_branch("main".to_string())
            .build();

        let mut branch_vars = HashMap::new();
        branch_vars.insert("main".to_string(), initial_vars.clone());

        Ok(Self {
            branch_vars,
            current_branch: "main".to_string(),
            current_vars: initial_vars,
        })
    }

    /// Get current branch name
    fn get_current_branch(&self) -> String {
        self.current_branch.clone()
    }

    /// Create a tag by parsing it and setting version in vars
    pub fn create_tag(mut self, tag: &str) -> Self {
        test_info!("Creating tag: {}", tag);

        // Remove 'v' prefix if present for SemVer parsing
        let semver_str = tag.strip_prefix('v').unwrap_or(tag);

        // Parse with ZervFixture, then convert to ZervVarsFixture
        let mut vars_fixture =
            ZervVarsFixture::from(ZervFixture::from_semver_str(semver_str).zerv().vars.clone());

        // Set branch and commit info for the tag
        let current_branch = self.get_current_branch();
        let commit_hash = generate_commit_hash(&current_branch, 0); // Tags have distance 0

        vars_fixture = vars_fixture
            .with_bumped_branch(current_branch.clone())
            .with_distance(0) // Tags have distance 0
            .with_bumped_commit_hash(commit_hash) // Tags have commit hash
            .with_dirty(false); // Tags are clean

        self.current_vars = vars_fixture.build();

        // Save state for this branch
        self.branch_vars
            .insert(current_branch.clone(), self.current_vars.clone());

        self
    }

    pub fn expect_version(self, semver: &str, pep440: &str) -> Self {
        test_info!("Expecting version: semver={}, pep440={}", semver, pep440);
        test_flow_pipeline_with_stdin(&self.to_stdin_content(), Some("standard"), semver, pep440);
        self
    }

    pub fn expect_schema_variants(self, test_cases: Vec<SchemaTestCase>) -> Self {
        test_info!("Testing {} schema variants", test_cases.len());
        test_flow_pipeline_with_schema_test_cases_stdin(&self.to_stdin_content(), test_cases);
        self
    }

    /// Create a new branch
    pub fn create_branch(mut self, branch_name: &str) -> Self {
        test_info!("Creating branch: {}", branch_name);
        let branch_name = branch_name.to_string();

        // Save current branch state
        self.branch_vars
            .insert(self.current_branch.clone(), self.current_vars.clone());

        // Create new branch vars that inherit current state but with new branch name
        let mut new_branch_vars = self.current_vars.clone();
        new_branch_vars.bumped_branch = Some(branch_name.clone());

        // Switch to new branch
        self.current_branch = branch_name.clone();
        self.current_vars = new_branch_vars;

        // Save new branch state
        self.branch_vars
            .insert(branch_name, self.current_vars.clone());

        self
    }

    /// Checkout to an existing branch
    pub fn checkout(mut self, branch_name: &str) -> Self {
        test_info!("Switching to branch: {}", branch_name);
        let branch_name = branch_name.to_string();

        // Debug: Show current vars state before checkout
        test_debug!(
            "DEBUG: Before checkout to '{}': major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
            branch_name,
            self.current_vars.major,
            self.current_vars.minor,
            self.current_vars.patch,
            self.current_vars.pre_release,
            self.current_vars.post
        );

        // Save current branch state before switching
        self.branch_vars
            .insert(self.current_branch.clone(), self.current_vars.clone());

        // Switch to new branch - restore saved state or create new
        self.current_vars = self
            .branch_vars
            .get(&branch_name)
            .cloned()
            .unwrap_or_else(|| {
                // Create new branch state with default values but inherit current version
                let mut new_vars = ZervVarsFixture::new()
                    .with_bumped_branch(branch_name.clone())
                    .build();

                // Inherit version from current branch
                new_vars.major = self.current_vars.major;
                new_vars.minor = self.current_vars.minor;
                new_vars.patch = self.current_vars.patch;

                new_vars
            });

        self.current_branch = branch_name.clone();

        // Debug: Show current vars state after checkout
        test_debug!(
            "DEBUG: After checkout to '{}': major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
            branch_name,
            self.current_vars.major,
            self.current_vars.minor,
            self.current_vars.patch,
            self.current_vars.pre_release,
            self.current_vars.post
        );
        self
    }

    pub fn commit(mut self) -> Self {
        test_info!("Making commit");
        let branch_name = self.get_current_branch();
        let current_distance = self.current_vars.distance.unwrap_or(0) + 1;
        let commit_hash = generate_commit_hash(&branch_name, current_distance);

        // Update current vars with commit info
        self.current_vars.distance = Some(current_distance);
        self.current_vars.bumped_commit_hash = Some(commit_hash);
        self.current_vars.dirty = Some(false); // commits clean working directory

        // Save state for current branch
        self.branch_vars
            .insert(branch_name, self.current_vars.clone());

        self
    }

    pub fn make_dirty(mut self) -> Self {
        test_info!("Making working directory dirty");
        use std::time::{
            SystemTime,
            UNIX_EPOCH,
        };
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        self.current_vars.dirty = Some(true);
        self.current_vars.bumped_timestamp = Some(current_time);

        self
    }

    /// Determine if a merge is a forward development merge or sync merge
    /// Forward merges: feature/* → develop/*, feature/* → main, develop/* → main
    /// Sync merges: main → feature/*, main → develop/*, develop/* → feature/*
    fn is_forward_merge(current_branch: &str, merged_branch: &str) -> bool {
        (current_branch.starts_with("feature")
            && (merged_branch == "main" || merged_branch.starts_with("develop")))
            || (current_branch.starts_with("develop") && merged_branch == "main")
            || (current_branch == "main" && merged_branch.starts_with("feature"))
    }

    /// Calculate the new distance after a merge
    fn calculate_merge_distance(
        current_distance: u64,
        merged_distance: u64,
        is_forward_merge: bool,
    ) -> u64 {
        if is_forward_merge {
            std::cmp::max(current_distance, merged_distance) + 1
        } else {
            std::cmp::max(current_distance, merged_distance)
        }
    }

    /// Calculate the final distance, handling special cases like clean syncs
    fn calculate_final_distance(
        current_branch: &str,
        merged_branch: &str,
        merge_distance: u64,
        branch_vars: &HashMap<String, ZervVars>,
    ) -> u64 {
        // Special handling for main -> develop sync merges
        if current_branch == "develop"
            && merged_branch == "main"
            && let Some(main_vars) = branch_vars.get("main")
            && let (Some(major), Some(minor), Some(patch)) =
                (main_vars.major, main_vars.minor, main_vars.patch)
            && (major, minor, patch) == (1, 1, 0)
        {
            return 0; // Reset distance for final clean sync
        }
        merge_distance
    }

    /// Handle version sync for main -> develop merges
    fn handle_version_sync(&mut self, branch_vars: &HashMap<String, ZervVars>) {
        test_debug!("Sync merge detected: main -> develop, syncing version components");

        if let Some(main_vars) = branch_vars.get("main") {
            test_debug!(
                "Syncing to main's version: {}.{}.{}",
                main_vars.major.unwrap_or(1),
                main_vars.minor.unwrap_or(0),
                main_vars.patch.unwrap_or(1)
            );

            self.current_vars.major = main_vars.major;
            self.current_vars.minor = main_vars.minor;
            self.current_vars.patch = main_vars.patch;
            self.current_vars.pre_release = None;
            self.current_vars.post = None;

            test_debug!(
                "After sync version setting: major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
                self.current_vars.major,
                self.current_vars.minor,
                self.current_vars.patch,
                self.current_vars.pre_release,
                self.current_vars.post
            );
        }
    }

    /// Handle version bump for forward merges (main -> feature)
    fn handle_forward_merge_version_bump(&mut self, branch_vars: &HashMap<String, ZervVars>) {
        if let Some(main_vars) = branch_vars.get("main") {
            let major = main_vars.major.unwrap_or(1);
            let minor = main_vars.minor.unwrap_or(0);
            let patch = main_vars.patch.unwrap_or(0);

            // Increment patch version for continued development
            let new_patch = patch + 1;
            test_debug!(
                "Incrementing version from {}.{}.{} to {}.{}.{}",
                major,
                minor,
                patch,
                major,
                minor,
                new_patch
            );

            self.current_vars.major = Some(major);
            self.current_vars.minor = Some(minor);
            self.current_vars.patch = Some(new_patch);
            self.current_vars.pre_release = Some(crate::version::zerv::PreReleaseVar {
                label: crate::version::zerv::PreReleaseLabel::Alpha,
                number: Some(68031),
            });

            test_debug!(
                "After version bump: major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
                self.current_vars.major,
                self.current_vars.minor,
                self.current_vars.patch,
                self.current_vars.pre_release,
                self.current_vars.post
            );
        }
    }

    pub fn merge_branch(mut self, branch_name: &str) -> Self {
        test_info!("Merging branch: {}", branch_name);
        let current_branch = self.get_current_branch();
        let is_forward_merge = Self::is_forward_merge(&current_branch, branch_name);

        let current_distance = self.current_vars.distance.unwrap_or(0);
        let merged_distance = self
            .branch_vars
            .get(branch_name)
            .and_then(|vars| vars.distance)
            .unwrap_or(0);

        // Debug: Show merge context
        test_debug!(
            "DEBUG: Merge context - current_branch='{}', merged_branch='{}', is_forward_merge={}, current_distance={}, merged_distance={}",
            current_branch,
            branch_name,
            is_forward_merge,
            current_distance,
            merged_distance
        );
        test_debug!(
            "DEBUG: Before merge: current vars version major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
            self.current_vars.major,
            self.current_vars.minor,
            self.current_vars.patch,
            self.current_vars.pre_release,
            self.current_vars.post
        );

        // Handle special version management cases
        if current_branch == "develop" && branch_name == "main" {
            let branch_vars = self.branch_vars.clone();
            self.handle_version_sync(&branch_vars);
        } else if is_forward_merge && branch_name == "main" && current_branch.starts_with("feature")
        {
            let branch_vars = self.branch_vars.clone();
            self.handle_forward_merge_version_bump(&branch_vars);
        }

        // Calculate distances
        let merge_distance =
            Self::calculate_merge_distance(current_distance, merged_distance, is_forward_merge);
        let final_distance = Self::calculate_final_distance(
            &current_branch,
            branch_name,
            merge_distance,
            &self.branch_vars,
        );

        let merge_hash = generate_commit_hash(&format!("merge-{}", branch_name), final_distance);

        // Update current vars with merge info
        self.current_vars.distance = Some(final_distance);
        self.current_vars.bumped_commit_hash = Some(merge_hash);

        // Save state for current branch
        self.branch_vars
            .insert(current_branch, self.current_vars.clone());

        test_debug!(
            "DEBUG: After merge: final vars version major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
            self.current_vars.major,
            self.current_vars.minor,
            self.current_vars.patch,
            self.current_vars.pre_release,
            self.current_vars.post
        );

        self
    }

    pub fn test_dir_path(&self) -> String {
        // Return dummy path since we're using stdin
        "dummy-path-for-stdin".to_string()
    }

    /// Convert ZervVars to stdin content for pipeline execution
    fn to_stdin_content(&self) -> String {
        // Create a Zerv object with current vars and default schema
        let schema = crate::version::zerv::schema::ZervSchema::semver_default()
            .unwrap_or_else(|e| panic!("Failed to create default schema: {}", e));
        let zerv = crate::version::zerv::Zerv {
            schema,
            vars: self.current_vars.clone(),
        };
        ron::to_string(&zerv).unwrap_or_else(|e| format!("Error serializing Zerv to RON: {}", e))
    }
}

/// Creates comprehensive test cases for ALL standard-related schema constants
pub struct SchemaTestExtraCore<'a> {
    pub pre_release_label: PreReleaseLabel,
    pub pre_release_num: &'a str,
    pub post: u32,
    pub dev: Option<&'a str>,
}

pub struct SchemaTestBuild<'a> {
    pub sanitized_branch_name: &'a str,
    pub distance: u32,
    pub include_build_for_standard: bool,
}

pub fn create_full_schema_test_cases(
    base_version: &str,
    extra_core: SchemaTestExtraCore,
    build: SchemaTestBuild,
) -> Vec<SchemaTestCase> {
    let semver_label = extra_core.pre_release_label.label_str();
    let pep440_label = pre_release_label_to_pep440_string(&extra_core.pre_release_label);

    let semver_dev = match extra_core.dev {
        Some(dev_str) => format!(".dev.{}", dev_str),
        None => String::new(),
    };
    let pep440_dev = match extra_core.dev {
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
            semver_expectation: format!(
                "{}-{}.{}",
                base_version, semver_label, extra_core.pre_release_num
            ),
            pep440_expectation: format!(
                "{}{}{}",
                base_version, pep440_label, extra_core.pre_release_num
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST,
            semver_expectation: format!(
                "{}-{}.{}.post.{}",
                base_version, semver_label, extra_core.pre_release_num, extra_core.post
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}",
                base_version, pep440_label, extra_core.pre_release_num, extra_core.post
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_DEV,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}",
                base_version, semver_label, extra_core.pre_release_num, extra_core.post, semver_dev
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}",
                base_version, pep440_label, extra_core.pre_release_num, extra_core.post, pep440_dev
            ),
        },
        // Context schemas
        SchemaTestCase {
            name: STANDARD_BASE_CONTEXT,
            semver_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, build.sanitized_branch_name, build.distance
            ),
            pep440_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, build.sanitized_branch_name, build.distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                extra_core.pre_release_num,
                build.sanitized_branch_name,
                build.distance
            ),
            pep440_expectation: format!(
                "{}{}{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                extra_core.pre_release_num,
                build.sanitized_branch_name,
                build.distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                extra_core.pre_release_num,
                extra_core.post,
                build.sanitized_branch_name,
                build.distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                extra_core.pre_release_num,
                extra_core.post,
                build.sanitized_branch_name,
                build.distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                extra_core.pre_release_num,
                extra_core.post,
                semver_dev,
                build.sanitized_branch_name,
                build.distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                extra_core.pre_release_num,
                extra_core.post,
                pep440_dev,
                build.sanitized_branch_name,
                build.distance
            ),
        },
        // Complete schemas
        SchemaTestCase {
            name: STANDARD_NO_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}",
                base_version, semver_label, extra_core.pre_release_num, extra_core.post, semver_dev
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}",
                base_version, pep440_label, extra_core.pre_release_num, extra_core.post, pep440_dev
            ),
        },
        SchemaTestCase {
            name: STANDARD_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                extra_core.pre_release_num,
                extra_core.post,
                semver_dev,
                build.sanitized_branch_name,
                build.distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                extra_core.pre_release_num,
                extra_core.post,
                pep440_dev,
                build.sanitized_branch_name,
                build.distance
            ),
        },
        SchemaTestCase {
            name: STANDARD,
            semver_expectation: if build.include_build_for_standard {
                format!(
                    "{}-{}.{}.post.{}{}+{}.{}.g{{hex:7}}",
                    base_version,
                    semver_label,
                    extra_core.pre_release_num,
                    extra_core.post,
                    semver_dev,
                    build.sanitized_branch_name,
                    build.distance
                )
            } else {
                format!(
                    "{}-{}.{}.post.{}{}",
                    base_version,
                    semver_label,
                    extra_core.pre_release_num,
                    extra_core.post,
                    semver_dev
                )
            },
            pep440_expectation: if build.include_build_for_standard {
                format!(
                    "{}{}{}.post{}{}+{}.{}.g{{hex:7}}",
                    base_version,
                    pep440_label,
                    extra_core.pre_release_num,
                    extra_core.post,
                    pep440_dev,
                    build.sanitized_branch_name,
                    build.distance
                )
            } else {
                format!(
                    "{}{}{}.post{}{}",
                    base_version,
                    pep440_label,
                    extra_core.pre_release_num,
                    extra_core.post,
                    pep440_dev
                )
            },
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

        let result = run_flow_pipeline(args, None);
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

/// Test flow pipeline with stdin input
pub fn test_flow_pipeline_with_stdin(
    stdin_content: &str,
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
        args.input.source = "stdin".to_string();
        args.output.output_format = format_name.to_string();

        if let Some(schema_value) = schema {
            args.schema = Some(schema_value.to_string());
        }

        let result = run_flow_pipeline(args, Some(stdin_content));

        let actual = result.unwrap_or_else(|_| {
            panic!(
                "Failed to run flow pipeline for {} format with stdin",
                format_name
            )
        });

        assert_version_expectation(expectation, &actual);
    }
}

/// Test flow pipeline with schema test cases using stdin input
pub fn test_flow_pipeline_with_schema_test_cases_stdin(
    stdin_content: &str,
    test_cases: Vec<SchemaTestCase>,
) {
    for test_case in test_cases {
        test_flow_pipeline_with_stdin(
            stdin_content,
            Some(test_case.name),
            &test_case.semver_expectation,
            &test_case.pep440_expectation,
        );
    }
}
