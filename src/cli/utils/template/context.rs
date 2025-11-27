use crate::version::pep440::PEP440;
use crate::version::pep440::utils::pre_release_label_to_pep440_string;
use crate::version::semver::SemVer;
use crate::version::zerv::Zerv;

/// Template context for rendering
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ZervTemplateContext {
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
    // pub last_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_commit_hash_short: Option<String>,
    pub last_timestamp: Option<u64>,

    // Custom variables
    pub custom: serde_json::Value,

    // Formatted versions
    pub pep440: String,
    pub semver: String,

    // Parsed version components (nested objects)
    pub semver_obj: SemVerContext,
    pub pep440_obj: PEP440Context,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PreReleaseContext {
    pub label: String, // Current: "alpha", "beta", etc.
    pub number: Option<u64>,
    pub label_code: Option<String>, // Short coded format: "rc", "a", "b", etc.
    pub label_pep440: Option<String>, // PEP440 format: "rc", "a", "b", etc.
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SemVerContext {
    // 1.2.3-alpha.1.post.3.dev.5.something.else+build.456
    pub base_part: String,                // "1.2.3"
    pub pre_release_part: Option<String>, // "alpha.1.post.3.dev.5.something.else" or None
    pub build_part: Option<String>,       // "build.456" or None
    pub docker: String,                   // "1.2.3-alpha.1-build.456"
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PEP440Context {
    // 1.2.3a1.post3.dev5+build.456
    pub base_part: String,                // "1.2.3"
    pub pre_release_part: Option<String>, // "a1.post3.dev5" or None
    pub build_part: Option<String>,       // "build.456" or None
}

impl ZervTemplateContext {
    pub fn from_zerv(zerv: &Zerv) -> Self {
        let vars = &zerv.vars;

        let semver = SemVer::from(zerv.clone());
        let pep440 = PEP440::from(zerv.clone());

        Self {
            major: vars.major,
            minor: vars.minor,
            patch: vars.patch,
            epoch: vars.epoch,
            post: vars.post,
            dev: vars.dev,
            pre_release: vars.pre_release.as_ref().map(|pr| {
                let code_label = Some(pre_release_label_to_pep440_string(&pr.label).to_string());
                PreReleaseContext {
                    label: pr.label.label_str().to_string(),
                    number: pr.number,
                    label_code: code_label.clone(),
                    label_pep440: code_label,
                }
            }),
            distance: vars.distance,
            dirty: vars.dirty,
            bumped_branch: vars.bumped_branch.clone(),
            bumped_commit_hash: vars.bumped_commit_hash.clone(),
            bumped_commit_hash_short: vars.get_bumped_commit_hash_short(),
            bumped_timestamp: vars.bumped_timestamp,
            // last_branch: vars.last_branch.clone(),
            last_commit_hash: vars.last_commit_hash.clone(),
            last_commit_hash_short: vars.get_last_commit_hash_short(),
            last_timestamp: vars.last_timestamp,
            custom: vars.custom.clone(),
            pep440: pep440.to_string(),
            semver: semver.to_string(),

            // Parsed version components (nested objects)
            semver_obj: SemVerContext {
                base_part: semver.to_base_part(),
                pre_release_part: semver.to_pre_release_part(),
                build_part: semver.to_build_part(),
                docker: semver.to_docker_format(),
            },
            pep440_obj: PEP440Context {
                base_part: pep440.to_base_part(),
                pre_release_part: pep440.to_pre_release_part(),
                build_part: pep440.to_build_part(),
            },
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

        let context = ZervTemplateContext::from_zerv(zerv);

        assert_eq!(context.major, Some(1));
        assert_eq!(context.minor, Some(2));
        assert_eq!(context.patch, Some(3));
        assert_eq!(context.epoch, None);
        assert_eq!(context.post, None);
        assert_eq!(context.dev, None);
        assert_eq!(context.pre_release, None);
        assert_eq!(context.semver, "1.2.3");
        assert_eq!(context.pep440, "1.2.3");

        // Test new SemVer parsed components
        assert_eq!(context.semver_obj.base_part, "1.2.3");
        assert_eq!(context.semver_obj.pre_release_part, None);
        assert_eq!(context.semver_obj.build_part, None);
        assert_eq!(context.semver_obj.docker, "1.2.3");

        // Test new PEP440 parsed components
        assert_eq!(context.pep440_obj.base_part, "1.2.3");
        assert_eq!(context.pep440_obj.pre_release_part, None);
        assert_eq!(context.pep440_obj.build_part, None);
    }

    #[test]
    fn test_template_context_from_zerv_with_vcs() {
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            Some(5),
            Some(true),
            Some("feature/test".to_string()),
            Some("gabc123def456".to_string()),
            Some("abc123".to_string()),
            Some(1703123456),
            Some("main".to_string()),
        );
        let zerv = zerv_fixture.zerv();

        let context = ZervTemplateContext::from_zerv(zerv);

        assert_eq!(context.distance, Some(5));
        assert_eq!(context.dirty, Some(true));
        assert_eq!(context.bumped_branch, Some("feature/test".to_string()));
        assert_eq!(
            context.bumped_commit_hash,
            Some("gabc123def456".to_string())
        );
        assert_eq!(
            context.bumped_commit_hash_short,
            Some("gabc123d".to_string())
        );
        // bumped_timestamp is not set by with_vcs_data, so we won't test it here
    }

    #[test]
    fn test_template_context_from_zerv_with_pre_release() {
        let zerv_fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(crate::version::zerv::PreReleaseLabel::Alpha, Some(1));
        let zerv = zerv_fixture.zerv();

        let context = ZervTemplateContext::from_zerv(zerv);

        assert!(context.pre_release.is_some());
        let pre_release = context.pre_release.unwrap();
        assert_eq!(pre_release.label, "alpha");
        assert_eq!(pre_release.number, Some(1));
        assert_eq!(pre_release.label_code, Some("a".to_string()));
        assert_eq!(pre_release.label_pep440, Some("a".to_string()));

        // Test new SemVer parsed components
        assert_eq!(context.semver_obj.base_part, "1.2.3");
        assert_eq!(
            context.semver_obj.pre_release_part,
            Some("alpha.1".to_string())
        );
        assert_eq!(context.semver_obj.build_part, None);
        assert_eq!(context.semver_obj.docker, "1.2.3-alpha.1");

        // Test new PEP440 parsed components
        assert_eq!(context.pep440_obj.base_part, "1.2.3");
        assert_eq!(context.pep440_obj.pre_release_part, Some("a1".to_string()));
        assert_eq!(context.pep440_obj.build_part, None);
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

        let context = ZervTemplateContext::from_zerv(&zerv);

        assert_eq!(context.custom["build"], "42");
        assert_eq!(context.custom["metadata"], "test");
    }

    #[test]
    fn test_template_context_with_complex_version() {
        let zerv_fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(crate::version::zerv::PreReleaseLabel::Rc, Some(2));
        let zerv = zerv_fixture.zerv();

        let context = ZervTemplateContext::from_zerv(zerv);

        // Test pre-release context with label codes
        assert!(context.pre_release.is_some());
        let pre_release = context.pre_release.unwrap();
        assert_eq!(pre_release.label, "rc");
        assert_eq!(pre_release.number, Some(2));
        assert_eq!(pre_release.label_code, Some("rc".to_string()));
        assert_eq!(pre_release.label_pep440, Some("rc".to_string()));

        // Test SemVer components
        assert_eq!(context.semver_obj.base_part, "1.2.3");
        assert_eq!(
            context.semver_obj.pre_release_part,
            Some("rc.2".to_string())
        );
        assert_eq!(context.semver_obj.build_part, None);
        assert_eq!(context.semver_obj.docker, "1.2.3-rc.2");

        // Test PEP440 components
        assert_eq!(context.pep440_obj.base_part, "1.2.3");
        assert_eq!(context.pep440_obj.pre_release_part, Some("rc2".to_string()));
        assert_eq!(context.pep440_obj.build_part, None);
    }

    #[test]
    fn test_template_context_beta_pre_release() {
        let zerv_fixture = ZervFixture::new()
            .with_version(2, 1, 0)
            .with_pre_release(crate::version::zerv::PreReleaseLabel::Beta, None);
        let zerv = zerv_fixture.zerv();

        let context = ZervTemplateContext::from_zerv(zerv);

        // Test pre-release context with label codes
        assert!(context.pre_release.is_some());
        let pre_release = context.pre_release.unwrap();
        assert_eq!(pre_release.label, "beta");
        assert_eq!(pre_release.number, None);
        assert_eq!(pre_release.label_code, Some("b".to_string()));
        assert_eq!(pre_release.label_pep440, Some("b".to_string()));

        // Test SemVer components
        assert_eq!(context.semver_obj.base_part, "2.1.0");
        assert_eq!(
            context.semver_obj.pre_release_part,
            Some("beta".to_string())
        );
        assert_eq!(context.semver_obj.build_part, None);
        assert_eq!(context.semver_obj.docker, "2.1.0-beta");

        // Test PEP440 components
        assert_eq!(context.pep440_obj.base_part, "2.1.0");
        assert_eq!(context.pep440_obj.pre_release_part, Some("b0".to_string()));
        assert_eq!(context.pep440_obj.build_part, None);
    }

    #[test]
    fn test_template_context_single_digit_version() {
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0);
        let zerv = zerv_fixture.zerv();

        let context = ZervTemplateContext::from_zerv(zerv);

        // Test basic components
        assert_eq!(context.major, Some(1));
        assert_eq!(context.minor, Some(0));
        assert_eq!(context.patch, Some(0));

        // Test SemVer components
        assert_eq!(context.semver_obj.base_part, "1.0.0");
        assert_eq!(context.semver_obj.pre_release_part, None);
        assert_eq!(context.semver_obj.build_part, None);
        assert_eq!(context.semver_obj.docker, "1.0.0");

        // Test PEP440 components
        assert_eq!(context.pep440_obj.base_part, "1.0.0");
        assert_eq!(context.pep440_obj.pre_release_part, None);
        assert_eq!(context.pep440_obj.build_part, None);
    }

    #[test]
    fn test_pre_release_context_all_labels() {
        use crate::version::zerv::PreReleaseLabel;

        let test_cases = vec![
            (PreReleaseLabel::Alpha, Some("alpha"), Some("a")),
            (PreReleaseLabel::Beta, Some("beta"), Some("b")),
            (PreReleaseLabel::Rc, Some("rc"), Some("rc")),
        ];

        for (label, expected_label, expected_pep440_label) in test_cases {
            let zerv_fixture = ZervFixture::new()
                .with_version(1, 0, 0)
                .with_pre_release(label, Some(1));
            let zerv = zerv_fixture.zerv();

            let context = ZervTemplateContext::from_zerv(zerv);

            assert!(context.pre_release.is_some());
            let pre_release = context.pre_release.unwrap();
            assert_eq!(pre_release.label, expected_label.unwrap());
            assert_eq!(pre_release.number, Some(1));
            assert_eq!(
                pre_release.label_code,
                Some(expected_pep440_label.unwrap().to_string())
            );
            assert_eq!(
                pre_release.label_pep440,
                Some(expected_pep440_label.unwrap().to_string())
            );

            // Test PEP440 pre-release part format
            assert_eq!(
                context.pep440_obj.pre_release_part,
                Some(format!("{}1", expected_pep440_label.unwrap()))
            );
        }
    }
}
