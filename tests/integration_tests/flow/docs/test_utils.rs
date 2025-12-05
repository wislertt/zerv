// Test utilities for flow documentation tests
// This module provides utilities for testing documentation examples using TestCommand

use std::collections::HashMap;

use zerv::schema::ZervSchemaPreset;
use zerv::test_utils::{
    ZervFixture,
    ZervVarsFixture,
    assert_version_expectation,
};
use zerv::version::zerv::{
    Zerv,
    ZervVars,
};

use crate::integration_tests::util::command::TestCommand;

/// Documentation test scenario with CLI command execution using TestCommand
/// Similar to FlowTestScenario but uses actual CLI execution instead of pipeline
pub struct TestScenario {
    /// Branch name -> ZervVars for that branch
    branch_vars: HashMap<String, ZervVars>,

    /// Current active branch
    current_branch: String,

    /// Current branch's vars
    current_vars: ZervVars,
}

impl TestScenario {
    /// Create a new test scenario with ZervVarsFixture
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

    /// Generate a deterministic commit hash
    fn generate_commit_hash(branch_name: &str, distance: u64) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{
            Hash,
            Hasher,
        };

        // Generate multiple hash values to create a 20-byte (40 hex char) hash
        let mut hash_bytes = Vec::new();

        // Hash different combinations to get more entropy
        for i in 0..5 {
            let mut hasher = DefaultHasher::new();
            format!("{}-{}-{}", branch_name, distance, i).hash(&mut hasher);
            let hash_val = hasher.finish();
            hash_bytes.extend_from_slice(&hash_val.to_le_bytes());
        }

        // Take first 20 bytes (160 bits) which gives 40 hex chars
        let hash_20_bytes: Vec<u8> = hash_bytes.into_iter().take(20).collect();

        // Convert to hex string
        let mut hex_string = String::with_capacity(40);
        for byte in hash_20_bytes {
            hex_string.push_str(&format!("{:02x}", byte));
        }

        format!("g{}", hex_string)
    }

    /// Create a tag by parsing it and setting version in vars
    pub fn create_tag(mut self, tag: &str) -> Self {
        // Remove 'v' prefix if present for SemVer parsing
        let semver_str = tag.strip_prefix('v').unwrap_or(tag);

        // Create ZervFixture for the tag version
        let zerv_fixture = ZervFixture::from_semver_str(semver_str);
        let zerv = zerv_fixture.zerv();

        // Convert to ZervVarsFixture
        let mut vars_fixture = ZervVarsFixture::from(zerv.vars.clone());

        // Set branch and commit info for the tag
        let current_branch = self.get_current_branch();
        let commit_hash = Self::generate_commit_hash(&current_branch, 0); // Tags have distance 0
        let last_commit_hash = Self::generate_commit_hash(&current_branch, 0); // Same commit hash for tag
        let current_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        vars_fixture = vars_fixture
            .with_bumped_branch(current_branch.clone())
            // NOTE: Tags should NOT have distance set - distance:None for clean releases
            .with_bumped_commit_hash(commit_hash) // Tags have commit hash
            .with_last_commit_hash(last_commit_hash) // Tag commit hash
            .with_last_timestamp(current_timestamp) // Tag timestamp
            .with_dirty(false); // Tags are clean

        self.current_vars = vars_fixture.build();

        // Save state for this branch
        self.branch_vars
            .insert(current_branch.clone(), self.current_vars.clone());

        self
    }

    /// Create a new branch
    pub fn create_branch(mut self, branch_name: &str) -> Self {
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
        let branch_name = branch_name.to_string();

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

        self.current_branch = branch_name;
        self
    }

    /// Make a commit
    pub fn commit(mut self) -> Self {
        let branch_name = self.get_current_branch();
        let current_distance = self.current_vars.distance.unwrap_or(0) + 1;
        let commit_hash = Self::generate_commit_hash(&branch_name, current_distance);

        // Update current vars with commit info
        self.current_vars.distance = Some(current_distance);
        self.current_vars.bumped_commit_hash = Some(commit_hash);
        self.current_vars.dirty = Some(false); // commits clean working directory

        // Save state for current branch
        self.branch_vars
            .insert(branch_name, self.current_vars.clone());

        self
    }

    /// Make working directory dirty
    pub fn make_dirty(mut self) -> Self {
        self.current_vars.dirty = Some(true);
        self
    }

    /// Convert ZervVars to stdin content for TestCommand execution
    fn to_stdin_content(&self) -> String {
        // Create a Zerv object with standard schema
        let zerv = Zerv {
            schema: ZervSchemaPreset::Standard.schema_with_zerv(&self.current_vars),
            vars: self.current_vars.clone(),
        };
        ron::to_string(&zerv).unwrap_or_else(|e| format!("Error serializing Zerv to RON: {}", e))
    }

    /// Assert that a single CLI command produces the expected output
    /// Supports pattern matching like {hex:7}, {timestamp}, etc.
    pub fn assert_command(self, command: &str, expected_output: &str) -> Self {
        let stdin_content = self.to_stdin_content();
        let actual_output = TestCommand::run_with_stdin(command, stdin_content);

        assert_version_expectation(expected_output, &actual_output);
        self
    }

    /// Assert that a single CLI command output contains all expected substrings
    /// Used for checking key components in complex outputs like RON format
    pub fn assert_command_contains(self, command: &str, expected_substrings: &[&str]) -> Self {
        let stdin_content = self.to_stdin_content();
        let actual_output = TestCommand::run_with_stdin(command, stdin_content);

        for substring in expected_substrings {
            assert!(
                actual_output.contains(substring),
                "Expected output to contain '{}', but it did not.\nActual output:\n{}",
                substring,
                actual_output
            );
        }
        self
    }

    /// Assert that multiple CLI commands executed as a pipeline produce the expected output
    /// Executes commands sequentially, piping output of each command to stdin of the next
    pub fn assert_commands(self, commands: &[&str], expected_output: &str) -> Self {
        if commands.is_empty() {
            panic!("No commands provided to assert_commands");
        }

        // Start with stdin content from our scenario
        let mut current_output = self.to_stdin_content();

        // Execute all commands except the last one, piping output
        for (i, command) in commands.iter().enumerate() {
            if i == commands.len() - 1 {
                // Last command: execute and assert output
                let final_output = TestCommand::run_with_stdin(command, current_output.clone());
                assert_version_expectation(expected_output, &final_output);
            } else {
                // Intermediate command: execute and capture output for next command
                current_output = TestCommand::run_with_stdin(command, current_output);
            }
        }

        self
    }
}
