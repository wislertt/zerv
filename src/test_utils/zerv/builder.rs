//! ZervRonBuilder for programmatic RON fixture generation
//!
//! This module provides the `ZervRonBuilder` struct with a fluent API for
//! creating Zerv RON fixtures programmatically.

use crate::version::zerv::{Component, PreReleaseLabel, PreReleaseVar, Zerv, ZervSchema, ZervVars};
use ron::to_string;
use serde_json;

/// Builder for creating Zerv RON fixtures programmatically
#[derive(Debug, Clone)]
pub struct ZervRonBuilder {
    schema: ZervSchema,
    vars: ZervVars,
}

impl ZervRonBuilder {
    /// Create a new ZervRonBuilder with default values
    pub fn new() -> Self {
        Self {
            schema: ZervSchema {
                core: vec![],
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars {
                ..Default::default()
            },
        }
    }

    /// Set the core version (major, minor, patch)
    pub fn core_version(mut self, major: u64, minor: u64, patch: u64) -> Self {
        self.schema.core = vec![
            Component::VarField("major".to_string()),
            Component::VarField("minor".to_string()),
            Component::VarField("patch".to_string()),
        ];
        self.vars.major = Some(major);
        self.vars.minor = Some(minor);
        self.vars.patch = Some(patch);
        self
    }

    /// Add epoch to the schema and variables
    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.schema
            .core
            .insert(0, Component::VarField("epoch".to_string()));
        self.vars.epoch = Some(epoch);
        self
    }

    /// Add pre-release information
    pub fn with_pre_release(mut self, label: PreReleaseLabel, number: Option<u64>) -> Self {
        self.schema
            .extra_core
            .push(Component::VarField("pre_release".to_string()));
        self.vars.pre_release = Some(PreReleaseVar { label, number });
        self
    }

    /// Add post-release information
    pub fn with_post(mut self, post: u64) -> Self {
        self.schema
            .extra_core
            .push(Component::VarField("post".to_string()));
        self.vars.post = Some(post);
        self
    }

    /// Add development version information
    pub fn with_dev(mut self, dev: u64) -> Self {
        self.schema
            .extra_core
            .push(Component::VarField("dev".to_string()));
        self.vars.dev = Some(dev);
        self
    }

    /// Add custom extra core components
    pub fn with_extra_core_components(mut self, components: Vec<Component>) -> Self {
        self.schema.extra_core.extend(components);
        self
    }

    /// Add build components
    pub fn with_build_components(mut self, components: Vec<Component>) -> Self {
        self.schema.build = components;
        self
    }

    /// Add VCS data (branch, commit hash, distance, dirty)
    pub fn with_vcs_data(
        mut self,
        branch: &str,
        commit_hash: &str,
        distance: u64,
        dirty: bool,
    ) -> Self {
        self.vars.bumped_branch = Some(branch.to_string());
        self.vars.bumped_commit_hash = Some(commit_hash.to_string());
        // Note: bumped_commit_hash_short is now derived from bumped_commit_hash
        self.vars.distance = Some(distance);
        self.vars.dirty = Some(dirty);
        self
    }

    /// Add last version information
    pub fn with_last_version(mut self, branch: &str, commit_hash: &str, timestamp: u64) -> Self {
        self.vars.last_branch = Some(branch.to_string());
        self.vars.last_commit_hash = Some(commit_hash.to_string());
        self.vars.last_timestamp = Some(timestamp);
        self
    }

    /// Add custom variables
    pub fn with_custom_vars(mut self, custom: serde_json::Value) -> Self {
        self.vars.custom = custom;
        self
    }

    /// Add a custom field to the schema (for testing invalid fields)
    pub fn with_custom_field(mut self, field_name: &str) -> Self {
        self.schema
            .extra_core
            .push(Component::VarField(field_name.to_string()));
        self
    }

    /// Add a timestamp pattern to the schema
    pub fn with_timestamp_pattern(mut self, pattern: &str) -> Self {
        self.schema
            .build
            .push(Component::VarTimestamp(pattern.to_string()));
        self
    }

    /// Create the Zerv object
    pub fn build(self) -> Zerv {
        Zerv::new(self.schema, self.vars)
            .expect("Failed to create Zerv object - check schema and vars compatibility")
    }

    /// Generate RON string from the built Zerv object
    pub fn to_ron(self) -> String {
        let zerv = self.build();
        to_string(&zerv).expect("Failed to serialize Zerv to RON")
    }

    /// Get mutable reference to vars for direct manipulation
    pub fn vars_mut(&mut self) -> &mut ZervVars {
        &mut self.vars
    }

    /// Get mutable reference to schema for direct manipulation
    pub fn schema_mut(&mut self) -> &mut ZervSchema {
        &mut self.schema
    }
}

impl Default for ZervRonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::Zerv;
    use ron::from_str;
    use rstest::*;

    #[rstest]
    #[case(1, 2, 3)]
    #[case(0, 0, 1)]
    #[case(2, 10, 5)]
    #[case(10, 0, 0)]
    fn test_zerv_ron_builder_core_version(
        #[case] major: u64,
        #[case] minor: u64,
        #[case] patch: u64,
    ) {
        let ron = ZervRonBuilder::new()
            .core_version(major, minor, patch)
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");
        assert_eq!(zerv.vars.major, Some(major));
        assert_eq!(zerv.vars.minor, Some(minor));
        assert_eq!(zerv.vars.patch, Some(patch));
    }

    #[rstest]
    #[case(PreReleaseLabel::Alpha, Some(1))]
    #[case(PreReleaseLabel::Beta, Some(2))]
    #[case(PreReleaseLabel::Rc, Some(3))]
    #[case(PreReleaseLabel::Alpha, None)]
    #[case(PreReleaseLabel::Beta, None)]
    fn test_zerv_ron_builder_with_pre_release(
        #[case] label: PreReleaseLabel,
        #[case] number: Option<u64>,
    ) {
        let ron = ZervRonBuilder::new()
            .core_version(1, 0, 0)
            .with_pre_release(label.clone(), number)
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");
        assert_eq!(zerv.vars.major, Some(1));
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().label, label);
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().number, number);
    }

    #[rstest]
    #[case("main", "abc1234", 5, false)]
    #[case("develop", "def5678", 0, true)]
    #[case("feature/branch", "ghi9012", 10, false)]
    #[case("release/v1.0", "jkl3456", 1, true)]
    fn test_zerv_ron_builder_with_vcs_data(
        #[case] branch: &str,
        #[case] commit_hash: &str,
        #[case] distance: u64,
        #[case] dirty: bool,
    ) {
        let ron = ZervRonBuilder::new()
            .core_version(1, 2, 3)
            .with_vcs_data(branch, commit_hash, distance, dirty)
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");
        assert_eq!(zerv.vars.bumped_branch, Some(branch.to_string()));
        assert_eq!(zerv.vars.bumped_commit_hash, Some(commit_hash.to_string()));
        assert_eq!(
            zerv.vars.get_bumped_commit_hash_short(),
            Some(commit_hash[..7].to_string())
        );
        assert_eq!(zerv.vars.distance, Some(distance));
        assert_eq!(zerv.vars.dirty, Some(dirty));
    }

    #[rstest]
    #[case(serde_json::json!({"environment": "prod", "build_number": 123}))]
    #[case(serde_json::json!({"env": "dev", "version": "1.0.0", "debug": true}))]
    #[case(serde_json::json!({"custom_field": "value", "number": 42}))]
    #[case(serde_json::json!({}))]
    fn test_zerv_ron_builder_with_custom_vars(#[case] custom_vars: serde_json::Value) {
        let ron = ZervRonBuilder::new()
            .core_version(1, 2, 3)
            .with_custom_vars(custom_vars.clone())
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");
        assert_eq!(zerv.vars.custom, custom_vars);
    }

    #[rstest]
    #[case(0, 1, 2, 3)]
    #[case(1, 2, 3, 4)]
    #[case(5, 10, 15, 20)]
    fn test_zerv_ron_builder_with_epoch(
        #[case] epoch: u64,
        #[case] major: u64,
        #[case] minor: u64,
        #[case] patch: u64,
    ) {
        let ron = ZervRonBuilder::new()
            .core_version(major, minor, patch)
            .with_epoch(epoch)
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");
        assert_eq!(zerv.vars.epoch, Some(epoch));
        assert_eq!(zerv.vars.major, Some(major));
        assert_eq!(zerv.vars.minor, Some(minor));
        assert_eq!(zerv.vars.patch, Some(patch));
    }

    #[rstest]
    #[case(1, 2, 3, 5)]
    #[case(0, 0, 1, 10)]
    #[case(2, 5, 0, 1)]
    fn test_zerv_ron_builder_with_post(
        #[case] major: u64,
        #[case] minor: u64,
        #[case] patch: u64,
        #[case] post: u64,
    ) {
        let ron = ZervRonBuilder::new()
            .core_version(major, minor, patch)
            .with_post(post)
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");
        assert_eq!(zerv.vars.major, Some(major));
        assert_eq!(zerv.vars.minor, Some(minor));
        assert_eq!(zerv.vars.patch, Some(patch));
        assert_eq!(zerv.vars.post, Some(post));
    }

    #[rstest]
    #[case(1, 2, 3, 7)]
    #[case(0, 0, 1, 15)]
    #[case(2, 5, 0, 3)]
    fn test_zerv_ron_builder_with_dev(
        #[case] major: u64,
        #[case] minor: u64,
        #[case] patch: u64,
        #[case] dev: u64,
    ) {
        let ron = ZervRonBuilder::new()
            .core_version(major, minor, patch)
            .with_dev(dev)
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");
        assert_eq!(zerv.vars.major, Some(major));
        assert_eq!(zerv.vars.minor, Some(minor));
        assert_eq!(zerv.vars.patch, Some(patch));
        assert_eq!(zerv.vars.dev, Some(dev));
    }

    #[test]
    fn test_zerv_ron_builder_fluent_chaining() {
        let ron = ZervRonBuilder::new()
            .core_version(2, 1, 0)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_post(5)
            .with_dev(10)
            .with_vcs_data("main", "abc1234", 3, false)
            .with_last_version("main", "def5678", 1234567890)
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");

        // Verify all components are set
        assert_eq!(zerv.vars.epoch, Some(1));
        assert_eq!(zerv.vars.major, Some(2));
        assert_eq!(zerv.vars.minor, Some(1));
        assert_eq!(zerv.vars.patch, Some(0));
        assert_eq!(
            zerv.vars.pre_release.as_ref().unwrap().label,
            PreReleaseLabel::Beta
        );
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().number, Some(2));
        assert_eq!(zerv.vars.post, Some(5));
        assert_eq!(zerv.vars.dev, Some(10));
        assert_eq!(zerv.vars.bumped_branch, Some("main".to_string()));
        assert_eq!(zerv.vars.bumped_commit_hash, Some("abc1234".to_string()));
        assert_eq!(zerv.vars.distance, Some(3));
        assert_eq!(zerv.vars.dirty, Some(false));
        assert_eq!(zerv.vars.last_branch, Some("main".to_string()));
        assert_eq!(zerv.vars.last_commit_hash, Some("def5678".to_string()));
        assert_eq!(zerv.vars.last_timestamp, Some(1234567890));
    }

    #[test]
    fn test_zerv_ron_builder_custom_field() {
        let ron = ZervRonBuilder::new()
            .core_version(1, 0, 0)
            .with_custom_field("custom_field")
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");
        assert!(zerv.schema.extra_core.iter().any(|c| {
            if let Component::VarField(name) = c {
                name == "custom_field"
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_zerv_ron_builder_timestamp_pattern() {
        let ron = ZervRonBuilder::new()
            .core_version(1, 0, 0)
            .with_timestamp_pattern("%Y%m%d")
            .to_ron();

        let zerv: Zerv = from_str(&ron).expect("Should parse");
        assert!(zerv.schema.build.iter().any(|c| {
            if let Component::VarTimestamp(pattern) = c {
                pattern == "%Y%m%d"
            } else {
                false
            }
        }));
    }
}
