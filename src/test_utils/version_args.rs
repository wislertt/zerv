use crate::cli::version::args::VersionArgs;
use crate::version::zerv::bump::types::BumpType;

/// Enum for override types - stores override values for testing
#[derive(Debug, Clone, PartialEq)]
pub enum OverrideType {
    TagVersion(String),
    Distance(u32),
    Dirty(bool),
    CurrentBranch(String),
    CommitHash(String),
    Major(u32),
    Minor(u32),
    Patch(u32),
    Post(u32),
    Dev(u32),
    PreReleaseLabel(String),
    PreReleaseNum(u32),
    Epoch(u32),
}

/// Test fixture for creating VersionArgs with sensible defaults and chainable methods
pub struct VersionArgsFixture {
    args: VersionArgs,
}

impl VersionArgsFixture {
    /// Create a new fixture with default values
    pub fn new() -> Self {
        Self {
            args: VersionArgs::default(),
        }
    }

    /// Build and return the final VersionArgs
    pub fn build(self) -> VersionArgs {
        self.args
    }

    /// Set source
    pub fn with_source(mut self, source: &str) -> Self {
        self.args.source = source.to_string();
        self
    }

    /// Set schema
    pub fn with_schema(mut self, schema: &str) -> Self {
        self.args.schema = Some(schema.to_string());
        self
    }

    /// Set schema RON
    pub fn with_schema_ron(mut self, schema_ron: &str) -> Self {
        self.args.schema_ron = Some(schema_ron.to_string());
        self
    }

    /// Set input format
    pub fn with_input_format(mut self, format: &str) -> Self {
        self.args.input_format = format.to_string();
        self
    }

    /// Set output format
    pub fn with_output_format(mut self, format: &str) -> Self {
        self.args.output_format = format.to_string();
        self
    }

    /// Set directory
    pub fn with_directory(mut self, directory: &str) -> Self {
        self.args.directory = Some(directory.to_string());
        self
    }

    /// Set output template
    pub fn with_output_template(mut self, template: &str) -> Self {
        self.args.output_template = Some(template.to_string());
        self
    }

    /// Set output prefix
    pub fn with_output_prefix(mut self, prefix: &str) -> Self {
        self.args.output_prefix = Some(prefix.to_string());
        self
    }

    // Chainable methods for VCS overrides

    /// Set tag version
    pub fn with_tag_version(mut self, tag_version: &str) -> Self {
        self.args.tag_version = Some(tag_version.to_string());
        self
    }

    /// Set distance
    pub fn with_distance(mut self, distance: u32) -> Self {
        self.args.distance = Some(distance);
        self
    }

    /// Set dirty flag
    pub fn with_dirty(mut self, dirty: bool) -> Self {
        self.args.dirty = dirty;
        self
    }

    /// Set no_dirty flag
    pub fn with_no_dirty(mut self, no_dirty: bool) -> Self {
        self.args.no_dirty = no_dirty;
        self
    }

    /// Set clean flag
    pub fn with_clean_flag(mut self, clean: bool) -> Self {
        self.args.clean = clean;
        self
    }

    /// Set current branch
    pub fn with_current_branch(mut self, branch: &str) -> Self {
        self.args.current_branch = Some(branch.to_string());
        self
    }

    /// Set commit hash
    pub fn with_commit_hash(mut self, hash: &str) -> Self {
        self.args.commit_hash = Some(hash.to_string());
        self
    }

    // Chainable methods for version component overrides

    /// Set post value
    pub fn with_post(mut self, post: u32) -> Self {
        self.args.post = Some(post);
        self
    }

    /// Set dev value
    pub fn with_dev(mut self, dev: u32) -> Self {
        self.args.dev = Some(dev);
        self
    }

    /// Set pre-release label
    pub fn with_pre_release_label(mut self, label: &str) -> Self {
        self.args.pre_release_label = Some(label.to_string());
        self
    }

    /// Set pre-release number
    pub fn with_pre_release_num(mut self, num: u32) -> Self {
        self.args.pre_release_num = Some(num);
        self
    }

    /// Set epoch
    pub fn with_epoch(mut self, epoch: u32) -> Self {
        self.args.epoch = Some(epoch);
        self
    }

    /// Set major version
    pub fn with_major(mut self, major: u32) -> Self {
        self.args.major = Some(major);
        self
    }

    /// Set minor version
    pub fn with_minor(mut self, minor: u32) -> Self {
        self.args.minor = Some(minor);
        self
    }

    /// Set patch version
    pub fn with_patch(mut self, patch: u32) -> Self {
        self.args.patch = Some(patch);
        self
    }

    /// Set custom variables
    pub fn with_custom(mut self, custom: &str) -> Self {
        self.args.custom = Some(custom.to_string());
        self
    }

    // Chainable methods for bump operations

    /// Set bump major
    pub fn with_bump_major(mut self, increment: u32) -> Self {
        self.args.bump_major = Some(Some(increment));
        self
    }

    /// Set bump minor
    pub fn with_bump_minor(mut self, increment: u32) -> Self {
        self.args.bump_minor = Some(Some(increment));
        self
    }

    /// Set bump patch
    pub fn with_bump_patch(mut self, increment: u32) -> Self {
        self.args.bump_patch = Some(Some(increment));
        self
    }

    /// Set bump post
    pub fn with_bump_post(mut self, increment: u32) -> Self {
        self.args.bump_post = Some(Some(increment));
        self
    }

    /// Set bump dev
    pub fn with_bump_dev(mut self, increment: u32) -> Self {
        self.args.bump_dev = Some(Some(increment));
        self
    }

    /// Set bump pre-release number
    pub fn with_bump_pre_release_num(mut self, increment: u32) -> Self {
        self.args.bump_pre_release_num = Some(Some(increment));
        self
    }

    /// Set bump epoch
    pub fn with_bump_epoch(mut self, increment: u32) -> Self {
        self.args.bump_epoch = Some(Some(increment));
        self
    }

    /// Set bump pre-release label
    pub fn with_bump_pre_release_label(mut self, label: &str) -> Self {
        self.args.bump_pre_release_label = Some(label.to_string());
        self
    }

    /// Set bump context flag
    pub fn with_bump_context(mut self, bump_context: bool) -> Self {
        self.args.bump_context = bump_context;
        self
    }

    /// Set no bump context flag
    pub fn with_no_bump_context(mut self, no_bump_context: bool) -> Self {
        self.args.no_bump_context = no_bump_context;
        self
    }

    // Chainable methods for complex operations

    /// Apply bump specifications from BumpType vector
    pub fn with_bump_specs(mut self, bumps: Vec<BumpType>) -> Self {
        for bump_type in bumps {
            match bump_type {
                BumpType::Major(increment) => self.args.bump_major = Some(Some(increment as u32)),
                BumpType::Minor(increment) => self.args.bump_minor = Some(Some(increment as u32)),
                BumpType::Patch(increment) => self.args.bump_patch = Some(Some(increment as u32)),
                BumpType::Post(increment) => self.args.bump_post = Some(Some(increment as u32)),
                BumpType::Dev(increment) => self.args.bump_dev = Some(Some(increment as u32)),
                BumpType::Epoch(increment) => self.args.bump_epoch = Some(Some(increment as u32)),
                BumpType::PreReleaseNum(increment) => {
                    self.args.bump_pre_release_num = Some(Some(increment as u32))
                }
                BumpType::PreReleaseLabel(_) => {
                    // For now, we don't handle pre-release label bumps in test fixtures
                    // This can be extended later when needed
                }
            }
        }
        self
    }

    /// Apply override specifications from OverrideType vector
    pub fn with_override_specs(mut self, overrides: Vec<OverrideType>) -> Self {
        for override_type in overrides {
            match override_type {
                OverrideType::TagVersion(version) => self.args.tag_version = Some(version),
                OverrideType::Distance(distance) => self.args.distance = Some(distance),
                OverrideType::Dirty(dirty) => self.args.dirty = dirty,
                OverrideType::CurrentBranch(branch) => self.args.current_branch = Some(branch),
                OverrideType::CommitHash(hash) => self.args.commit_hash = Some(hash),
                OverrideType::Major(major) => self.args.major = Some(major),
                OverrideType::Minor(minor) => self.args.minor = Some(minor),
                OverrideType::Patch(patch) => self.args.patch = Some(patch),
                OverrideType::Post(post) => self.args.post = Some(post),
                OverrideType::Dev(dev) => self.args.dev = Some(dev),
                OverrideType::PreReleaseLabel(label) => self.args.pre_release_label = Some(label),
                OverrideType::PreReleaseNum(num) => self.args.pre_release_num = Some(num),
                OverrideType::Epoch(epoch) => self.args.epoch = Some(epoch),
            }
        }
        self
    }
}

impl Default for VersionArgsFixture {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{formats, sources};
    use crate::version::zerv::bump::types::BumpType;

    #[test]
    fn test_new_creates_default_fixture() {
        let fixture = VersionArgsFixture::new();
        let args = fixture.build();

        assert_eq!(args.source, sources::GIT);
        assert_eq!(args.input_format, formats::AUTO);
        assert_eq!(args.output_format, formats::SEMVER);
        assert_eq!(args.tag_version, None);
        assert_eq!(args.schema, None);
        assert!(!args.dirty);
        assert!(!args.clean);
    }

    #[test]
    fn test_chainable_basic_configuration() {
        let args = VersionArgsFixture::new()
            .with_tag_version("2.0.0")
            .with_source("custom")
            .with_schema("test-schema")
            .with_output_format(formats::PEP440)
            .with_directory("/test/dir")
            .build();

        assert_eq!(args.tag_version, Some("2.0.0".to_string()));
        assert_eq!(args.source, "custom");
        assert_eq!(args.schema, Some("test-schema".to_string()));
        assert_eq!(args.output_format, formats::PEP440);
        assert_eq!(args.directory, Some("/test/dir".to_string()));
    }

    #[test]
    fn test_chainable_vcs_overrides() {
        let args = VersionArgsFixture::new()
            .with_tag_version("v3.0.0")
            .with_distance(10)
            .with_dirty(true)
            .with_current_branch("feature/test")
            .with_commit_hash("deadbeef")
            .build();

        assert_eq!(args.tag_version, Some("v3.0.0".to_string()));
        assert_eq!(args.distance, Some(10));
        assert!(args.dirty);
        assert_eq!(args.current_branch, Some("feature/test".to_string()));
        assert_eq!(args.commit_hash, Some("deadbeef".to_string()));
    }

    #[test]
    fn test_chainable_bump_operations() {
        let args = VersionArgsFixture::new()
            .with_bump_major(2)
            .with_bump_minor(3)
            .with_bump_patch(4)
            .with_bump_post(5)
            .with_bump_dev(6)
            .with_bump_epoch(7)
            .with_bump_pre_release_num(8)
            .build();

        assert_eq!(args.bump_major, Some(Some(2)));
        assert_eq!(args.bump_minor, Some(Some(3)));
        assert_eq!(args.bump_patch, Some(Some(4)));
        assert_eq!(args.bump_post, Some(Some(5)));
        assert_eq!(args.bump_dev, Some(Some(6)));
        assert_eq!(args.bump_epoch, Some(Some(7)));
        assert_eq!(args.bump_pre_release_num, Some(Some(8)));
    }

    #[test]
    fn test_with_bump_specs_chainable() {
        let bumps = vec![BumpType::Major(2), BumpType::Minor(3), BumpType::Patch(1)];

        let args = VersionArgsFixture::new()
            .with_bump_specs(bumps)
            .with_tag_version("v1.0.0")
            .build();

        assert_eq!(args.bump_major, Some(Some(2)));
        assert_eq!(args.bump_minor, Some(Some(3)));
        assert_eq!(args.bump_patch, Some(Some(1)));
        assert_eq!(args.tag_version, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_with_override_specs_chainable() {
        let overrides = vec![
            OverrideType::TagVersion("v2.0.0".to_string()),
            OverrideType::Distance(15),
            OverrideType::Dirty(true),
            OverrideType::CurrentBranch("main".to_string()),
        ];

        let args = VersionArgsFixture::new()
            .with_override_specs(overrides)
            .with_output_format(formats::PEP440)
            .build();

        assert_eq!(args.tag_version, Some("v2.0.0".to_string()));
        assert_eq!(args.distance, Some(15));
        assert!(args.dirty);
        assert_eq!(args.current_branch, Some("main".to_string()));
        assert_eq!(args.output_format, formats::PEP440);
    }

    #[test]
    fn test_default_trait() {
        let fixture1 = VersionArgsFixture::default();
        let fixture2 = VersionArgsFixture::new();

        let args1 = fixture1.build();
        let args2 = fixture2.build();

        // Both should create identical default configurations
        assert_eq!(args1.source, args2.source);
        assert_eq!(args1.input_format, args2.input_format);
        assert_eq!(args1.output_format, args2.output_format);
        assert_eq!(args1.dirty, args2.dirty);
    }
}
