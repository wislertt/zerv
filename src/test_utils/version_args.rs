use crate::cli::version::args::VersionArgs;
use crate::constants::{formats, sources};

/// Test fixture for creating VersionArgs with sensible defaults
pub struct VersionArgsFixture;

impl VersionArgsFixture {
    /// Create a basic VersionArgs with default values
    pub fn create() -> VersionArgs {
        VersionArgs {
            version: None,
            source: sources::GIT.to_string(),
            schema: None,
            schema_ron: None,
            input_format: formats::AUTO.to_string(),
            output_format: formats::SEMVER.to_string(),
            tag_version: None,
            distance: None,
            dirty: false,
            no_dirty: false,
            clean: false,
            current_branch: None,
            commit_hash: None,
            post: None,
            dev: None,
            pre_release_label: None,
            pre_release_num: None,
            epoch: None,
            custom: None,
            bump_major: None,
            bump_minor: None,
            bump_patch: None,
            bump_distance: None,
            bump_post: None,
            bump_dev: None,
            bump_pre_release_num: None,
            bump_epoch: None,
            bump_context: false,
            no_bump_context: false,
            output_template: None,
            output_prefix: None,
            directory: None,
        }
    }

    /// Create a VersionArgs for testing with overrides
    pub fn with_overrides() -> VersionArgs {
        let mut args = Self::create();
        args.tag_version = Some("v2.0.0".to_string());
        args.distance = Some(5);
        args.dirty = true;
        args.current_branch = Some("feature/test".to_string());
        args.commit_hash = Some("abc123def456".to_string());
        args
    }

    /// Create a VersionArgs for testing with clean state
    pub fn with_clean() -> VersionArgs {
        let mut args = Self::create();
        args.clean = true;
        args
    }

    /// Create a VersionArgs for testing with specific directory
    pub fn with_directory(directory: &str) -> VersionArgs {
        let mut args = Self::create();
        args.directory = Some(directory.to_string());
        args
    }

    /// Create a VersionArgs for testing with specific output format
    pub fn with_output_format(format: &str) -> VersionArgs {
        let mut args = Self::create();
        args.output_format = format.to_string();
        args
    }

    /// Create a VersionArgs for testing with specific schema
    pub fn with_schema(schema: &str) -> VersionArgs {
        let mut args = Self::create();
        args.schema = Some(schema.to_string());
        args
    }

    /// Create a VersionArgs for testing with bump operations
    pub fn with_bumps() -> VersionArgs {
        let mut args = Self::create();
        args.bump_major = Some(Some(1));
        args.bump_minor = Some(Some(2));
        args.bump_patch = Some(Some(3));
        args
    }

    /// Create a VersionArgs for testing with context control
    pub fn with_no_bump_context() -> VersionArgs {
        let mut args = Self::create();
        args.no_bump_context = true;
        args
    }

    /// Create a VersionArgs for testing with conflicting options
    pub fn with_conflicts() -> VersionArgs {
        let mut args = Self::create();
        args.clean = true;
        args.distance = Some(5); // This conflicts with clean
        args.dirty = true; // This also conflicts with clean
        args
    }

    /// Create a VersionArgs for testing with custom variables
    pub fn with_custom_vars() -> VersionArgs {
        let mut args = Self::create();
        args.custom = Some(r#"{"build_id": "123", "environment": "test"}"#.to_string());
        args
    }

    /// Create a VersionArgs for testing with pre-release overrides
    pub fn with_pre_release() -> VersionArgs {
        let mut args = Self::create();
        args.pre_release_label = Some("alpha".to_string());
        args.pre_release_num = Some(1);
        args
    }

    /// Create a VersionArgs for testing with post/dev overrides
    pub fn with_post_dev() -> VersionArgs {
        let mut args = Self::create();
        args.post = Some(5);
        args.dev = Some(10);
        args
    }
}

impl Default for VersionArgsFixture {
    fn default() -> Self {
        Self
    }
}
