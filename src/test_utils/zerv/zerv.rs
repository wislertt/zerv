use crate::version::zerv::{PreReleaseLabel, Zerv};
use crate::version::{pep440::PEP440, semver::SemVer};
use std::str::FromStr;

use super::{ZervSchemaFixture, ZervVarsFixture};

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
        self.zerv.vars.pre_release = Some(crate::version::zerv::PreReleaseVar { label, number });
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

    /// Use standard tier 1 schema (major.minor.patch) - chainable
    pub fn with_standard_tier_1(mut self) -> Self {
        self.zerv.schema = ZervSchemaFixture::standard_tier_1().build();
        self
    }

    /// Use standard tier 2 schema (with build metadata) - chainable
    pub fn with_standard_tier_2(mut self) -> Self {
        self.zerv.schema = ZervSchemaFixture::standard_tier_2().build();
        self
    }

    /// Use standard tier 3 schema (with dev components) - chainable
    pub fn with_standard_tier_3(mut self) -> Self {
        self.zerv.schema = ZervSchemaFixture::standard_tier_3().build();
        self
    }

    /// Use calver tier 1 schema - chainable
    pub fn with_calver_tier_1(mut self) -> Self {
        self.zerv.schema = ZervSchemaFixture::calver_tier_1().build();
        self
    }

    /// Use calver tier 2 schema - chainable
    pub fn with_calver_tier_2(mut self) -> Self {
        self.zerv.schema = ZervSchemaFixture::calver_tier_2().build();
        self
    }

    /// Use calver tier 3 schema - chainable
    pub fn with_calver_tier_3(mut self) -> Self {
        self.zerv.schema = ZervSchemaFixture::calver_tier_3().build();
        self
    }

    /// Set VCS data (chainable)
    #[allow(clippy::too_many_arguments)]
    pub fn with_vcs_data(
        mut self,
        distance: u64,
        dirty: bool,
        bumped_branch: String,
        bumped_commit_hash: String,
        last_commit_hash: String,
        last_timestamp: u64,
        last_branch: String,
    ) -> Self {
        self.zerv.vars.distance = Some(distance);
        self.zerv.vars.dirty = Some(dirty);
        self.zerv.vars.bumped_branch = Some(bumped_branch);
        self.zerv.vars.bumped_commit_hash = Some(bumped_commit_hash);
        self.zerv.vars.last_commit_hash = Some(last_commit_hash);
        self.zerv.vars.last_timestamp = Some(last_timestamp);
        self.zerv.vars.last_branch = Some(last_branch);
        self
    }

    /// Create from SemVer string (chainable)
    pub fn from_semver_str(semver_str: &str) -> Result<Self, crate::error::ZervError> {
        let semver = SemVer::from_str(semver_str)?;
        let zerv: Zerv = semver.into();
        Ok(Self { zerv })
    }

    /// Create from PEP440 string (chainable)
    pub fn from_pep440_str(pep440_str: &str) -> Result<Self, crate::error::ZervError> {
        let pep440 = PEP440::from_str(pep440_str)?;
        let zerv: Zerv = pep440.into();
        Ok(Self { zerv })
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
        assert_eq!(zerv.schema.core.len(), 3);
        assert!(!zerv.schema.extra_core.is_empty()); // standard_tier_1 has extra_core
        assert!(zerv.schema.build.is_empty());

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
        let tier1 = ZervFixture::new().with_standard_tier_1().build();
        let tier2 = ZervFixture::new().with_standard_tier_2().build();
        let tier3 = ZervFixture::new().with_standard_tier_3().build();

        // All should have core components
        assert_eq!(tier1.schema.core.len(), 3);
        assert_eq!(tier2.schema.core.len(), 3);
        assert_eq!(tier3.schema.core.len(), 3);
    }

    #[test]
    fn test_from_semver_str() {
        let zerv = ZervFixture::from_semver_str("1.2.3-alpha.1+build.123")
            .unwrap()
            .build();

        assert_eq!(zerv.vars.major, Some(1));
        assert_eq!(zerv.vars.minor, Some(2));
        assert_eq!(zerv.vars.patch, Some(3));
        assert!(zerv.vars.pre_release.is_some());
        assert!(!zerv.schema.build.is_empty());
    }

    #[test]
    fn test_from_pep440_str() {
        let zerv = ZervFixture::from_pep440_str("2!1.2.3a1.post1.dev1+local.123")
            .unwrap()
            .build();

        assert_eq!(zerv.vars.major, Some(1));
        assert_eq!(zerv.vars.minor, Some(2));
        assert_eq!(zerv.vars.patch, Some(3));
        assert_eq!(zerv.vars.epoch, Some(2));
        assert!(zerv.vars.pre_release.is_some());
        assert!(zerv.vars.post.is_some());
        assert!(zerv.vars.dev.is_some());
        assert!(!zerv.schema.build.is_empty());
    }

    #[test]
    fn test_from_invalid_strings() {
        assert!(ZervFixture::from_semver_str("invalid").is_err());
        assert!(ZervFixture::from_pep440_str("invalid").is_err());
    }

    #[test]
    fn test_chainable_with_string_parsing() {
        let zerv = ZervFixture::from_semver_str("1.0.0")
            .unwrap()
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
