use crate::version::pep440::PEP440;
use crate::version::semver::SemVer;
use crate::version::zerv::Zerv;

/// Template context for rendering
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TemplateContext {
    // Core version fields
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub epoch: Option<u64>,

    // Metadata fields
    pub post: Option<u64>,
    pub dev: Option<u64>,

    // Pre-release fields
    pub pre_release: Option<PreReleaseContext>,

    // VCS fields
    pub distance: Option<u64>,
    pub dirty: Option<bool>,
    pub bumped_branch: Option<String>,
    pub bumped_commit_hash: Option<String>,
    pub bumped_commit_hash_short: Option<String>,
    pub bumped_timestamp: Option<u64>,

    // Last version fields
    pub last_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_commit_hash_short: Option<String>,
    pub last_timestamp: Option<u64>,

    // Custom variables
    pub custom: serde_json::Value,

    // Formatted versions
    pub pep440: String,
    pub semver: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PreReleaseContext {
    pub label: String,
    pub number: Option<u64>,
}

impl TemplateContext {
    pub fn from_zerv(zerv: &Zerv) -> Self {
        let vars = &zerv.vars;
        Self {
            major: vars.major,
            minor: vars.minor,
            patch: vars.patch,
            epoch: vars.epoch,
            post: vars.post,
            dev: vars.dev,
            pre_release: vars.pre_release.as_ref().map(|pr| PreReleaseContext {
                label: pr.label.label_str().to_string(),
                number: pr.number,
            }),
            distance: vars.distance,
            dirty: vars.dirty,
            bumped_branch: vars.bumped_branch.clone(),
            bumped_commit_hash: vars.bumped_commit_hash.clone(),
            bumped_commit_hash_short: vars.get_bumped_commit_hash_short(),
            bumped_timestamp: vars.bumped_timestamp,
            last_branch: vars.last_branch.clone(),
            last_commit_hash: vars.last_commit_hash.clone(),
            last_commit_hash_short: vars.get_last_commit_hash_short(),
            last_timestamp: vars.last_timestamp,
            custom: vars.custom.clone(),
            pep440: PEP440::from(zerv.clone()).to_string(),
            semver: SemVer::from(zerv.clone()).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::zerv::{
        Zerv,
        ZervSchema,
        ZervVars,
    };

    #[test]
    fn test_template_context_from_zerv_basic() {
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let context = TemplateContext::from_zerv(zerv);

        assert_eq!(context.major, Some(1));
        assert_eq!(context.minor, Some(2));
        assert_eq!(context.patch, Some(3));
        assert_eq!(context.epoch, None);
        assert_eq!(context.post, None);
        assert_eq!(context.dev, None);
        assert_eq!(context.pre_release, None);
        assert_eq!(context.semver, "1.2.3");
        assert_eq!(context.pep440, "1.2.3");
    }

    #[test]
    fn test_template_context_from_zerv_with_vcs() {
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            Some(5),
            Some(true),
            Some("feature/test".to_string()),
            Some("abc123def456".to_string()),
            Some("abc123".to_string()),
            Some(1703123456),
            Some("main".to_string()),
        );
        let zerv = zerv_fixture.zerv();

        let context = TemplateContext::from_zerv(zerv);

        assert_eq!(context.distance, Some(5));
        assert_eq!(context.dirty, Some(true));
        assert_eq!(context.bumped_branch, Some("feature/test".to_string()));
        assert_eq!(context.bumped_commit_hash, Some("abc123def456".to_string()));
        assert_eq!(
            context.bumped_commit_hash_short,
            Some("abc123d".to_string())
        );
        // bumped_timestamp is not set by with_vcs_data, so we won't test it here
    }

    #[test]
    fn test_template_context_from_zerv_with_pre_release() {
        let zerv_fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(crate::version::zerv::PreReleaseLabel::Alpha, Some(1));
        let zerv = zerv_fixture.zerv();

        let context = TemplateContext::from_zerv(zerv);

        assert!(context.pre_release.is_some());
        let pre_release = context.pre_release.unwrap();
        assert_eq!(pre_release.label, "alpha");
        assert_eq!(pre_release.number, Some(1));
    }

    #[test]
    fn test_template_context_from_zerv_with_custom_vars() {
        let vars = ZervVars {
            major: Some(1),
            minor: Some(0),
            patch: Some(0),
            custom: serde_json::json!({
                "build": "42",
                "metadata": "test"
            }),
            ..Default::default()
        };

        let schema = ZervSchema::semver_default().unwrap();
        let zerv = Zerv::new(schema, vars).unwrap();

        let context = TemplateContext::from_zerv(&zerv);

        assert_eq!(context.custom["build"], "42");
        assert_eq!(context.custom["metadata"], "test");
    }
}
