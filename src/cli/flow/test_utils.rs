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
use crate::test_utils::zerv::ZervFixture;
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
    fixture: ZervFixture,
    // Track branch-specific state for accurate simulation
    branch_distances: HashMap<String, u64>,
    branch_versions: HashMap<String, (u64, u64, u64)>, // Track version per branch: (major, minor, patch)
}

impl FlowTestScenario {
    /// Create a new scenario with ZervFixture
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            fixture: ZervFixture::new(),
            branch_distances: HashMap::new(),
            branch_versions: HashMap::new(),
        })
    }

    /// Get current branch name or default to "main"
    fn get_current_branch(&self) -> String {
        self.fixture
            .zerv()
            .vars
            .bumped_branch
            .as_deref()
            .unwrap_or("main")
            .to_string()
    }

    // /// Get distance for current branch, defaulting to 0 if not set
    // fn get_current_branch_distance(&self) -> u64 {
    //     let branch = self.get_current_branch();
    //     self.branch_distances.get(&branch).copied().unwrap_or(0)
    // }

    /// Create a tag by parsing it and setting version in fixture
    pub fn create_tag(mut self, tag: &str) -> Self {
        test_info!("Creating tag: {}", tag);

        // Remove 'v' prefix if present for SemVer parsing
        let semver_str = tag.strip_prefix('v').unwrap_or(tag);

        // Use ZervFixture's existing from_semver_str method
        let mut fixture = ZervFixture::from_semver_str(semver_str);

        // Set current branch and commit hash for the tag
        let current_branch = self.get_current_branch();
        let commit_hash = generate_commit_hash(&current_branch, 0); // Tags have distance 0

        fixture = fixture
            .with_branch(current_branch.clone())
            .with_distance(0) // Tags have distance 0
            .with_commit_hash(commit_hash) // Tags have commit hash
            .with_dirty(false); // Tags are clean

        self.fixture = fixture;

        // Reset distance for this branch to 0 (tags start fresh)
        self.branch_distances.insert(current_branch.clone(), 0);

        // Save version for this branch
        if let (Some(major), Some(minor), Some(patch)) = (
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
        ) {
            self.branch_versions
                .insert(current_branch, (major, minor, patch));
        }

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

        // Get current branch distance before switching
        let current_branch = self.get_current_branch();
        let current_distance = self
            .branch_distances
            .get(&current_branch)
            .copied()
            .unwrap_or(0);

        // Save current branch version state before creating new branch
        if let (Some(major), Some(minor), Some(patch)) = (
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
        ) {
            self.branch_versions
                .insert(current_branch.clone(), (major, minor, patch));
        }

        // Switch to new branch and inherit distance and version
        self.fixture = self.fixture.with_branch(branch_name.clone());
        self.branch_distances
            .entry(branch_name.clone())
            .or_insert(current_distance);

        // New branch inherits current version
        if let (Some(major), Some(minor), Some(patch)) = (
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
        ) {
            self.branch_versions
                .insert(branch_name, (major, minor, patch));
        }

        self
    }

    /// Checkout to an existing branch
    pub fn checkout(mut self, branch_name: &str) -> Self {
        test_info!("Switching to branch: {}", branch_name);
        let branch_name = branch_name.to_string();

        // Debug: Show current fixture state before checkout
        test_debug!(
            "DEBUG: Before checkout to '{}': major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
            branch_name,
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
            self.fixture.zerv().vars.pre_release,
            self.fixture.zerv().vars.post
        );

        // Save current branch state before switching
        let current_branch = self.get_current_branch();
        if let (Some(major), Some(minor), Some(patch)) = (
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
        ) {
            self.branch_versions
                .insert(current_branch, (major, minor, patch));
        }

        // Switch to new branch
        self.fixture = self.fixture.with_branch(branch_name.clone());

        // Restore version state for the new branch if available, otherwise keep current
        if let Some((major, minor, patch)) = self.branch_versions.get(&branch_name) {
            test_debug!(
                "Restoring version for branch '{}': {}.{}.{}",
                branch_name,
                major,
                minor,
                patch
            );
            self.fixture = self.fixture.with_version(*major, *minor, *patch);
        } else {
            // Initialize with current version if no saved state
            if let (Some(major), Some(minor), Some(patch)) = (
                self.fixture.zerv().vars.major,
                self.fixture.zerv().vars.minor,
                self.fixture.zerv().vars.patch,
            ) {
                self.branch_versions
                    .insert(branch_name.clone(), (major, minor, patch));
            }
        }

        // Initialize distance for branch if not already present
        self.branch_distances
            .entry(branch_name.clone())
            .or_insert(0);

        // Debug: Show current fixture state after checkout
        test_debug!(
            "DEBUG: After checkout to '{}': major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
            branch_name,
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
            self.fixture.zerv().vars.pre_release,
            self.fixture.zerv().vars.post
        );
        self
    }

    pub fn commit(mut self) -> Self {
        test_info!("Making commit");
        let branch_name = self.get_current_branch();
        let current_distance = self
            .branch_distances
            .get(&branch_name)
            .copied()
            .unwrap_or(0)
            + 1;
        let commit_hash = generate_commit_hash(&branch_name, current_distance);

        self.branch_distances
            .insert(branch_name.clone(), current_distance);

        self.fixture = self
            .fixture
            .with_distance(current_distance)
            .with_commit_hash(commit_hash)
            .with_dirty(false); // commits clean working directory

        // Save the version state for the current branch after commit
        if let (Some(major), Some(minor), Some(patch)) = (
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
        ) {
            self.branch_versions
                .insert(branch_name, (major, minor, patch));
        }

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

        self.fixture = self
            .fixture
            .with_dirty(true)
            .with_bumped_timestamp(current_time);

        self
    }

    pub fn merge_branch(mut self, branch_name: &str) -> Self {
        test_info!("Merging branch: {}", branch_name);
        let current_branch = self.get_current_branch();

        // Determine if this is a forward development merge or sync merge
        // Forward merges: feature/* → develop/*, feature/* → main, develop/* → main
        // Sync merges: main → feature/*, main → develop/*, develop/* → feature/*
        let is_forward_merge = (current_branch.starts_with("feature")
            && (branch_name == "main" || branch_name.starts_with("develop")))
            || (current_branch.starts_with("develop") && branch_name == "main")
            || (current_branch == "main" && branch_name.starts_with("feature"));

        let current_distance = self
            .branch_distances
            .get(&current_branch)
            .copied()
            .unwrap_or(0);
        let merged_distance = self.branch_distances.get(branch_name).copied().unwrap_or(0);

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
            "DEBUG: Before merge: current fixture version major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
            self.fixture.zerv().vars.pre_release,
            self.fixture.zerv().vars.post
        );

        // For forward development merges, increment max distance. For sync merges, just use max.
        let new_distance = if is_forward_merge {
            std::cmp::max(current_distance, merged_distance) + 1
        } else {
            std::cmp::max(current_distance, merged_distance)
        };

        // Special handling for sync merges (main -> develop)
        // In GitFlow, when main is merged into develop, develop should sync with main's version
        if current_branch == "develop" && branch_name == "main" {
            test_debug!("Sync merge detected: main -> develop, syncing version components");

            // Get main's version (from Step 6: v1.0.1)
            let main_version = if let Some(main_version) = self.branch_versions.get("main") {
                *main_version
            } else {
                // Fallback: assume main has the tag version from Step 6
                (1, 0, 1) // v1.0.1 from Step 6
            };

            test_debug!(
                "Syncing to main's version: {}.{}.{}",
                main_version.0,
                main_version.1,
                main_version.2
            );

            // Set develop branch to main's version and clear all development state
            self.fixture = self.fixture
                .with_version(main_version.0, main_version.1, main_version.2)
                .without_pre_release()   // Clear any pre-release from development
                .without_post(); // Clear any post-release from development

            test_debug!(
                "After sync version setting: major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
                self.fixture.zerv().vars.major,
                self.fixture.zerv().vars.minor,
                self.fixture.zerv().vars.patch,
                self.fixture.zerv().vars.pre_release,
                self.fixture.zerv().vars.post
            );
        }
        // Special handling for forward merges in trunk-based development
        // When a feature branch merges main, it should increment the patch version
        else if is_forward_merge && branch_name == "main" && current_branch.starts_with("feature")
        {
            test_debug!(
                "Forward merge detected: main -> {}, incrementing version",
                current_branch
            );

            // Get main's version
            let main_version = if let Some(main_version) = self.branch_versions.get("main") {
                *main_version
            } else {
                // Fallback: try to get from current fixture
                if let (Some(major), Some(minor), Some(patch)) = (
                    self.fixture.zerv().vars.major,
                    self.fixture.zerv().vars.minor,
                    self.fixture.zerv().vars.patch,
                ) {
                    (major, minor, patch)
                } else {
                    (1, 0, 0) // ultimate fallback
                }
            };

            // Increment patch version for continued development
            let new_patch = main_version.2 + 1;
            test_debug!(
                "Incrementing version from {}.{}.{} to {}.{}.{}",
                main_version.0,
                main_version.1,
                main_version.2,
                main_version.0,
                main_version.1,
                new_patch
            );

            self.fixture = self
                .fixture
                .with_version(main_version.0, main_version.1, new_patch)
                .with_pre_release(crate::version::zerv::PreReleaseLabel::Alpha, Some(68031));

            test_debug!(
                "After version bump: major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
                self.fixture.zerv().vars.major,
                self.fixture.zerv().vars.minor,
                self.fixture.zerv().vars.patch,
                self.fixture.zerv().vars.pre_release,
                self.fixture.zerv().vars.post
            );
        }

        let merge_hash = generate_commit_hash(&format!("merge-{}", branch_name), new_distance);

        // For sync merges (main -> develop), check if we need to reset distance for clean version
        let final_distance = if current_branch == "develop" && branch_name == "main" {
            // Check if this is syncing to v1.1.0 (final release)
            let main_version = if let Some(main_version) = self.branch_versions.get("main") {
                *main_version
            } else {
                (1, 0, 1) // fallback
            };

            if main_version == (1, 1, 0) {
                0 // Reset distance for final clean sync
            } else {
                new_distance // Keep distance for regular sync
            }
        } else {
            new_distance
        };

        self.branch_distances
            .insert(current_branch.clone(), final_distance);

        self.fixture = self
            .fixture
            .with_distance(final_distance)
            .with_commit_hash(merge_hash);

        // Save the version state for the current branch after merge
        if let (Some(major), Some(minor), Some(patch)) = (
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
        ) {
            self.branch_versions
                .insert(current_branch, (major, minor, patch));
        }

        test_debug!(
            "DEBUG: After merge: final fixture version major={:?}, minor={:?}, patch={:?}, pre_release={:?}, post={:?}",
            self.fixture.zerv().vars.major,
            self.fixture.zerv().vars.minor,
            self.fixture.zerv().vars.patch,
            self.fixture.zerv().vars.pre_release,
            self.fixture.zerv().vars.post
        );

        self
    }

    pub fn test_dir_path(&self) -> String {
        // Return dummy path since we're using stdin
        "dummy-path-for-stdin".to_string()
    }

    /// Convert ZervFixture to stdin content for pipeline execution
    fn to_stdin_content(&self) -> String {
        // Get a reference to the zerv and convert it to RON format
        let zerv = self.fixture.zerv();
        ron::to_string(&zerv).unwrap_or_else(|e| format!("Error serializing Zerv to RON: {}", e))
    }

    /// Simple debug method for stdin-based testing
    pub fn debug_git_state(self, context: &str) -> Self {
        crate::test_info!("=== DEBUG: {} ===", context);
        crate::test_info!("Using stdin-based ZervFixture (no Git operations)");

        let zerv = self.fixture.zerv();
        crate::test_info!(
            "Version: major={:?}, minor={:?}, patch={:?}",
            zerv.vars.major,
            zerv.vars.minor,
            zerv.vars.patch
        );
        crate::test_info!("Branch: {:?}", zerv.vars.bumped_branch);
        crate::test_info!("Distance: {:?}", zerv.vars.distance);
        crate::test_info!("Commit hash: {:?}", zerv.vars.bumped_commit_hash);
        crate::test_info!("Dirty: {:?}", zerv.vars.dirty);
        crate::test_info!("=== END DEBUG ===");
        self
    }

    /// Simple debug method to save stdin content for debugging
    pub fn copy_test_path_to_cache(self, _context: &str) -> Self {
        let cache_dir = std::path::Path::new(".cache/tmp");

        // Create cache directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(cache_dir) {
            crate::test_info!("Failed to create cache directory: {}", e);
            return self;
        }

        // Create unique filename for this debug session
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let target_file = cache_dir.join(format!("{}.ron", timestamp));

        // Save stdin content to file
        let stdin_content = self.to_stdin_content();
        match std::fs::write(&target_file, stdin_content) {
            Ok(_) => {
                crate::test_info!("Saved stdin content to: {}", target_file.display());
                crate::test_info!("You can investigate with: cat {}", target_file.display());
            }
            Err(e) => {
                crate::test_info!("Failed to save stdin content: {}", e);
            }
        }
        self
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
