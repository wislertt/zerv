use indexmap::IndexMap;
use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

use crate::utils::sanitize::Sanitizer;
use crate::version::zerv::resolve_timestamp;
use crate::version::zerv::vars::ZervVars;

/// Variable field enum for type-safe field references
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    EnumString,
    Display,
    EnumIter,
    AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum Var {
    // Primary components (schema.core only, correct order when present, used once each)
    Major,
    Minor,
    Patch,

    // Secondary components (schema.extra_core only, used once each, any order)
    Epoch,
    PreRelease,
    Post,
    Dev,

    // Context components (anywhere, multiple uses allowed)
    // VCS state fields
    Distance,
    Dirty,

    // VCS context fields (bumped)
    BumpedBranch,
    BumpedCommitHash,
    BumpedCommitHashShort,
    BumpedTimestamp,

    // VCS context fields (last)
    LastBranch,
    LastCommitHash,
    LastCommitHashShort,
    LastTimestamp,

    // Custom fields
    #[serde(rename = "custom")]
    #[strum(disabled)]
    Custom(String),

    // Timestamp patterns
    #[serde(rename = "ts")]
    #[strum(disabled)]
    Timestamp(String),
}

impl Var {
    /// Primary component ordering for schema validation
    pub fn primary_component_order() -> &'static IndexMap<Var, ()> {
        static ORDER: std::sync::LazyLock<IndexMap<Var, ()>> = std::sync::LazyLock::new(|| {
            [Var::Major, Var::Minor, Var::Patch]
                .into_iter()
                .map(|v| (v, ()))
                .collect()
        });
        &ORDER
    }

    /// Check if this is a primary component (major/minor/patch)
    pub fn is_primary_component(&self) -> bool {
        matches!(self, Var::Major | Var::Minor | Var::Patch)
    }

    /// Check if this is a secondary component (epoch/pre_release/post/dev)
    pub fn is_secondary_component(&self) -> bool {
        matches!(self, Var::Epoch | Var::PreRelease | Var::Post | Var::Dev)
    }

    /// Check if this is a context component (everything else)
    pub fn is_context_component(&self) -> bool {
        !self.is_primary_component() && !self.is_secondary_component()
    }

    /// Get just the primary value (no labels)
    pub fn resolve_value(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Option<String> {
        match self {
            // Core version fields
            Var::Major => vars.major.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::Minor => vars.minor.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::Patch => vars.patch.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::Epoch => vars.epoch.map(|v| sanitizer.sanitize(&v.to_string())),

            // Metadata fields - return just the value
            Var::Post => vars.post.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::Dev => vars.dev.map(|v| sanitizer.sanitize(&v.to_string())),

            // Pre-release - return number if present, otherwise None
            Var::PreRelease => vars
                .pre_release
                .as_ref()
                .and_then(|pr| pr.number)
                .map(|num| sanitizer.sanitize(&num.to_string())),

            // VCS fields
            Var::BumpedBranch => vars.bumped_branch.as_ref().map(|b| sanitizer.sanitize(b)),
            Var::Distance => vars.distance.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::BumpedCommitHashShort => vars
                .get_bumped_commit_hash_short()
                .map(|h| sanitizer.sanitize(&h)),
            Var::BumpedCommitHash => vars
                .bumped_commit_hash
                .as_ref()
                .map(|h| sanitizer.sanitize(h)),
            Var::BumpedTimestamp => vars
                .bumped_timestamp
                .map(|v| sanitizer.sanitize(&v.to_string())),

            // Last version fields
            Var::LastBranch => vars.last_branch.as_ref().map(|b| sanitizer.sanitize(b)),
            Var::LastCommitHash => vars
                .last_commit_hash
                .as_ref()
                .map(|h| sanitizer.sanitize(h)),
            Var::LastCommitHashShort => vars
                .get_last_commit_hash_short()
                .map(|h| sanitizer.sanitize(&h)),
            Var::LastTimestamp => vars
                .last_timestamp
                .map(|v| sanitizer.sanitize(&v.to_string())),

            // VCS state fields
            Var::Dirty => vars.dirty.map(|v| sanitizer.sanitize(&v.to_string())),

            // Custom fields - lookup in JSON with dot notation
            Var::Custom(name) => vars
                .get_custom_value(name)
                .map(|value| sanitizer.sanitize(&value)),

            // Timestamp
            Var::Timestamp(pattern) => {
                let timestamp = vars.bumped_timestamp.or(vars.last_timestamp);
                if let Some(ts) = timestamp {
                    resolve_timestamp(pattern, ts)
                        .ok()
                        .map(|result| sanitizer.sanitize(&result))
                } else {
                    None
                }
            }
        }
    }

    /// Helper function for fields that return parts + value
    fn resolve_parts_with_value(
        &self,
        vars: &ZervVars,
        value_sanitizer: &Sanitizer,
        parts: Vec<String>,
    ) -> Vec<String> {
        if let Some(value) = self.resolve_value(vars, value_sanitizer) {
            let mut result = parts;
            result.push(value);
            result
        } else {
            vec![]
        }
    }

    /// Get expanded values with separate sanitizers for keys and values
    pub fn resolve_expanded_values_with_key_sanitizer(
        &self,
        vars: &ZervVars,
        value_sanitizer: &Sanitizer,
        key_sanitizer: &Sanitizer,
    ) -> Vec<String> {
        match self {
            // Core version fields - return label + value
            Var::Major => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("major")],
            ),
            Var::Minor => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("minor")],
            ),
            Var::Patch => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("patch")],
            ),
            Var::Epoch => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("epoch")],
            ),

            // Metadata fields - return label + value
            Var::Post => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("post")],
            ),
            Var::Dev => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("dev")],
            ),

            // Pre-release - label + optional value
            Var::PreRelease => {
                if let Some(pr) = &vars.pre_release {
                    let mut parts = vec![key_sanitizer.sanitize(pr.label.label_str())];
                    if let Some(value) = self.resolve_value(vars, value_sanitizer) {
                        parts.push(value);
                    }
                    parts
                } else {
                    vec![]
                }
            }

            // VCS fields
            Var::BumpedBranch => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("branch")],
            ),
            Var::Distance => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("distance")],
            ),
            Var::BumpedCommitHashShort => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("commit")],
            ),
            Var::BumpedCommitHash => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("commit_hash")],
            ),
            Var::BumpedTimestamp => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("timestamp")],
            ),

            // Last version fields
            Var::LastBranch => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("last_branch")],
            ),
            Var::LastCommitHash => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("last_commit")],
            ),
            Var::LastCommitHashShort => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("last_commit_short")],
            ),
            Var::LastTimestamp => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("last_timestamp")],
            ),

            // VCS state fields
            Var::Dirty => self.resolve_parts_with_value(
                vars,
                value_sanitizer,
                vec![key_sanitizer.sanitize("dirty")],
            ),

            // Custom fields - split by dots and sanitize each part
            Var::Custom(name) => {
                if vars.get_custom_value(name).is_some() {
                    // If we have custom data, return key parts + value
                    let key_parts: Vec<String> =
                        name.split('.').map(|s| key_sanitizer.sanitize(s)).collect();
                    self.resolve_parts_with_value(vars, value_sanitizer, key_parts)
                } else {
                    // If no custom data, just return the field name as-is
                    vec![name.clone()]
                }
            }

            // Timestamp - no label, just value
            Var::Timestamp(_) => self
                .resolve_value(vars, value_sanitizer)
                .map(|v| vec![v])
                .unwrap_or_default(),
        }
    }

    /// Get expanded values (for formats that need labels + values)
    /// Uses key sanitizer for keys by default
    pub fn resolve_expanded_values(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Vec<String> {
        let key_sanitizer = Sanitizer::key();
        self.resolve_expanded_values_with_key_sanitizer(vars, sanitizer, &key_sanitizer)
    }
}

/// Component enum for internal use with compact serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "str")]
    Str(String),
    #[serde(rename = "int")]
    Int(u64),
    #[serde(rename = "var")]
    Var(Var),
}

impl Component {
    /// Get just the primary value (no labels)
    pub fn resolve_value(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Option<String> {
        match self {
            Component::Str(s) => Some(sanitizer.sanitize(s)),
            Component::Int(n) => Some(sanitizer.sanitize(&n.to_string())),
            Component::Var(var) => var.resolve_value(vars, sanitizer),
        }
    }

    /// Get expanded values (for formats that need labels + values)
    pub fn resolve_expanded_values(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Vec<String> {
        match self {
            Component::Var(var) => var.resolve_expanded_values(vars, sanitizer),
            // For literals, expanded values is just the single value
            Component::Str(_) | Component::Int(_) => self
                .resolve_value(vars, sanitizer)
                .map(|v| vec![v])
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::ZervFixture;
    use crate::version::zerv::core::PreReleaseLabel;

    // Test fixtures
    fn base_fixture() -> ZervFixture {
        ZervFixture::new().with_version(1, 2, 3)
    }

    fn custom_fixture() -> ZervFixture {
        let mut fixture = ZervFixture::new().with_version(1, 0, 0).build();
        fixture.vars.custom = serde_json::json!({
            "build_id": "123",
            "environment": "prod",
            "metadata": {
                "author": "ci",
                "timestamp": 1703123456
            }
        });
        ZervFixture::from(fixture)
    }

    // Core version field tests
    #[rstest]
    #[case(Var::Major, Some("1"))]
    #[case(Var::Minor, Some("2"))]
    #[case(Var::Patch, Some("3"))]
    #[case(Var::Epoch, None)]
    fn test_var_core_fields(#[case] var: Var, #[case] expected: Option<&str>) {
        let zerv = base_fixture().build();
        let sanitizer = Sanitizer::uint();
        assert_eq!(
            var.resolve_value(&zerv.vars, &sanitizer),
            expected.map(String::from)
        );
    }

    // Metadata field tests
    #[rstest]
    #[case(Var::Post, None)]
    #[case(Var::Dev, None)]
    fn test_var_metadata_fields_none(#[case] var: Var, #[case] expected: Option<&str>) {
        let zerv = base_fixture().build();
        let sanitizer = Sanitizer::uint();
        assert_eq!(
            var.resolve_value(&zerv.vars, &sanitizer),
            expected.map(String::from)
        );
    }

    #[rstest]
    #[case(Var::Post, 5)]
    #[case(Var::Dev, 10)]
    #[case(Var::Epoch, 2)]
    fn test_var_metadata_fields_with_values(#[case] var: Var, #[case] value: u64) {
        let zerv = match var {
            Var::Post => base_fixture().with_post(value).build(),
            Var::Dev => base_fixture().with_dev(value).build(),
            Var::Epoch => base_fixture().with_epoch(value).build(),
            _ => panic!("Invalid var"),
        };
        let sanitizer = Sanitizer::uint();
        assert_eq!(
            var.resolve_value(&zerv.vars, &sanitizer),
            Some(value.to_string())
        );
    }

    // Pre-release field tests
    #[rstest]
    #[case(PreReleaseLabel::Alpha, Some(1), Some("1"))]
    #[case(PreReleaseLabel::Beta, Some(2), Some("2"))]
    #[case(PreReleaseLabel::Rc, Some(3), Some("3"))]
    #[case(PreReleaseLabel::Alpha, None, None)]
    fn test_var_pre_release(
        #[case] label: PreReleaseLabel,
        #[case] number: Option<u64>,
        #[case] expected: Option<&str>,
    ) {
        let zerv = base_fixture().with_pre_release(label, number).build();
        let sanitizer = Sanitizer::uint();
        assert_eq!(
            Var::PreRelease.resolve_value(&zerv.vars, &sanitizer),
            expected.map(String::from)
        );
    }

    // VCS field tests
    #[rstest]
    #[case(Var::BumpedBranch, "main")]
    #[case(Var::BumpedBranch, "develop")]
    fn test_var_branch_fields(#[case] var: Var, #[case] branch: &str) {
        let zerv = base_fixture().with_branch(branch.to_string()).build();
        let sanitizer = Sanitizer::semver_str();
        assert_eq!(
            var.resolve_value(&zerv.vars, &sanitizer),
            Some(branch.to_string())
        );
    }

    #[rstest]
    #[case(5)]
    #[case(0)]
    #[case(100)]
    fn test_var_distance(#[case] distance: u64) {
        let zerv = base_fixture().with_distance(distance).build();
        let sanitizer = Sanitizer::uint();
        assert_eq!(
            Var::Distance.resolve_value(&zerv.vars, &sanitizer),
            Some(distance.to_string())
        );
    }

    #[rstest]
    #[case(Var::BumpedCommitHashShort, "abcdef1234567890", "abcdef1")]
    #[case(Var::BumpedCommitHashShort, "123456789", "1234567")]
    fn test_var_commit_hash_short(#[case] var: Var, #[case] hash: &str, #[case] expected: &str) {
        let zerv = base_fixture().with_commit_hash(hash.to_string()).build();
        let sanitizer = Sanitizer::semver_str();
        assert_eq!(
            var.resolve_value(&zerv.vars, &sanitizer),
            Some(expected.to_string())
        );
    }

    #[rstest]
    #[case("abcdef1234567890")]
    #[case("123456789abcdef")]
    fn test_var_bumped_commit_hash(#[case] hash: &str) {
        let zerv = base_fixture().with_commit_hash(hash.to_string()).build();
        let sanitizer = Sanitizer::semver_str();
        assert_eq!(
            Var::BumpedCommitHash.resolve_value(&zerv.vars, &sanitizer),
            Some(hash.to_string())
        );
    }

    #[rstest]
    #[case(true, "true")]
    #[case(false, "false")]
    fn test_var_dirty(#[case] dirty: bool, #[case] expected: &str) {
        let mut zerv = base_fixture().build();
        zerv.vars.dirty = Some(dirty);
        let sanitizer = Sanitizer::semver_str();
        assert_eq!(
            Var::Dirty.resolve_value(&zerv.vars, &sanitizer),
            Some(expected.to_string())
        );
    }

    // Last version field tests
    #[rstest]
    #[case(Var::LastBranch, "last-branch")]
    fn test_var_last_branch(#[case] var: Var, #[case] branch: &str) {
        let mut zerv = base_fixture().build();
        zerv.vars.last_branch = Some(branch.to_string());
        let sanitizer = Sanitizer::semver_str();
        assert_eq!(
            var.resolve_value(&zerv.vars, &sanitizer),
            Some("last.branch".to_string())
        );
    }

    #[rstest]
    #[case("last-commit-hash")]
    fn test_var_last_commit_hash(#[case] hash: &str) {
        let mut zerv = base_fixture().build();
        zerv.vars.last_commit_hash = Some(hash.to_string());
        let sanitizer = Sanitizer::semver_str();
        assert_eq!(
            Var::LastCommitHash.resolve_value(&zerv.vars, &sanitizer),
            Some("last.commit.hash".to_string())
        );
    }

    #[rstest]
    #[case(1703123456)]
    fn test_var_timestamps(#[case] timestamp: u64) {
        let mut zerv = base_fixture().build();
        zerv.vars.bumped_timestamp = Some(timestamp);
        zerv.vars.last_timestamp = Some(timestamp + 1000);
        let sanitizer = Sanitizer::uint();

        assert_eq!(
            Var::BumpedTimestamp.resolve_value(&zerv.vars, &sanitizer),
            Some(timestamp.to_string())
        );
        assert_eq!(
            Var::LastTimestamp.resolve_value(&zerv.vars, &sanitizer),
            Some((timestamp + 1000).to_string())
        );
    }

    // Custom field tests
    #[rstest]
    #[case("build_id", Some("123"))]
    #[case("environment", Some("prod"))]
    #[case("metadata.author", Some("ci"))]
    #[case("metadata.timestamp", Some("1703123456"))]
    #[case("nonexistent", None)]
    #[case("metadata.nonexistent", None)]
    fn test_var_custom_fields(#[case] key: &str, #[case] expected: Option<&str>) {
        let zerv = custom_fixture().build();
        let sanitizer = Sanitizer::semver_str();
        let var = Var::Custom(key.to_string());
        assert_eq!(
            var.resolve_value(&zerv.vars, &sanitizer),
            expected.map(String::from)
        );
    }

    // Timestamp pattern tests
    #[rstest]
    #[case("YYYY", 1703123456, Some("2023"))]
    #[case("MM", 1703123456, Some("12"))]
    #[case("DD", 1703123456, Some("21"))]
    #[case("invalid", 1703123456, None)]
    fn test_var_timestamp_patterns(
        #[case] pattern: &str,
        #[case] timestamp: u64,
        #[case] expected: Option<&str>,
    ) {
        let mut zerv = base_fixture().build();
        zerv.vars.bumped_timestamp = Some(timestamp);
        let sanitizer = Sanitizer::semver_str();
        let var = Var::Timestamp(pattern.to_string());
        assert_eq!(
            var.resolve_value(&zerv.vars, &sanitizer),
            expected.map(String::from)
        );
    }

    // Sanitization tests
    #[rstest]
    #[case(Sanitizer::pep440_local_str(), "Feature/API-v2", "feature.api.v2")]
    #[case(Sanitizer::semver_str(), "Feature/API-v2", "Feature.API.v2")]
    fn test_var_sanitization(
        #[case] sanitizer: Sanitizer,
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let zerv = base_fixture().with_branch(input.to_string()).build();
        assert_eq!(
            Var::BumpedBranch.resolve_value(&zerv.vars, &sanitizer),
            Some(expected.to_string())
        );
    }

    // Component tests
    #[rstest]
    #[case(Component::Str("test".to_string()), Sanitizer::semver_str(), Some("test"))]
    #[case(Component::Int(42), Sanitizer::uint(), Some("42"))]
    #[case(Component::Int(0), Sanitizer::uint(), Some("0"))]
    fn test_component_resolve_value(
        #[case] component: Component,
        #[case] sanitizer: Sanitizer,
        #[case] expected: Option<&str>,
    ) {
        let zerv = base_fixture().build();
        assert_eq!(
            component.resolve_value(&zerv.vars, &sanitizer),
            expected.map(String::from)
        );
    }

    // Expanded values tests - Core fields
    #[rstest]
    #[case(Var::Major, vec!["major", "1"])]
    #[case(Var::Minor, vec!["minor", "2"])]
    #[case(Var::Patch, vec!["patch", "3"])]
    fn test_var_expanded_core_fields(#[case] var: Var, #[case] expected: Vec<&str>) {
        let zerv = base_fixture().build();
        let sanitizer = Sanitizer::uint();
        let result: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(var.resolve_expanded_values(&zerv.vars, &sanitizer), result);
    }

    // Expanded values tests - Metadata fields
    #[rstest]
    #[case(Var::Post, 5, vec!["post", "5"])]
    #[case(Var::Dev, 10, vec!["dev", "10"])]
    #[case(Var::Epoch, 2, vec!["epoch", "2"])]
    fn test_var_expanded_metadata_fields(
        #[case] var: Var,
        #[case] value: u64,
        #[case] expected: Vec<&str>,
    ) {
        let zerv = match var {
            Var::Post => base_fixture().with_post(value).build(),
            Var::Dev => base_fixture().with_dev(value).build(),
            Var::Epoch => base_fixture().with_epoch(value).build(),
            _ => panic!("Invalid var"),
        };
        let sanitizer = Sanitizer::uint();
        let result: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(var.resolve_expanded_values(&zerv.vars, &sanitizer), result);
    }

    // Expanded values tests - Pre-release
    #[rstest]
    #[case(PreReleaseLabel::Alpha, Some(1), vec!["alpha", "1"])]
    #[case(PreReleaseLabel::Beta, Some(2), vec!["beta", "2"])]
    #[case(PreReleaseLabel::Rc, None, vec!["rc"])]
    fn test_var_expanded_pre_release(
        #[case] label: PreReleaseLabel,
        #[case] number: Option<u64>,
        #[case] expected: Vec<&str>,
    ) {
        let zerv = base_fixture().with_pre_release(label, number).build();
        let sanitizer = Sanitizer::uint();
        let result: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            Var::PreRelease.resolve_expanded_values(&zerv.vars, &sanitizer),
            result
        );
    }

    // Expanded values tests - VCS fields
    #[rstest]
    #[case(Var::BumpedBranch, "main", Sanitizer::semver_str(), vec!["branch", "main"])]
    #[case(Var::Distance, "5", Sanitizer::uint(), vec!["distance", "5"])]
    fn test_var_expanded_vcs_fields(
        #[case] var: Var,
        #[case] value: &str,
        #[case] sanitizer: Sanitizer,
        #[case] expected: Vec<&str>,
    ) {
        let zerv = match var {
            Var::BumpedBranch => base_fixture().with_branch(value.to_string()).build(),
            Var::Distance => base_fixture().with_distance(value.parse().unwrap()).build(),
            _ => panic!("Invalid var"),
        };
        let result: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(var.resolve_expanded_values(&zerv.vars, &sanitizer), result);
    }

    // Expanded values tests - Custom fields
    #[rstest]
    #[case("build_id", Sanitizer::semver_str(), vec!["build.id", "123"])]
    #[case("build_id", Sanitizer::uint(), vec!["build.id", "123"])]
    #[case("metadata.author", Sanitizer::semver_str(), vec!["metadata", "author", "ci"])]
    #[case("nonexistent", Sanitizer::semver_str(), vec!["nonexistent"])]
    fn test_var_expanded_custom_fields(
        #[case] key: &str,
        #[case] sanitizer: Sanitizer,
        #[case] expected: Vec<&str>,
    ) {
        let zerv = custom_fixture().build();
        let var = Var::Custom(key.to_string());
        let result: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(var.resolve_expanded_values(&zerv.vars, &sanitizer), result);
    }

    // Component expanded values tests
    #[rstest]
    #[case(Component::Str("test".to_string()), Sanitizer::semver_str(), vec!["test"])]
    #[case(Component::Int(42), Sanitizer::uint(), vec!["42"])]
    fn test_component_expanded_values(
        #[case] component: Component,
        #[case] sanitizer: Sanitizer,
        #[case] expected: Vec<&str>,
    ) {
        let zerv = base_fixture().build();
        let result: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            component.resolve_expanded_values(&zerv.vars, &sanitizer),
            result
        );
    }

    // Key sanitizer separation test
    #[test]
    fn test_key_sanitizer_separation() {
        let zerv = base_fixture()
            .with_branch("feature/test".to_string())
            .build();
        let value_sanitizer = Sanitizer::pep440_local_str();
        let key_sanitizer = Sanitizer::str(Some("_"), false, false, None);

        let result = Var::BumpedBranch.resolve_expanded_values_with_key_sanitizer(
            &zerv.vars,
            &value_sanitizer,
            &key_sanitizer,
        );

        assert_eq!(
            result,
            vec!["branch".to_string(), "feature.test".to_string()]
        );
    }

    // Edge case tests
    #[rstest]
    #[case(Var::Epoch, vec![])]
    #[case(Var::Post, vec![])]
    #[case(Var::Dev, vec![])]
    fn test_var_expanded_empty_fields(#[case] var: Var, #[case] expected: Vec<&str>) {
        let zerv = base_fixture().build();
        let sanitizer = Sanitizer::uint();
        let result: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(var.resolve_expanded_values(&zerv.vars, &sanitizer), result);
    }

    #[test]
    fn test_timestamp_fallback() {
        let mut zerv = base_fixture().build();
        zerv.vars.last_timestamp = Some(1703123456);
        let sanitizer = Sanitizer::semver_str();
        let var = Var::Timestamp("YYYY".to_string());
        assert_eq!(
            var.resolve_value(&zerv.vars, &sanitizer),
            Some("2023".to_string())
        );
    }

    // Component categorization tests
    #[rstest]
    #[case(Var::Major, true)]
    #[case(Var::Minor, true)]
    #[case(Var::Patch, true)]
    #[case(Var::Epoch, false)]
    #[case(Var::PreRelease, false)]
    #[case(Var::Distance, false)]
    #[case(Var::Custom("test".to_string()), false)]
    fn test_is_primary_component(#[case] var: Var, #[case] expected: bool) {
        assert_eq!(var.is_primary_component(), expected);
    }

    #[rstest]
    #[case(Var::Epoch, true)]
    #[case(Var::PreRelease, true)]
    #[case(Var::Post, true)]
    #[case(Var::Dev, true)]
    #[case(Var::Major, false)]
    #[case(Var::Distance, false)]
    #[case(Var::Custom("test".to_string()), false)]
    fn test_is_secondary_component(#[case] var: Var, #[case] expected: bool) {
        assert_eq!(var.is_secondary_component(), expected);
    }

    #[rstest]
    #[case(Var::Distance, true)]
    #[case(Var::Dirty, true)]
    #[case(Var::BumpedBranch, true)]
    #[case(Var::Custom("test".to_string()), true)]
    #[case(Var::Timestamp("YYYY".to_string()), true)]
    #[case(Var::Major, false)]
    #[case(Var::Epoch, false)]
    fn test_is_context_component(#[case] var: Var, #[case] expected: bool) {
        assert_eq!(var.is_context_component(), expected);
    }

    #[test]
    fn test_primary_component_order() {
        let order = Var::primary_component_order();

        // Test correct order
        assert_eq!(order.get_index_of(&Var::Major), Some(0));
        assert_eq!(order.get_index_of(&Var::Minor), Some(1));
        assert_eq!(order.get_index_of(&Var::Patch), Some(2));

        // Test non-primary components not in order
        assert_eq!(order.get_index_of(&Var::Epoch), None);
        assert_eq!(order.get_index_of(&Var::Distance), None);
    }
}
