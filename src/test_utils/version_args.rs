use crate::cli::utils::template::Template;
use crate::cli::version::args::VersionArgs;
use crate::test_utils::types::{
    BumpType,
    OverrideType,
};

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
        self.args.main.source = source.to_string();
        self
    }

    /// Set schema
    pub fn with_schema(mut self, schema: &str) -> Self {
        self.args.main.schema = Some(schema.to_string());
        self
    }

    /// Set schema RON
    pub fn with_schema_ron(mut self, schema_ron: &str) -> Self {
        self.args.main.schema_ron = Some(schema_ron.to_string());
        self
    }

    /// Set input format
    pub fn with_input_format(mut self, format: &str) -> Self {
        self.args.main.input_format = format.to_string();
        self
    }

    /// Set output format
    pub fn with_output_format(mut self, format: &str) -> Self {
        self.args.main.output_format = format.to_string();
        self
    }

    /// Set directory
    pub fn with_directory(mut self, directory: &str) -> Self {
        self.args.main.directory = Some(directory.to_string());
        self
    }

    /// Set output template
    pub fn with_output_template(mut self, template: &str) -> Self {
        self.args.main.output_template = Some(Template::Value(template.to_string()));
        self
    }

    /// Set output prefix
    pub fn with_output_prefix(mut self, prefix: &str) -> Self {
        self.args.main.output_prefix = Some(prefix.to_string());
        self
    }

    // Chainable methods for VCS overrides

    /// Set tag version
    pub fn with_tag_version(mut self, tag_version: &str) -> Self {
        self.args.overrides.tag_version = Some(tag_version.to_string());
        self
    }

    /// Set distance
    pub fn with_distance(mut self, distance: u32) -> Self {
        self.args.overrides.distance = Some(distance);
        self
    }

    /// Set dirty flag
    pub fn with_dirty(mut self, dirty: bool) -> Self {
        self.args.overrides.dirty = dirty;
        self
    }

    /// Set no_dirty flag
    pub fn with_no_dirty(mut self, no_dirty: bool) -> Self {
        self.args.overrides.no_dirty = no_dirty;
        self
    }

    /// Set clean flag
    pub fn with_clean_flag(mut self, clean: bool) -> Self {
        self.args.overrides.clean = clean;
        self
    }

    /// Set current branch
    pub fn with_current_branch(mut self, branch: &str) -> Self {
        self.args.overrides.bumped_branch = Some(branch.to_string());
        self
    }

    /// Set commit hash
    pub fn with_commit_hash(mut self, hash: &str) -> Self {
        self.args.overrides.bumped_commit_hash = Some(hash.to_string());
        self
    }

    // Chainable methods for version component overrides

    /// Set post value
    pub fn with_post(mut self, post: u32) -> Self {
        self.args.overrides.post = Some(Template::Value(post));
        self
    }

    /// Set dev value
    pub fn with_dev(mut self, dev: u32) -> Self {
        self.args.overrides.dev = Some(Template::Value(dev));
        self
    }

    /// Set pre-release label
    pub fn with_pre_release_label(mut self, label: &str) -> Self {
        self.args.overrides.pre_release_label = Some(label.to_string());
        self
    }

    /// Set pre-release number
    pub fn with_pre_release_num(mut self, num: u32) -> Self {
        self.args.overrides.pre_release_num = Some(num.into());
        self
    }

    /// Set epoch
    pub fn with_epoch(mut self, epoch: u32) -> Self {
        self.args.overrides.epoch = Some(epoch.into());
        self
    }

    /// Set major version
    pub fn with_major(mut self, major: u32) -> Self {
        self.args.overrides.major = Some(major.into());
        self
    }

    /// Set minor version
    pub fn with_minor(mut self, minor: u32) -> Self {
        self.args.overrides.minor = Some(minor.into());
        self
    }

    /// Set patch version
    pub fn with_patch(mut self, patch: u32) -> Self {
        self.args.overrides.patch = Some(patch.into());
        self
    }

    /// Set custom variables
    pub fn with_custom(mut self, custom: &str) -> Self {
        self.args.overrides.custom = Some(custom.to_string());
        self
    }

    // Chainable methods for bump operations

    /// Set bump major
    pub fn with_bump_major(mut self, increment: u32) -> Self {
        self.args.bumps.bump_major = Some(Some(increment.into()));
        self
    }

    /// Set bump minor
    pub fn with_bump_minor(mut self, increment: u32) -> Self {
        self.args.bumps.bump_minor = Some(Some(increment.into()));
        self
    }

    /// Set bump patch
    pub fn with_bump_patch(mut self, increment: u32) -> Self {
        self.args.bumps.bump_patch = Some(Some(increment.into()));
        self
    }

    /// Set bump post
    pub fn with_bump_post(mut self, increment: u32) -> Self {
        self.args.bumps.bump_post = Some(Some(increment.into()));
        self
    }

    /// Set bump dev
    pub fn with_bump_dev(mut self, increment: u32) -> Self {
        self.args.bumps.bump_dev = Some(Some(increment.into()));
        self
    }

    /// Set bump pre-release number
    pub fn with_bump_pre_release_num(mut self, increment: u32) -> Self {
        self.args.bumps.bump_pre_release_num = Some(Some(increment.into()));
        self
    }

    /// Set bump epoch
    pub fn with_bump_epoch(mut self, increment: u32) -> Self {
        self.args.bumps.bump_epoch = Some(Some(increment.into()));
        self
    }

    /// Set bump pre-release label
    pub fn with_bump_pre_release_label(mut self, label: &str) -> Self {
        self.args.bumps.bump_pre_release_label = Some(label.to_string());
        self
    }

    /// Set bump context flag
    pub fn with_bump_context(mut self, bump_context: bool) -> Self {
        self.args.bumps.bump_context = bump_context;
        self
    }

    /// Set no bump context flag
    pub fn with_no_bump_context(mut self, no_bump_context: bool) -> Self {
        self.args.bumps.no_bump_context = no_bump_context;
        self
    }

    // Chainable methods for complex operations

    /// Apply bump specifications from BumpType vector
    pub fn with_bump_specs(mut self, bumps: Vec<BumpType>) -> Self {
        for bump_type in bumps {
            match bump_type {
                BumpType::Major(increment) => {
                    self.args.bumps.bump_major = Some(Some((increment as u32).into()))
                }
                BumpType::Minor(increment) => {
                    self.args.bumps.bump_minor = Some(Some((increment as u32).into()))
                }
                BumpType::Patch(increment) => {
                    self.args.bumps.bump_patch = Some(Some((increment as u32).into()))
                }
                BumpType::Post(increment) => {
                    self.args.bumps.bump_post = Some(Some((increment as u32).into()))
                }
                BumpType::Dev(increment) => {
                    self.args.bumps.bump_dev = Some(Some((increment as u32).into()))
                }
                BumpType::Epoch(increment) => {
                    self.args.bumps.bump_epoch = Some(Some((increment as u32).into()))
                }
                BumpType::PreReleaseNum(increment) => {
                    self.args.bumps.bump_pre_release_num = Some(Some((increment as u32).into()))
                }
                BumpType::PreReleaseLabel(_) => {
                    // For now, we don't handle pre-release label bumps in test fixtures
                    // This can be extended later when needed
                }
                BumpType::SchemaBump {
                    section,
                    index,
                    value,
                } => {
                    // Convert to spec format: "0" or "0=5"
                    let spec = match value {
                        Some(v) => format!("{index}={v}"), // "0=5"
                        None => index.to_string(),         // "0"
                    };
                    match section.as_str() {
                        "core" => {
                            self.args.bumps.bump_core.push(spec.into());
                        }
                        "extra_core" => {
                            self.args.bumps.bump_extra_core.push(spec.into());
                        }
                        "build" => {
                            self.args.bumps.bump_build.push(spec.into());
                        }
                        _ => {
                            // Unknown section - ignore for now
                        }
                    }
                }
            }
        }
        self
    }

    /// Apply override specifications from OverrideType vector
    pub fn with_override_specs(mut self, overrides: Vec<OverrideType>) -> Self {
        for override_type in overrides {
            match override_type {
                OverrideType::TagVersion(version) => {
                    self.args.overrides.tag_version = Some(version)
                }
                OverrideType::Distance(distance) => self.args.overrides.distance = Some(distance),
                OverrideType::Dirty(dirty) => self.args.overrides.dirty = dirty,
                OverrideType::BumpedBranch(branch) => {
                    self.args.overrides.bumped_branch = Some(branch)
                }
                OverrideType::BumpedCommitHash(hash) => {
                    self.args.overrides.bumped_commit_hash = Some(hash)
                }
                OverrideType::BumpedTimestamp(timestamp) => {
                    self.args.overrides.bumped_timestamp = Some(timestamp)
                }
                OverrideType::Major(major) => self.args.overrides.major = Some(major.into()),
                OverrideType::Minor(minor) => self.args.overrides.minor = Some(minor.into()),
                OverrideType::Patch(patch) => self.args.overrides.patch = Some(patch.into()),
                OverrideType::Post(post) => self.args.overrides.post = Some(post.into()),
                OverrideType::Dev(dev) => self.args.overrides.dev = Some(dev.into()),
                OverrideType::PreReleaseLabel(label) => {
                    self.args.overrides.pre_release_label = Some(label)
                }
                OverrideType::PreReleaseNum(num) => {
                    self.args.overrides.pre_release_num = Some(num.into())
                }
                OverrideType::Epoch(epoch) => self.args.overrides.epoch = Some(epoch.into()),
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
    use crate::test_utils::types::BumpType;
    use crate::utils::constants::{
        formats,
        sources,
    };

    #[test]
    fn test_new_creates_default_fixture() {
        let fixture = VersionArgsFixture::new();
        let args = fixture.build();

        assert_eq!(args.main.source, sources::GIT);
        assert_eq!(args.main.input_format, formats::AUTO);
        assert_eq!(args.main.output_format, formats::SEMVER);
        assert_eq!(args.overrides.tag_version, None);
        assert_eq!(args.main.schema, None);
        assert!(!args.overrides.dirty);
        assert!(!args.overrides.clean);
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

        assert_eq!(args.overrides.tag_version, Some("2.0.0".to_string()));
        assert_eq!(args.main.source, "custom");
        assert_eq!(args.main.schema, Some("test-schema".to_string()));
        assert_eq!(args.main.output_format, formats::PEP440);
        assert_eq!(args.main.directory, Some("/test/dir".to_string()));
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

        assert_eq!(args.overrides.tag_version, Some("v3.0.0".to_string()));
        assert_eq!(args.overrides.distance, Some(10));
        assert!(args.overrides.dirty);
        assert_eq!(
            args.overrides.bumped_branch,
            Some("feature/test".to_string())
        );
        assert_eq!(
            args.overrides.bumped_commit_hash,
            Some("deadbeef".to_string())
        );
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

        assert_eq!(args.bumps.bump_major, Some(Some(2.into())));
        assert_eq!(args.bumps.bump_minor, Some(Some(3.into())));
        assert_eq!(args.bumps.bump_patch, Some(Some(4.into())));
        assert_eq!(args.bumps.bump_post, Some(Some(5.into())));
        assert_eq!(args.bumps.bump_dev, Some(Some(6.into())));
        assert_eq!(args.bumps.bump_epoch, Some(Some(7.into())));
        assert_eq!(args.bumps.bump_pre_release_num, Some(Some(8.into())));
    }

    #[test]
    fn test_with_bump_specs_chainable() {
        let bumps = vec![BumpType::Major(2), BumpType::Minor(3), BumpType::Patch(1)];

        let args = VersionArgsFixture::new()
            .with_bump_specs(bumps)
            .with_tag_version("v1.0.0")
            .build();

        assert_eq!(args.bumps.bump_major, Some(Some(2.into())));
        assert_eq!(args.bumps.bump_minor, Some(Some(3.into())));
        assert_eq!(args.bumps.bump_patch, Some(Some(1.into())));
        assert_eq!(args.overrides.tag_version, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_with_override_specs_chainable() {
        let overrides = vec![
            OverrideType::TagVersion("v2.0.0".to_string()),
            OverrideType::Distance(15),
            OverrideType::Dirty(true),
            OverrideType::BumpedBranch("main".to_string()),
        ];

        let args = VersionArgsFixture::new()
            .with_override_specs(overrides)
            .with_output_format(formats::PEP440)
            .build();

        assert_eq!(args.overrides.tag_version, Some("v2.0.0".to_string()));
        assert_eq!(args.overrides.distance, Some(15));
        assert!(args.overrides.dirty);
        assert_eq!(args.overrides.bumped_branch, Some("main".to_string()));
        assert_eq!(args.main.output_format, formats::PEP440);
    }

    #[test]
    fn test_default_trait() {
        let fixture1 = VersionArgsFixture::default();
        let fixture2 = VersionArgsFixture::new();

        let args1 = fixture1.build();
        let args2 = fixture2.build();

        // Both should create identical default configurations
        assert_eq!(args1.main.source, args2.main.source);
        assert_eq!(args1.main.input_format, args2.main.input_format);
        assert_eq!(args1.main.output_format, args2.main.output_format);
        assert_eq!(args1.overrides.dirty, args2.overrides.dirty);
    }

    // Tests for uncovered methods

    #[test]
    fn test_with_schema_ron() {
        let args = VersionArgsFixture::new()
            .with_schema_ron("core: [{var: \"major\"}]")
            .build();
        assert_eq!(
            args.main.schema_ron,
            Some("core: [{var: \"major\"}]".to_string())
        );
    }

    #[test]
    fn test_with_output_template() {
        let args = VersionArgsFixture::new()
            .with_output_template("v{{major}}.{{minor}}.{{patch}}")
            .build();
        assert_eq!(
            args.main.output_template,
            Some(crate::cli::utils::template::Template::Value(
                "v{{major}}.{{minor}}.{{patch}}".to_string()
            ))
        );
    }

    #[test]
    fn test_with_output_prefix() {
        let args = VersionArgsFixture::new()
            .with_output_prefix("release-")
            .build();
        assert_eq!(args.main.output_prefix, Some("release-".to_string()));
    }

    #[test]
    fn test_with_no_dirty() {
        let args = VersionArgsFixture::new().with_no_dirty(true).build();
        assert!(args.overrides.no_dirty);

        let args2 = VersionArgsFixture::new().with_no_dirty(false).build();
        assert!(!args2.overrides.no_dirty);
    }

    #[test]
    fn test_with_clean_flag() {
        let args = VersionArgsFixture::new().with_clean_flag(true).build();
        assert!(args.overrides.clean);

        let args2 = VersionArgsFixture::new().with_clean_flag(false).build();
        assert!(!args2.overrides.clean);
    }

    #[test]
    fn test_with_commit_hash() {
        let args = VersionArgsFixture::new()
            .with_commit_hash("deadbeef1234567890")
            .build();
        assert_eq!(
            args.overrides.bumped_commit_hash,
            Some("deadbeef1234567890".to_string())
        );
    }

    #[test]
    fn test_with_custom() {
        let args = VersionArgsFixture::new()
            .with_custom("build_number")
            .build();
        assert_eq!(args.overrides.custom, Some("build_number".to_string()));
    }

    #[test]
    fn test_all_uncovered_methods_chainable() {
        let args = VersionArgsFixture::new()
            .with_schema_ron("test-schema")
            .with_output_template("{{version}}")
            .with_output_prefix("v")
            .with_no_dirty(true)
            .with_clean_flag(true)
            .with_commit_hash("hash123")
            .with_custom("custom_value")
            .build();

        // Verify all settings were applied
        assert_eq!(args.main.schema_ron, Some("test-schema".to_string()));
        assert_eq!(
            args.main.output_template,
            Some(crate::cli::utils::template::Template::Value(
                "{{version}}".to_string()
            ))
        );
        assert_eq!(args.main.output_prefix, Some("v".to_string()));
        assert!(args.overrides.no_dirty);
        assert!(args.overrides.clean);
        assert_eq!(
            args.overrides.bumped_commit_hash,
            Some("hash123".to_string())
        );
        assert_eq!(args.overrides.custom, Some("custom_value".to_string()));
    }
}
