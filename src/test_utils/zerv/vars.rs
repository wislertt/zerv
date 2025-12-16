use crate::version::zerv::{
    PreReleaseLabel,
    PreReleaseVar,
    ZervVars,
};

/// Fixture for creating ZervVars test data
pub struct ZervVarsFixture {
    vars: ZervVars,
}

impl ZervVarsFixture {
    /// Create a new fixture with default 1.0.0 version
    pub fn new() -> Self {
        Self {
            vars: ZervVars {
                major: Some(1),
                minor: Some(0),
                patch: Some(0),
                ..Default::default()
            },
        }
    }

    /// Build and return the final ZervVars
    pub fn build(self) -> ZervVars {
        self.vars
    }

    /// Set version components (chainable)
    pub fn with_version(mut self, major: u64, minor: u64, patch: u64) -> Self {
        self.vars.major = Some(major);
        self.vars.minor = Some(minor);
        self.vars.patch = Some(patch);
        self
    }

    /// Add pre-release information
    pub fn with_pre_release(mut self, label: PreReleaseLabel, number: Option<u64>) -> Self {
        self.vars.pre_release = Some(PreReleaseVar { label, number });
        self
    }

    /// Add epoch
    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.vars.epoch = Some(epoch);
        self
    }

    /// Add post version
    pub fn with_post(mut self, post: u64) -> Self {
        self.vars.post = Some(post);
        self
    }

    /// Add dev version
    pub fn with_dev(mut self, dev: u64) -> Self {
        self.vars.dev = Some(dev);
        self
    }

    /// Add distance
    pub fn with_distance(mut self, distance: u64) -> Self {
        self.vars.distance = Some(distance);
        self
    }

    /// Add dirty flag
    pub fn with_dirty(mut self, dirty: bool) -> Self {
        self.vars.dirty = Some(dirty);
        self
    }

    /// Add bumped branch
    pub fn with_bumped_branch(mut self, branch: String) -> Self {
        self.vars.bumped_branch = Some(branch);
        self
    }

    /// Add bumped commit hash
    pub fn with_bumped_commit_hash(mut self, hash: String) -> Self {
        self.vars.bumped_commit_hash = Some(hash);
        self
    }

    /// Add last commit hash
    pub fn with_last_commit_hash(mut self, hash: String) -> Self {
        self.vars.last_commit_hash = Some(hash);
        self
    }

    /// Add last timestamp
    pub fn with_last_timestamp(mut self, timestamp: u64) -> Self {
        self.vars.last_timestamp = Some(timestamp);
        self
    }

    /// Add bumped timestamp
    pub fn with_bumped_timestamp(mut self, timestamp: u64) -> Self {
        self.vars.bumped_timestamp = Some(timestamp);
        self
    }

    /// Clear pre-release (set to None)
    pub fn without_pre_release(mut self) -> Self {
        self.vars.pre_release = None;
        self
    }

    /// Clear post-release (set to None)
    pub fn without_post(mut self) -> Self {
        self.vars.post = None;
        self
    }

    /// Add last branch
    pub fn with_last_branch(mut self, branch: String) -> Self {
        self.vars.last_branch = Some(branch);
        self
    }

    /// Create with all VCS-related fields
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
        self.vars.distance = distance;
        self.vars.dirty = dirty;
        self.vars.bumped_branch = bumped_branch;
        self.vars.bumped_commit_hash = bumped_commit_hash;
        self.vars.last_commit_hash = last_commit_hash;
        self.vars.last_timestamp = last_timestamp;
        self.vars.last_branch = last_branch;
        self
    }

    // Legacy compatibility - keep basic() as alias for new()
    pub fn basic() -> Self {
        Self::new()
    }
}

impl Default for ZervVarsFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ZervVarsFixture> for ZervVars {
    fn from(fixture: ZervVarsFixture) -> Self {
        fixture.vars
    }
}

impl From<ZervVars> for ZervVarsFixture {
    fn from(vars: ZervVars) -> Self {
        Self { vars }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_default_fixture() {
        let fixture = ZervVarsFixture::new();
        let vars = fixture.build();

        assert_eq!(vars.major, Some(1));
        assert_eq!(vars.minor, Some(0));
        assert_eq!(vars.patch, Some(0));
        assert_eq!(vars.pre_release, None);
        assert_eq!(vars.epoch, None);
        assert_eq!(vars.post, None);
        assert_eq!(vars.dev, None);
    }

    #[test]
    fn test_with_version() {
        let vars = ZervVarsFixture::new().with_version(2, 5, 3).build();

        assert_eq!(vars.major, Some(2));
        assert_eq!(vars.minor, Some(5));
        assert_eq!(vars.patch, Some(3));
    }

    #[test]
    fn test_with_pre_release_with_number() {
        let vars = ZervVarsFixture::new()
            .with_pre_release(PreReleaseLabel::Alpha, Some(5))
            .build();

        assert!(vars.pre_release.is_some());
        let pre = vars.pre_release.unwrap();
        assert_eq!(pre.label, PreReleaseLabel::Alpha);
        assert_eq!(pre.number, Some(5));
    }

    #[test]
    fn test_with_pre_release_without_number() {
        let vars = ZervVarsFixture::new()
            .with_pre_release(PreReleaseLabel::Beta, None)
            .build();

        assert!(vars.pre_release.is_some());
        let pre = vars.pre_release.unwrap();
        assert_eq!(pre.label, PreReleaseLabel::Beta);
        assert_eq!(pre.number, None);
    }

    #[test]
    fn test_with_epoch() {
        let vars = ZervVarsFixture::new().with_epoch(42).build();

        assert_eq!(vars.epoch, Some(42));
    }

    #[test]
    fn test_with_post() {
        let vars = ZervVarsFixture::new().with_post(10).build();

        assert_eq!(vars.post, Some(10));
    }

    #[test]
    fn test_with_dev() {
        let vars = ZervVarsFixture::new().with_dev(7).build();

        assert_eq!(vars.dev, Some(7));
    }

    #[test]
    fn test_with_distance() {
        let vars = ZervVarsFixture::new().with_distance(5).build();

        assert_eq!(vars.distance, Some(5));
    }

    #[test]
    fn test_with_dirty_true() {
        let vars = ZervVarsFixture::new().with_dirty(true).build();

        assert_eq!(vars.dirty, Some(true));
    }

    #[test]
    fn test_with_dirty_false() {
        let vars = ZervVarsFixture::new().with_dirty(false).build();

        assert_eq!(vars.dirty, Some(false));
    }

    #[test]
    fn test_with_bumped_branch() {
        let vars = ZervVarsFixture::new()
            .with_bumped_branch("feature/test".to_string())
            .build();

        assert_eq!(vars.bumped_branch, Some("feature/test".to_string()));
    }

    #[test]
    fn test_with_bumped_commit_hash() {
        let vars = ZervVarsFixture::new()
            .with_bumped_commit_hash("deadbeef1234567890".to_string())
            .build();

        assert_eq!(
            vars.bumped_commit_hash,
            Some("deadbeef1234567890".to_string())
        );
    }

    #[test]
    fn test_with_last_branch() {
        let vars = ZervVarsFixture::new()
            .with_last_branch("main".to_string())
            .build();

        assert_eq!(vars.last_branch, Some("main".to_string()));
    }

    #[test]
    fn test_with_last_commit_hash() {
        let vars = ZervVarsFixture::new()
            .with_last_commit_hash("abcdef1234567890".to_string())
            .build();

        assert_eq!(vars.last_commit_hash, Some("abcdef1234567890".to_string()));
    }

    #[test]
    fn test_with_last_timestamp() {
        let vars = ZervVarsFixture::new()
            .with_last_timestamp(1703000000)
            .build();

        assert_eq!(vars.last_timestamp, Some(1703000000));
    }

    #[test]
    fn test_from_zerv_vars() {
        let original_vars = ZervVars {
            major: Some(2),
            minor: Some(1),
            patch: Some(0),
            pre_release: Some(PreReleaseVar {
                label: PreReleaseLabel::Rc,
                number: Some(3),
            }),
            epoch: Some(1),
            post: Some(5),
            dev: Some(2),
            distance: Some(10),
            dirty: Some(true),
            bumped_branch: Some("release".to_string()),
            bumped_commit_hash: Some("hash123".to_string()),
            bumped_timestamp: Some(1703123456),
            last_branch: Some("main".to_string()),
            last_commit_hash: Some("hash456".to_string()),
            last_timestamp: Some(1703000000),
            last_tag_version: Some("v2.1.0-rc.3".to_string()),
            custom: serde_json::json!({}),
        };

        let fixture = ZervVarsFixture::from(original_vars.clone());
        let result_vars = fixture.build();

        assert_eq!(result_vars, original_vars);
    }

    #[test]
    fn test_chainable_methods_complex() {
        let vars = ZervVarsFixture::new()
            .with_version(3, 2, 1)
            .with_pre_release(PreReleaseLabel::Beta, Some(4))
            .with_epoch(2)
            .with_post(1)
            .with_dev(3)
            .with_distance(7)
            .with_dirty(true)
            .with_bumped_branch("develop".to_string())
            .with_bumped_commit_hash("commit123".to_string())
            .with_last_branch("main".to_string())
            .with_last_commit_hash("commit456".to_string())
            .with_last_timestamp(1703000000)
            .build();

        // Verify all values are set correctly
        assert_eq!(vars.major, Some(3));
        assert_eq!(vars.minor, Some(2));
        assert_eq!(vars.patch, Some(1));
        assert!(vars.pre_release.is_some());
        let pre = vars.pre_release.as_ref().unwrap();
        assert_eq!(pre.label, PreReleaseLabel::Beta);
        assert_eq!(pre.number, Some(4));
        assert_eq!(vars.epoch, Some(2));
        assert_eq!(vars.post, Some(1));
        assert_eq!(vars.dev, Some(3));
        assert_eq!(vars.distance, Some(7));
        assert_eq!(vars.dirty, Some(true));
        assert_eq!(vars.bumped_branch, Some("develop".to_string()));
        assert_eq!(vars.bumped_commit_hash, Some("commit123".to_string()));
        assert_eq!(vars.last_branch, Some("main".to_string()));
        assert_eq!(vars.last_commit_hash, Some("commit456".to_string()));
        assert_eq!(vars.last_timestamp, Some(1703000000));
    }
}
