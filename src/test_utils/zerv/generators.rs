//! Generator functions for common RON fixture patterns
//!
//! This module provides high-level generator functions that use the ZervRonBuilder
//! to create common RON fixture patterns.

use super::builder::ZervRonBuilder;
use crate::constants::{ron_fields, shared_fields, timestamp_patterns};
use crate::version::zerv::{Component, PreReleaseLabel};
use serde_json;

/// Common parameters for Zerv RON generation
#[derive(Debug, Clone)]
pub struct ZervParams {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub branch: String,
    pub commit_hash: String,
    pub distance: u64,
    pub dirty: bool,
}

impl ZervParams {
    pub fn new(
        major: u64,
        minor: u64,
        patch: u64,
        branch: &str,
        commit_hash: &str,
        distance: u64,
        dirty: bool,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            branch: branch.to_string(),
            commit_hash: commit_hash.to_string(),
            distance,
            dirty,
        }
    }
}

/// Generate basic Zerv RON with core version fields only
pub fn basic_zerv_ron(major: u64, minor: u64, patch: u64) -> String {
    ZervRonBuilder::new()
        .core_version(major, minor, patch)
        .to_ron()
}

/// Generate Zerv RON with VCS context
pub fn vcs_zerv_ron(params: &ZervParams, post: u64) -> String {
    let mut builder = ZervRonBuilder::new()
        .core_version(params.major, params.minor, params.patch)
        .with_post(post)
        .with_build_components(vec![
            Component::VarField(ron_fields::BRANCH.to_string()),
            Component::VarField(ron_fields::COMMIT_HASH_SHORT.to_string()),
        ]);

    // Set VCS fields using the correct field names
    builder.vars_mut().bumped_branch = Some(params.branch.clone());
    builder.vars_mut().bumped_commit_hash_short = Some(params.commit_hash[..7].to_string());
    builder.vars_mut().distance = Some(params.distance);
    builder.vars_mut().dirty = Some(params.dirty);

    builder.to_ron()
}

/// Generate Zerv RON with pre-release information
pub fn prerelease_zerv_ron(
    params: &ZervParams,
    label: PreReleaseLabel,
    number: Option<u64>,
) -> String {
    ZervRonBuilder::new()
        .core_version(params.major, params.minor, params.patch)
        .with_pre_release(label, number)
        .with_build_components(vec![
            Component::VarField(ron_fields::BRANCH.to_string()),
            Component::VarTimestamp(timestamp_patterns::COMPACT_DATE.to_string()),
        ])
        .with_vcs_data(
            &params.branch,
            &params.commit_hash,
            params.distance,
            params.dirty,
        )
        .to_ron()
}

/// Generate Zerv RON with custom fields
pub fn custom_fields_zerv_ron(params: &ZervParams, custom_vars: serde_json::Value) -> String {
    ZervRonBuilder::new()
        .core_version(params.major, params.minor, params.patch)
        .with_extra_core_components(vec![
            Component::VarField(format!("{}.environment", shared_fields::CUSTOM)),
            Component::VarField(format!("{}.build_number", shared_fields::CUSTOM)),
        ])
        .with_build_components(vec![
            Component::VarTimestamp(timestamp_patterns::COMPACT_DATETIME.to_string()),
            Component::VarField(ron_fields::COMMIT_HASH_SHORT.to_string()),
        ])
        .with_vcs_data(&params.branch, &params.commit_hash, 0, false)
        .with_custom_vars(custom_vars)
        .to_ron()
}

/// Generate Zerv RON with PEP440 epoch
pub fn epoch_zerv_ron(
    params: &ZervParams,
    epoch: u64,
    label: PreReleaseLabel,
    number: Option<u64>,
    post: u64,
) -> String {
    ZervRonBuilder::new()
        .core_version(params.major, params.minor, params.patch)
        .with_epoch(epoch)
        .with_pre_release(label, number)
        .with_post(post)
        .with_build_components(vec![
            Component::VarField(ron_fields::BRANCH.to_string()),
            Component::VarTimestamp(timestamp_patterns::YYYY.to_string()),
            Component::VarTimestamp(timestamp_patterns::MM.to_string()),
        ])
        .with_vcs_data(
            &params.branch,
            &params.commit_hash,
            params.distance,
            params.dirty,
        )
        .to_ron()
}

/// Generate Zerv RON with development version
pub fn dev_zerv_ron(params: &ZervParams, dev: u64) -> String {
    ZervRonBuilder::new()
        .core_version(params.major, params.minor, params.patch)
        .with_dev(dev)
        .with_build_components(vec![
            Component::VarField(ron_fields::BRANCH.to_string()),
            Component::VarField(shared_fields::DISTANCE.to_string()),
            Component::VarField(ron_fields::COMMIT_HASH_SHORT.to_string()),
        ])
        .with_vcs_data(
            &params.branch,
            &params.commit_hash,
            params.distance,
            params.dirty,
        )
        .to_ron()
}

/// Generate Zerv RON with last version information
pub fn last_version_zerv_ron(
    params: &ZervParams,
    post: u64,
    last_branch: &str,
    last_commit_hash: &str,
    last_timestamp: u64,
) -> String {
    ZervRonBuilder::new()
        .core_version(params.major, params.minor, params.patch)
        .with_post(post)
        .with_build_components(vec![
            Component::VarField(shared_fields::LAST_BRANCH.to_string()),
            Component::VarField(shared_fields::LAST_COMMIT_HASH.to_string()),
            Component::VarTimestamp(shared_fields::LAST_TIMESTAMP.to_string()),
        ])
        .with_last_version(last_branch, last_commit_hash, last_timestamp)
        .with_vcs_data(
            &params.branch,
            &params.commit_hash,
            params.distance,
            params.dirty,
        )
        .to_ron()
}

/// Generate invalid RON that fails to parse (malformed RON syntax)
pub fn malformed_ron() -> String {
    format!(
        r#"(
        schema: (
            core: [var("{}"), var("{}"), var("{}")],
            extra_core: [],
            build: []
        ),
        vars: (
            major: Some(1),
            minor: Some(2),
            patch: Some(3)
            // Missing closing parenthesis - malformed RON
    "#,
        shared_fields::MAJOR,
        shared_fields::MINOR,
        shared_fields::PATCH
    )
}

/// Generate Zerv RON with missing core variables (for error testing)
/// Note: This creates a valid RON but with missing minor/patch in vars
pub fn missing_core_vars_zerv_ron(major: u64) -> String {
    let mut builder = ZervRonBuilder::new();
    builder.schema_mut().core = vec![
        Component::VarField(shared_fields::MAJOR.to_string()),
        Component::VarField(shared_fields::MINOR.to_string()),
        Component::VarField(shared_fields::PATCH.to_string()),
    ];
    builder.vars_mut().major = Some(major);
    // Intentionally missing minor and patch
    builder.to_ron()
}

/// Generate invalid RON with malformed syntax (for error testing)
pub fn invalid_syntax_ron() -> String {
    format!(
        r#"(
        schema: (
            core: [var("{}"), var("{}"), var("{}")],
            extra_core: [],
            build: []
        ),
        vars: (
            major: Some(1),
            minor: Some(2),
            patch: Some(3)
            // Missing closing parenthesis - malformed RON
    "#,
        shared_fields::MAJOR,
        shared_fields::MINOR,
        shared_fields::PATCH
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::Zerv;
    use ron::from_str;

    #[test]
    fn test_generators_module() {
        use PreReleaseLabel;

        // Test basic generator
        let basic_ron = basic_zerv_ron(1, 2, 3);
        let zerv: Zerv = from_str(&basic_ron).expect("Should parse");
        assert_eq!(zerv.vars.major, Some(1));
        assert_eq!(zerv.vars.minor, Some(2));
        assert_eq!(zerv.vars.patch, Some(3));

        // Test VCS generator
        let params = ZervParams::new(1, 2, 3, "main", "abc1234", 5, false);
        let vcs_ron = vcs_zerv_ron(&params, 5);
        let zerv: Zerv = from_str(&vcs_ron).expect("Should parse");
        assert_eq!(zerv.vars.post, Some(5));
        assert_eq!(zerv.vars.bumped_branch, Some("main".to_string()));

        // Test pre-release generator
        let params = ZervParams::new(1, 2, 3, "main", "abc1234", 0, false);
        let prerelease_ron = prerelease_zerv_ron(&params, PreReleaseLabel::Alpha, Some(1));
        let zerv: Zerv = from_str(&prerelease_ron).expect("Should parse");
        assert_eq!(
            zerv.vars.pre_release.as_ref().unwrap().label,
            PreReleaseLabel::Alpha
        );
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().number, Some(1));
    }
}
