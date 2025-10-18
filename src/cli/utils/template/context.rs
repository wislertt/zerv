use crate::version::Zerv;
use crate::version::pep440::PEP440;
use crate::version::semver::SemVer;

/// Template context for Handlebars rendering
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
    use rstest::rstest;

    use super::*;
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::zerv::PreReleaseLabel;

    fn basic_zerv() -> ZervFixture {
        ZervFixture::new()
    }

    fn vcs_zerv() -> ZervFixture {
        ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            0,
            true,
            "main".to_string(),
            "abcdef123456".to_string(),
            "xyz789".to_string(),
            1234567890,
            "main".to_string(),
        )
    }

    fn pre_release_zerv() -> ZervFixture {
        ZervFixture::new().with_pre_release(PreReleaseLabel::Alpha, Some(1))
    }

    fn custom_vars_zerv() -> ZervFixture {
        use crate::version::zerv::{
            Zerv,
            ZervSchema,
            ZervVars,
        };

        let vars = ZervVars {
            major: Some(2),
            minor: Some(1),
            patch: Some(0),
            custom: serde_json::json!({
                "build_id": 123,
                "env": "prod",
                "metadata": {
                    "author": "ci",
                    "timestamp": 1703123456
                }
            }),
            ..Default::default()
        };

        let schema = ZervSchema::semver_default().unwrap();
        let zerv = Zerv::new(schema, vars).unwrap();
        ZervFixture::from(zerv)
    }

    fn basic_context() -> TemplateContext {
        TemplateContext {
            major: Some(1),
            minor: Some(0),
            patch: Some(0),
            epoch: None,
            post: None,
            dev: None,
            pre_release: None,
            distance: None,
            dirty: None,
            bumped_branch: None,
            bumped_commit_hash: None,
            bumped_commit_hash_short: None,
            bumped_timestamp: None,
            last_branch: None,
            last_commit_hash: None,
            last_commit_hash_short: None,
            last_timestamp: None,
            custom: serde_json::Value::Null,
            pep440: "1.0.0".to_string(),
            semver: "1.0.0".to_string(),
        }
    }

    fn vcs_context() -> TemplateContext {
        TemplateContext {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            epoch: None,
            post: None,
            dev: None,
            pre_release: None,
            distance: Some(0),
            dirty: Some(true),
            bumped_branch: Some("main".to_string()),
            bumped_commit_hash: Some("abcdef123456".to_string()),
            bumped_commit_hash_short: Some("abcdef1".to_string()),
            bumped_timestamp: None,
            last_branch: Some("main".to_string()),
            last_commit_hash: Some("xyz789".to_string()),
            last_commit_hash_short: Some("xyz789".to_string()),
            last_timestamp: Some(1234567890),
            custom: serde_json::Value::Null,
            pep440: "1.2.3".to_string(),
            semver: "1.2.3".to_string(),
        }
    }

    fn pre_release_context() -> TemplateContext {
        TemplateContext {
            major: Some(1),
            minor: Some(0),
            patch: Some(0),
            epoch: None,
            post: None,
            dev: None,
            pre_release: Some(PreReleaseContext {
                label: "alpha".to_string(),
                number: Some(1),
            }),
            distance: None,
            dirty: None,
            bumped_branch: None,
            bumped_commit_hash: None,
            bumped_commit_hash_short: None,
            bumped_timestamp: None,
            last_branch: None,
            last_commit_hash: None,
            last_commit_hash_short: None,
            last_timestamp: None,
            custom: serde_json::Value::Null,
            pep440: "1.0.0a1".to_string(),
            semver: "1.0.0-alpha.1".to_string(),
        }
    }

    fn custom_vars_context() -> TemplateContext {
        TemplateContext {
            major: Some(2),
            minor: Some(1),
            patch: Some(0),
            epoch: None,
            post: None,
            dev: None,
            pre_release: None,
            distance: None,
            dirty: None,
            bumped_branch: None,
            bumped_commit_hash: None,
            bumped_commit_hash_short: None,
            bumped_timestamp: None,
            last_branch: None,
            last_commit_hash: None,
            last_commit_hash_short: None,
            last_timestamp: None,
            custom: serde_json::json!({
                "build_id": 123,
                "env": "prod",
                "metadata": {
                    "author": "ci",
                    "timestamp": 1703123456
                }
            }),
            pep440: "2.1.0".to_string(),
            semver: "2.1.0".to_string(),
        }
    }

    #[rstest]
    #[case::basic(basic_zerv(), basic_context())]
    #[case::with_vcs(vcs_zerv(), vcs_context())]
    #[case::with_pre_release(pre_release_zerv(), pre_release_context())]
    #[case::with_custom_vars(custom_vars_zerv(), custom_vars_context())]
    fn test_template_context_from_zerv(
        #[case] fixture: ZervFixture,
        #[case] expected: TemplateContext,
    ) {
        let zerv = fixture.build();
        let context = TemplateContext::from_zerv(&zerv);
        assert_eq!(context, expected);
    }
}
