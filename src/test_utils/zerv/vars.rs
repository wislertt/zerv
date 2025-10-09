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

    /// Add last branch
    pub fn with_last_branch(mut self, branch: String) -> Self {
        self.vars.last_branch = Some(branch);
        self
    }

    /// Create with all VCS-related fields
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
        self.vars.distance = Some(distance);
        self.vars.dirty = Some(dirty);
        self.vars.bumped_branch = Some(bumped_branch);
        self.vars.bumped_commit_hash = Some(bumped_commit_hash);
        self.vars.last_commit_hash = Some(last_commit_hash);
        self.vars.last_timestamp = Some(last_timestamp);
        self.vars.last_branch = Some(last_branch);
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
