use regex::Regex;

/// Test utilities for version command validation
pub struct VersionTestUtils;

impl VersionTestUtils {
    /// Assert version matches exact string
    pub fn assert_exact_version(output: &str, expected: &str, scenario: &str) {
        let version = output.trim();
        assert_eq!(
            version, expected,
            "Version should match exactly for {scenario}: got '{version}', expected '{expected}'"
        );
    }

    /// Assert version matches pattern with commit hash placeholder
    pub fn assert_version_pattern(output: &str, pattern: &str, scenario: &str) {
        let version = output.trim();

        // Convert pattern to regex by escaping special chars and replacing <commit>
        let regex_pattern = pattern
            .replace('.', "\\.")
            .replace('+', "\\+")
            .replace("<commit>", "([0-9a-f]{7})");

        let regex = Regex::new(&regex_pattern)
            .unwrap_or_else(|_| panic!("Invalid regex pattern: {regex_pattern}"));

        let captures = regex.captures(version).unwrap_or_else(|| {
            panic!("Version should match pattern '{pattern}' for {scenario}: got '{version}'")
        });

        // Validate commit hash if present
        if let Some(commit_match) = captures.get(1) {
            let commit_hash = commit_match.as_str();
            assert_eq!(
                commit_hash.len(),
                7,
                "Commit hash should be 7 characters: {commit_hash}"
            );
            assert!(
                commit_hash.chars().all(|c| c.is_ascii_hexdigit()),
                "Commit hash should be hexadecimal: {commit_hash}"
            );
        }
    }

    /// Assert version contains base version and additional components
    pub fn assert_version_components(output: &str, base_version: &str, scenario: &str) {
        let version = output.trim();

        assert!(
            version.contains(base_version),
            "Version should contain base version '{base_version}' for {scenario}: got '{version}'"
        );

        // Validate version format structure
        assert!(
            !version.is_empty(),
            "Version should not be empty for {scenario}"
        );

        // Should contain at least major.minor.patch
        let parts: Vec<&str> = version.split('.').collect();
        assert!(
            parts.len() >= 3,
            "Version should have at least 3 parts (major.minor.patch) for {scenario}: got '{version}'"
        );
    }
}
