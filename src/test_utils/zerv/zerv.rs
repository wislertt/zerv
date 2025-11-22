use std::str::FromStr;

use super::{
    ZervSchemaFixture,
    ZervVarsFixture,
};
use crate::schema::ZervSchemaPreset;
use crate::version::pep440::PEP440;
use crate::version::semver::SemVer;
use crate::version::zerv::{
    Component,
    PreReleaseLabel,
    PreReleaseVar,
    Var,
    Zerv,
};

/// Fixture for creating complete Zerv test data
pub struct ZervFixture {
    zerv: Zerv,
}

impl ZervFixture {
    /// Create a new fixture with default 1.0.0 version using standard tier 1 schema
    pub fn new() -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::new().build(),
                ZervVarsFixture::new().build(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv: {e}")),
        }
    }

    /// Build and return the final Zerv
    pub fn build(self) -> Zerv {
        self.zerv
    }

    /// Alias for new() for backward compatibility
    pub fn basic() -> Self {
        Self::new()
    }

    /// Set version components (chainable)
    pub fn with_version(mut self, major: u64, minor: u64, patch: u64) -> Self {
        self.zerv.vars.major = Some(major);
        self.zerv.vars.minor = Some(minor);
        self.zerv.vars.patch = Some(patch);
        self
    }

    /// Set pre-release (chainable)
    pub fn with_pre_release(mut self, label: PreReleaseLabel, number: Option<u64>) -> Self {
        self.zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        self
    }

    /// Set epoch (chainable)
    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.zerv.vars.epoch = Some(epoch);
        self
    }

    /// Set post version (chainable)
    pub fn with_post(mut self, post: u64) -> Self {
        self.zerv.vars.post = Some(post);
        self
    }

    /// Set dev version (chainable)
    pub fn with_dev(mut self, dev: u64) -> Self {
        self.zerv.vars.dev = Some(dev);
        self
    }

    /// Set dirty flag (chainable)
    pub fn with_dirty(mut self, dirty: bool) -> Self {
        self.zerv.vars.dirty = Some(dirty);
        self
    }

    /// Add build component (chainable)
    pub fn with_build(mut self, component: Component) -> Self {
        let mut build = self.zerv.schema.build().clone();
        build.push(component);
        self.zerv.schema.set_build(build).unwrap();
        self
    }

    /// Add extra core component (chainable)
    pub fn with_extra_core(mut self, component: Component) -> Self {
        let mut extra_core = self.zerv.schema.extra_core().clone();
        extra_core.push(component);
        self.zerv.schema.set_extra_core(extra_core).unwrap();
        self
    }

    /// Set branch (chainable)
    pub fn with_branch(mut self, branch: String) -> Self {
        self.zerv.vars.bumped_branch = Some(branch);
        // Add Var to build schema if not already present
        let branch_field = Component::Var(Var::BumpedBranch);
        if !self.zerv.schema.build().contains(&branch_field) {
            let mut build = self.zerv.schema.build().clone();
            build.push(branch_field);
            self.zerv.schema.set_build(build).unwrap();
        }
        self
    }

    /// Set distance (chainable)
    pub fn with_distance(mut self, distance: u64) -> Self {
        self.zerv.vars.distance = Some(distance);
        // Add Var to build schema if not already present
        let distance_field = Component::Var(Var::Distance);
        if !self.zerv.schema.build().contains(&distance_field) {
            let mut build = self.zerv.schema.build().clone();
            build.push(distance_field);
            self.zerv.schema.set_build(build).unwrap();
        }
        self
    }

    /// Set commit hash (chainable)
    pub fn with_commit_hash(mut self, hash: String) -> Self {
        self.zerv.vars.bumped_commit_hash = Some(hash);
        // Add Var to build schema if not already present
        let hash_field = Component::Var(Var::BumpedCommitHashShort);
        if !self.zerv.schema.build().contains(&hash_field) {
            let mut build = self.zerv.schema.build().clone();
            build.push(hash_field);
            self.zerv.schema.set_build(build).unwrap();
        }
        self
    }

    /// Set core values directly (chainable)
    pub fn with_core_values(mut self, values: Vec<u64>) -> Self {
        // Clear existing core and rebuild with integers
        let mut core = Vec::new();
        for value in values {
            core.push(Component::UInt(value));
        }
        self.zerv.schema.set_core(core).unwrap();
        self
    }

    /// Use schema preset - chainable
    pub fn with_schema_preset(mut self, schema: ZervSchemaPreset) -> Self {
        self.zerv.schema = ZervSchemaFixture::from_preset(schema).build();
        self
    }

    /// Create with empty schema - chainable
    pub fn with_empty_schema(mut self) -> Self {
        self.zerv.schema.set_core(vec![Component::UInt(1)]).unwrap(); // Need at least one component
        self.zerv.schema.set_extra_core(vec![]).unwrap();
        self.zerv.schema.set_build(vec![]).unwrap();
        self
    }

    /// Add core component - chainable
    pub fn with_core(mut self, component: Component) -> Self {
        let mut core = self.zerv.schema.core().clone();
        core.push(component);
        self.zerv.schema.set_core(core).unwrap();
        self
    }

    /// Set core components directly - chainable
    pub fn with_core_components(mut self, components: Vec<Component>) -> Self {
        self.zerv.schema.set_core(components).unwrap();
        self
    }

    /// Set extra_core components directly - chainable
    pub fn with_extra_core_components(mut self, components: Vec<Component>) -> Self {
        self.zerv.schema.set_extra_core(components).unwrap();
        self
    }

    /// Set build components directly - chainable
    pub fn with_build_components(mut self, components: Vec<Component>) -> Self {
        self.zerv.schema.set_build(components).unwrap();
        self
    }

    /// Set VCS data (chainable)
    #[allow(clippy::too_many_arguments)]
    pub fn with_vcs_data(
        mut self,
        distance: Option<u64>,
        dirty: Option<bool>,
        bumped_branch: Option<String>,
        bumped_commit_hash: Option<String>,
        last_commit_hash: Option<String>,
        last_timestamp: Option<u64>,
        last_branch: Option<String>,
    ) -> Self {
        self.zerv.vars.distance = distance;
        self.zerv.vars.dirty = dirty;
        self.zerv.vars.bumped_branch = bumped_branch;
        self.zerv.vars.bumped_commit_hash = bumped_commit_hash;
        self.zerv.vars.last_commit_hash = last_commit_hash;
        self.zerv.vars.last_timestamp = last_timestamp;
        self.zerv.vars.last_branch = last_branch;
        self
    }

    /// Set bumped timestamp (chainable)
    pub fn with_bumped_timestamp(mut self, timestamp: u64) -> Self {
        self.zerv.vars.bumped_timestamp = Some(timestamp);
        self
    }

    /// Clear pre-release (chainable)
    pub fn without_pre_release(mut self) -> Self {
        self.zerv.vars.pre_release = None;
        self
    }

    /// Clear post-release (chainable)
    pub fn without_post(mut self) -> Self {
        self.zerv.vars.post = None;
        self
    }

    /// Create from SemVer string (chainable)
    pub fn from_semver_str(semver_str: &str) -> Self {
        let semver = SemVer::from_str(semver_str)
            .unwrap_or_else(|e| panic!("Failed to parse SemVer '{semver_str}': {e}"));
        let zerv: Zerv = semver.into();
        Self { zerv }
    }

    /// Create from PEP440 string (chainable)
    pub fn from_pep440_str(pep440_str: &str) -> Self {
        let pep440 = PEP440::from_str(pep440_str)
            .unwrap_or_else(|e| panic!("Failed to parse PEP440 '{pep440_str}': {e}"));
        let zerv: Zerv = pep440.into();
        Self { zerv }
    }

    /// Get the Zerv reference
    pub fn zerv(&self) -> &Zerv {
        &self.zerv
    }
}

impl Default for ZervFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ZervFixture> for Zerv {
    fn from(fixture: ZervFixture) -> Self {
        fixture.zerv
    }
}

impl From<Zerv> for ZervFixture {
    fn from(zerv: Zerv) -> Self {
        Self { zerv }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_zerv_fixture() {
        let fixture = ZervFixture::new();
        let zerv = fixture.zerv();

        // Verify the structure
        assert_eq!(zerv.schema.core().len(), 3);
        assert!(!zerv.schema.extra_core().is_empty()); // standard_tier_1 has extra_core
        assert!(zerv.schema.build().is_empty());

        // Verify vars
        assert_eq!(zerv.vars.major, Some(1));
        assert_eq!(zerv.vars.minor, Some(0));
        assert_eq!(zerv.vars.patch, Some(0));
    }

    #[test]
    fn test_chainable_methods() {
        let zerv = ZervFixture::new()
            .with_version(2, 1, 0)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .build();

        assert_eq!(zerv.vars.major, Some(2));
        assert_eq!(zerv.vars.minor, Some(1));
        assert_eq!(zerv.vars.patch, Some(0));
        assert_eq!(zerv.vars.epoch, Some(1));
        assert!(zerv.vars.pre_release.is_some());
    }

    #[test]
    fn test_schema_presets() {
        let tier1 = ZervFixture::new()
            .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePost)
            .build();
        let tier2 = ZervFixture::new()
            .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostContext)
            .build();
        let tier3 = ZervFixture::new()
            .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostDevContext)
            .build();

        // All should have core components
        assert_eq!(tier1.schema.core().len(), 3);
        assert_eq!(tier2.schema.core().len(), 3);
        assert_eq!(tier3.schema.core().len(), 3);
    }

    #[test]
    fn test_from_semver_str() {
        let zerv = ZervFixture::from_semver_str("1.2.3-alpha.1+build.123").build();

        assert_eq!(zerv.vars.major, Some(1));
        assert_eq!(zerv.vars.minor, Some(2));
        assert_eq!(zerv.vars.patch, Some(3));
        assert!(zerv.vars.pre_release.is_some());
        assert!(!zerv.schema.build().is_empty());
    }

    #[test]
    fn test_from_pep440_str() {
        let zerv = ZervFixture::from_pep440_str("2!1.2.3a1.post1.dev1+local.123").build();

        assert_eq!(zerv.vars.major, Some(1));
        assert_eq!(zerv.vars.minor, Some(2));
        assert_eq!(zerv.vars.patch, Some(3));
        assert_eq!(zerv.vars.epoch, Some(2));
        assert!(zerv.vars.pre_release.is_some());
        assert!(zerv.vars.post.is_some());
        assert!(zerv.vars.dev.is_some());
        assert!(!zerv.schema.build().is_empty());
    }

    #[test]
    #[should_panic(expected = "Failed to parse SemVer")]
    fn test_from_invalid_semver_string() {
        ZervFixture::from_semver_str("invalid");
    }

    #[test]
    #[should_panic(expected = "Failed to parse PEP440")]
    fn test_from_invalid_pep440_string() {
        ZervFixture::from_pep440_str("invalid");
    }

    #[test]
    fn test_chainable_with_string_parsing() {
        let zerv = ZervFixture::from_semver_str("1.0.0")
            .with_version(2, 1, 0)
            .with_epoch(1)
            .build();

        // Should override the parsed version with chained methods
        assert_eq!(zerv.vars.major, Some(2));
        assert_eq!(zerv.vars.minor, Some(1));
        assert_eq!(zerv.vars.patch, Some(0));
        assert_eq!(zerv.vars.epoch, Some(1));
    }
}
