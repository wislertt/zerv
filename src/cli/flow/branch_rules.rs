// Branch rules system for GitFlow support

use std::fmt;
use std::str::FromStr;

use ron::{
    from_str,
    to_string,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::error::ZervError;

/// Enum for type-safe pre-release labels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreReleaseLabel {
    Alpha,
    Beta,
    Rc,
}

/// Enum for type-safe post modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostMode {
    Tag,
    Commit,
}

/// Branch rule configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchRule {
    pub pattern: String,                    // "develop", "release/*", "feature/*"
    pub pre_release_label: PreReleaseLabel, // "beta", "rc", "alpha"
    #[serde(default)]
    pub pre_release_num: Option<u32>, // "1" for release branches, defaults to None
    pub post_mode: PostMode,                // "tag" for release, "commit" for others
}

/// Resolved branch arguments from branch rules
#[derive(Debug, Clone)]
pub struct ResolvedBranchArgs {
    pub pre_release_label: PreReleaseLabel,
    pub pre_release_num: Option<u32>,
    pub post_mode: PostMode,
}

/// Collection of branch rules with pattern matching
#[derive(Debug, Clone, Default)]
pub struct BranchRules {
    rules: Vec<BranchRule>,
}

impl BranchRule {
    /// Validate the branch rule configuration
    pub fn validate(&self) -> Result<(), ZervError> {
        // Wildcard patterns (ending with /*) must not have explicit pre_release_num
        if self.pattern.ends_with("/*") && self.pre_release_num.is_some() {
            return Err(ZervError::ConflictingOptions(format!(
                "Branch rule with wildcard pattern '{}' cannot have explicit pre_release_num. \
                Use None to extract number from branch name dynamically.",
                self.pattern
            )));
        }

        // Exact patterns (not ending with /*) must have pre_release_num specified
        if !self.pattern.ends_with("/*") && self.pre_release_num.is_none() {
            return Err(ZervError::ConflictingOptions(format!(
                "Branch rule with exact pattern '{}' must have explicit pre_release_num. \
                Specify pre_release_num: Some(N) or use wildcard pattern '{}/N' for dynamic extraction.",
                self.pattern, self.pattern
            )));
        }

        Ok(())
    }

    /// Resolve values for a specific branch that matches this rule's pattern
    pub fn resolve_for_branch(&self, branch_name: &str) -> ResolvedBranchArgs {
        ResolvedBranchArgs {
            pre_release_label: self.pre_release_label.clone(),
            pre_release_num: self.resolve_pre_release_num(branch_name),
            post_mode: self.post_mode.clone(),
        }
    }

    /// Resolve pre-release number for a branch that matches this rule
    fn resolve_pre_release_num(&self, branch_name: &str) -> Option<u32> {
        // 1. Use explicit number from rule
        if let Some(num) = self.pre_release_num {
            return Some(num);
        }

        // 2. Extract from branch pattern (e.g., "release/1" -> "1")
        self.extract_branch_number(branch_name)
    }

    /// Extract number from branch pattern (e.g., "release/1" -> "1" when pattern is "release/*")
    fn extract_branch_number(&self, branch_name: &str) -> Option<u32> {
        if !self.pattern.ends_with("/*") {
            return None;
        }

        let prefix = &self.pattern[..self.pattern.len() - 2]; // Remove "/*"
        if !branch_name.starts_with(prefix) || branch_name.len() == prefix.len() {
            return None;
        }

        let remainder = &branch_name[prefix.len()..];

        // Skip any non-numeric characters (like slash) at the beginning
        let numeric_start: String = remainder
            .chars()
            .skip_while(|c| !c.is_numeric())
            .take_while(|c| c.is_numeric())
            .collect();

        if numeric_start.is_empty() {
            None
        } else {
            numeric_start.parse().ok()
        }
    }
}

impl BranchRules {
    /// Create new branch rules from a vector of rules
    pub fn new(rules: Vec<BranchRule>) -> Result<Self, ZervError> {
        // Validate all rules
        for rule in &rules {
            rule.validate()?;
        }
        Ok(Self { rules })
    }

    /// Preprocess RON string to convert bare numbers to Some(number) for pre_release_num
    fn preprocess_ron_syntax(ron_str: &str) -> String {
        use regex::Regex;

        // Match pattern: pre_release_num: <number> and convert to pre_release_num: Some(<number>)
        // This regex finds pre_release_num field with bare numbers and wraps them in Some()
        let re = Regex::new(r"(pre_release_num:\s*)(\d+)").expect("Failed to compile regex");
        re.replace_all(ron_str, "${1}Some(${2})").to_string()
    }

    /// Find a rule that matches the given branch name
    pub fn find_rule(&self, branch: &str) -> Option<&BranchRule> {
        self.rules.iter().find(|rule| rule.matches(branch))
    }

    /// Get default branch rules for GitFlow
    pub fn default_rules() -> Self {
        let rules = vec![
            BranchRule {
                pattern: "develop".to_string(),
                pre_release_label: PreReleaseLabel::Beta,
                pre_release_num: Some(1),
                post_mode: PostMode::Commit,
            },
            BranchRule {
                pattern: "release/*".to_string(),
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: None, // Extract from branch name
                post_mode: PostMode::Tag,
            },
        ];
        Self::new(rules).expect("Default branch rules should be valid")
    }

    /// Find and resolve rule for a branch, or return default args
    pub fn resolve_for_branch(&self, branch_name: &str) -> ResolvedBranchArgs {
        if let Some(rule) = self.find_rule(branch_name) {
            rule.resolve_for_branch(branch_name)
        } else {
            // Default fallback for unmapped branches
            ResolvedBranchArgs {
                pre_release_label: PreReleaseLabel::Alpha,
                pre_release_num: None,
                post_mode: PostMode::Commit,
            }
        }
    }
}

impl FromStr for BranchRules {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Preprocess to convert bare numbers to Some(number)
        let processed_ron = Self::preprocess_ron_syntax(s);

        let rules: Vec<BranchRule> = from_str(&processed_ron).map_err(|e| {
            ZervError::InvalidFormat(format!("Failed to parse branch rules: {}", e))
        })?;

        // Validate all parsed rules
        for rule in &rules {
            rule.validate()?;
        }

        Ok(Self { rules })
    }
}

impl fmt::Display for BranchRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ron_string = to_string(&self.rules).map_err(|_| fmt::Error)?;
        write!(f, "{}", ron_string)
    }
}

impl BranchRule {
    /// Check if this rule matches the given branch name
    pub fn matches(&self, branch: &str) -> bool {
        if self.pattern.ends_with("/*") {
            // Wildcard pattern: "release/*" matches "release/1", "release/feature-name"
            let prefix = &self.pattern[..self.pattern.len() - 2];
            branch.starts_with(prefix) && branch.len() > prefix.len()
        } else {
            // Exact pattern match: "develop" matches only "develop"
            self.pattern == branch
        }
    }
}

// Helper implementations for string conversion
impl PreReleaseLabel {
    pub fn to_string(&self) -> &'static str {
        match self {
            PreReleaseLabel::Alpha => "alpha",
            PreReleaseLabel::Beta => "beta",
            PreReleaseLabel::Rc => "rc",
        }
    }
}

impl FromStr for PreReleaseLabel {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "alpha" => Ok(PreReleaseLabel::Alpha),
            "beta" => Ok(PreReleaseLabel::Beta),
            "rc" => Ok(PreReleaseLabel::Rc),
            _ => Err(ZervError::InvalidFormat(format!(
                "Invalid pre-release label: '{}'. Must be one of: alpha, beta, rc",
                s
            ))),
        }
    }
}

impl PostMode {
    pub fn to_string(&self) -> &'static str {
        match self {
            PostMode::Tag => "tag",
            PostMode::Commit => "commit",
        }
    }
}

impl FromStr for PostMode {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tag" => Ok(PostMode::Tag),
            "commit" => Ok(PostMode::Commit),
            _ => Err(ZervError::InvalidFormat(format!(
                "Invalid post mode: '{}'. Must be one of: tag, commit",
                s
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("alpha", PreReleaseLabel::Alpha)]
    #[case("BETA", PreReleaseLabel::Beta)]
    #[case("Rc", PreReleaseLabel::Rc)]
    #[case("rc", PreReleaseLabel::Rc)]
    #[case("ALPHA", PreReleaseLabel::Alpha)]
    #[case("beta", PreReleaseLabel::Beta)]
    fn test_pre_release_label_from_str(#[case] input: &str, #[case] expected: PreReleaseLabel) {
        assert_eq!(PreReleaseLabel::from_str(input).unwrap(), expected);
    }

    #[rstest]
    #[case("invalid")]
    #[case("")]
    #[case("release")]
    #[case("pre")]
    fn test_pre_release_label_from_str_invalid(#[case] input: &str) {
        assert!(PreReleaseLabel::from_str(input).is_err());
    }

    #[rstest]
    #[case("tag", PostMode::Tag)]
    #[case("COMMIT", PostMode::Commit)]
    #[case("Tag", PostMode::Tag)]
    #[case("commit", PostMode::Commit)]
    fn test_post_mode_from_str(#[case] input: &str, #[case] expected: PostMode) {
        assert_eq!(PostMode::from_str(input).unwrap(), expected);
    }

    #[rstest]
    #[case("invalid")]
    #[case("")]
    #[case("push")]
    #[case("merge")]
    fn test_post_mode_from_str_invalid(#[case] input: &str) {
        assert!(PostMode::from_str(input).is_err());
    }

    #[rstest]
    #[case("develop", "develop", true)]
    #[case("develop", "develop-feature", false)]
    #[case("develop", "develop/feature", false)]
    #[case("develop", "main", false)]
    #[case("main", "main", true)]
    #[case("main", "master", false)]
    #[case("master", "master", true)]
    #[case("master", "master/feature", false)]
    fn test_branch_rule_exact_match(
        #[case] pattern: &str,
        #[case] branch: &str,
        #[case] matches: bool,
    ) {
        let rule = BranchRule {
            pattern: pattern.to_string(),
            pre_release_label: PreReleaseLabel::Beta,
            pre_release_num: Some(1),
            post_mode: PostMode::Commit,
        };

        assert_eq!(rule.matches(branch), matches);
    }

    #[rstest]
    #[case("release/*", "release/1", true)]
    #[case("release/*", "release/2", true)]
    #[case("release/*", "release/feature-name", true)]
    #[case("release/*", "release", false)] // "release" alone should not match "release/*"
    #[case("release/*", "main", false)]
    #[case("hotfix/*", "hotfix/123", true)]
    #[case("hotfix/*", "hotfix/urgent-fix", true)]
    #[case("hotfix/*", "hotfix", false)]
    fn test_branch_rule_wildcard_match(
        #[case] pattern: &str,
        #[case] branch: &str,
        #[case] matches: bool,
    ) {
        let rule = BranchRule {
            pattern: pattern.to_string(),
            pre_release_label: PreReleaseLabel::Rc,
            pre_release_num: None,
            post_mode: PostMode::Tag,
        };

        assert_eq!(rule.matches(branch), matches);
    }

    #[rstest]
    #[case("release/*", "release/1", Some(1))]
    #[case("release/*", "release/42", Some(42))]
    #[case("release/*", "release/1/feature", Some(1))]
    #[case("release/*", "release/feature", None)]
    #[case("hotfix/*", "hotfix/123", Some(123))]
    #[case("hotfix/*", "hotfix/abc", None)]
    #[case("release/*", "main", None)]
    fn test_branch_rule_number_extraction(
        #[case] pattern: &str,
        #[case] branch_name: &str,
        #[case] expected: Option<u32>,
    ) {
        let rule = BranchRule {
            pattern: pattern.to_string(),
            pre_release_label: PreReleaseLabel::Rc,
            pre_release_num: None, // Must be specified in Rust code (#[serde(default)] only for deserialization)
            post_mode: PostMode::Tag,
        };

        assert_eq!(rule.resolve_pre_release_num(branch_name), expected);
    }

    #[test]
    fn test_branch_rule_explicit_number() {
        let rule = BranchRule {
            pattern: "develop".to_string(),
            pre_release_label: PreReleaseLabel::Beta,
            pre_release_num: Some(5),
            post_mode: PostMode::Commit,
        };

        // Should always use the explicit number, not extract from branch name
        assert_eq!(rule.resolve_pre_release_num("develop"), Some(5));
    }

    #[test]
    fn test_branch_rules_default() {
        let rules = BranchRules::default_rules();

        // Should have exactly 2 default rules
        assert_eq!(rules.rules.len(), 2);

        // Check develop rule
        let develop_rule = rules.find_rule("develop").unwrap();
        assert_eq!(develop_rule.pre_release_label, PreReleaseLabel::Beta);
        assert_eq!(develop_rule.pre_release_num, Some(1));
        assert_eq!(develop_rule.post_mode, PostMode::Commit);

        // Check release rule
        let release_rule = rules.find_rule("release/1").unwrap();
        assert_eq!(release_rule.pre_release_label, PreReleaseLabel::Rc);
        assert_eq!(release_rule.pre_release_num, None);
        assert_eq!(release_rule.post_mode, PostMode::Tag);
    }

    #[test]
    fn test_branch_rule_validation_wildcard_with_explicit_num() {
        let invalid_rule = BranchRule {
            pattern: "release/*".to_string(),
            pre_release_label: PreReleaseLabel::Rc,
            pre_release_num: Some(1), // This should be invalid for wildcard patterns
            post_mode: PostMode::Tag,
        };

        // Validation should fail
        let result = invalid_rule.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ZervError::ConflictingOptions(msg) => {
                assert!(msg.contains("wildcard pattern"));
                assert!(msg.contains("cannot have explicit pre_release_num"));
            }
            _ => panic!("Expected ConflictingOptions error"),
        }
    }

    #[test]
    fn test_branch_rules_new_with_validation() {
        // Valid rules should succeed
        let valid_rules = vec![
            BranchRule {
                pattern: "develop".to_string(),
                pre_release_label: PreReleaseLabel::Beta,
                pre_release_num: Some(1),
                post_mode: PostMode::Commit,
            },
            BranchRule {
                pattern: "release/*".to_string(),
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: None, // Valid: None for wildcard pattern
                post_mode: PostMode::Tag,
            },
        ];
        let result = BranchRules::new(valid_rules);
        assert!(result.is_ok());

        // Invalid rules should fail
        let invalid_rules = vec![BranchRule {
            pattern: "release/*".to_string(),
            pre_release_label: PreReleaseLabel::Rc,
            pre_release_num: Some(1), // Invalid: Some for wildcard pattern
            post_mode: PostMode::Tag,
        }];
        let result = BranchRules::new(invalid_rules);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_rules_uses_validation() {
        // This test ensures default_rules() uses the new() method with validation
        let rules = BranchRules::default_rules();

        // Should create successfully (panic if invalid)
        assert_eq!(rules.rules.len(), 2);

        // Default rules should be valid
        for rule in &rules.rules {
            assert!(rule.validate().is_ok());
        }
    }

    #[test]
    fn test_branch_rules_resolve_for_branch() {
        let rules = BranchRules::default_rules();

        // Mapped branch should use rule
        let develop_args = rules.resolve_for_branch("develop");
        assert_eq!(develop_args.pre_release_label, PreReleaseLabel::Beta);
        assert_eq!(develop_args.pre_release_num, Some(1));
        assert_eq!(develop_args.post_mode, PostMode::Commit);

        // Unmapped branch should use defaults
        let feature_args = rules.resolve_for_branch("feature/auth");
        assert_eq!(feature_args.pre_release_label, PreReleaseLabel::Alpha);
        assert_eq!(feature_args.pre_release_num, None); // FlowArgs will handle generation
        assert_eq!(feature_args.post_mode, PostMode::Commit);
    }

    #[test]
    fn test_branch_rules_from_str_simplified_syntax() {
        // Test simplified RON syntax with bare values instead of Some(N)
        let ron_str = r#"[
            (pattern: "develop", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
            (pattern: "release/*", pre_release_label: rc, post_mode: tag)
        ]"#;

        let rules: BranchRules = ron_str.parse().unwrap();
        assert_eq!(rules.rules.len(), 2);

        let develop_rule = rules.find_rule("develop").unwrap();
        assert_eq!(develop_rule.pre_release_label, PreReleaseLabel::Beta);
        assert_eq!(develop_rule.pre_release_num, Some(1)); // 1 becomes Some(1)

        let release_rule = rules.find_rule("release/1").unwrap();
        assert_eq!(release_rule.pre_release_label, PreReleaseLabel::Rc);
        assert_eq!(release_rule.pre_release_num, None); // Omitted becomes None
    }

    #[test]
    fn test_branch_rules_validation_exact_pattern_requires_num() {
        // Test that exact patterns without pre_release_num fail validation
        let ron_str = r#"[
            (pattern: "develop", pre_release_label: beta, post_mode: commit)
        ]"#;

        let result: Result<BranchRules, _> = ron_str.parse();
        assert!(result.is_err());
        match result.unwrap_err() {
            ZervError::ConflictingOptions(msg) => {
                assert!(msg.contains("exact pattern"));
                assert!(msg.contains("must have explicit pre_release_num"));
            }
            _ => panic!("Expected ConflictingOptions error"),
        }
    }

    #[rstest]
    #[case(PreReleaseLabel::Alpha, "alpha")]
    #[case(PreReleaseLabel::Beta, "beta")]
    #[case(PreReleaseLabel::Rc, "rc")]
    fn test_pre_release_label_to_string(#[case] label: PreReleaseLabel, #[case] expected: &str) {
        assert_eq!(label.to_string(), expected);
    }

    #[rstest]
    #[case(PostMode::Tag, "tag")]
    #[case(PostMode::Commit, "commit")]
    fn test_post_mode_to_string(#[case] mode: PostMode, #[case] expected: &str) {
        assert_eq!(mode.to_string(), expected);
    }

    #[test]
    fn test_branch_rules_from_str() {
        // Test both idiomatic parse() and direct from_str() methods
        let ron_str = r#"[
            (pattern: "develop", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
            (pattern: "release/*", pre_release_label: rc, post_mode: tag)
        ]"#;

        // Test idiomatic parse() method
        let rules_parsed: BranchRules = ron_str.parse().unwrap();
        assert_eq!(rules_parsed.rules.len(), 2);

        // Test direct from_str() method
        let rules_from_str = BranchRules::from_str(ron_str).unwrap();
        assert_eq!(rules_from_str.rules.len(), 2);

        // Both methods should produce identical results
        assert_eq!(rules_parsed.rules, rules_from_str.rules);

        // Verify the parsed rules
        let develop_rule = rules_parsed.find_rule("develop").unwrap();
        assert_eq!(develop_rule.pre_release_label, PreReleaseLabel::Beta);
        assert_eq!(develop_rule.pre_release_num, Some(1));

        let release_rule = rules_parsed.find_rule("release/1").unwrap();
        assert_eq!(release_rule.pre_release_label, PreReleaseLabel::Rc);
        assert_eq!(release_rule.pre_release_num, None);
    }

    #[test]
    fn test_branch_rules_from_str_invalid() {
        let invalid_ron = "invalid ron syntax";
        let result: Result<BranchRules, _> = invalid_ron.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_branch_rules_from_str_with_validation_errors() {
        // Test FromStr with invalid branch rules (wildcard pattern with explicit num)
        let invalid_ron = r#"[
            (pattern: "release/*", pre_release_label: rc, pre_release_num: 1, post_mode: tag)
        ]"#;

        let result: Result<BranchRules, _> = invalid_ron.parse();
        assert!(result.is_err());
        match result.unwrap_err() {
            ZervError::ConflictingOptions(msg) => {
                assert!(msg.contains("wildcard pattern"));
                assert!(msg.contains("cannot have explicit pre_release_num"));
            }
            _ => panic!("Expected ConflictingOptions error"),
        }
    }

    #[test]
    fn test_branch_rules_display_round_trip() {
        // Test that Display produces valid RON that can be parsed back
        let original_ron = r#"[
            (pattern: "develop", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
            (pattern: "release/*", pre_release_label: rc, post_mode: tag)
        ]"#;

        let rules: BranchRules = original_ron.parse().unwrap();
        let display_output = rules.to_string();

        // Should be able to parse the display output back
        let reparsed_rules: BranchRules = display_output.parse().unwrap();

        // Both should have the same rules
        assert_eq!(
            rules.find_rule("develop").unwrap().pre_release_label,
            reparsed_rules
                .find_rule("develop")
                .unwrap()
                .pre_release_label
        );
        assert_eq!(
            rules.find_rule("release/1").unwrap().pre_release_label,
            reparsed_rules
                .find_rule("release/1")
                .unwrap()
                .pre_release_label
        );
    }

    #[test]
    fn test_branch_rules_display_format() {
        // Test that Display produces expected RON format
        let rules = BranchRules::default_rules();
        let display_output = rules.to_string();

        // Should exactly match the expected GitFlow rules RON format (compact)
        let develop_rule = r#"(pattern:"develop",pre_release_label:beta,pre_release_num:Some(1),post_mode:commit)"#;
        let release_rule =
            r#"(pattern:"release/*",pre_release_label:rc,pre_release_num:None,post_mode:tag)"#;
        let expected = format!("[{},{}]", develop_rule, release_rule);

        assert_eq!(display_output, expected);
    }
}
