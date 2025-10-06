use super::{
    BuildMetadata,
    PreReleaseIdentifier,
    SemVer,
};
use crate::constants::ron_fields;
use crate::version::zerv::bump::precedence::PrecedenceOrder;
use crate::version::zerv::core::PreReleaseLabel;
use crate::version::zerv::{
    Component,
    PreReleaseVar,
    Zerv,
    ZervSchema,
    ZervVars,
};

type ProcessResult = (
    Option<PreReleaseVar>,
    Vec<Component>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
);

struct PreReleaseProcessor {
    pre_release: Option<PreReleaseVar>,
    extra_core: Vec<Component>,
    epoch: Option<u64>,
    post: Option<u64>,
    dev: Option<u64>,
}

impl PreReleaseProcessor {
    fn new() -> Self {
        Self {
            pre_release: None,
            extra_core: Vec::new(),
            epoch: None,
            post: None,
            dev: None,
        }
    }

    fn try_special_pattern(&mut self, pr: &[PreReleaseIdentifier], i: usize) -> bool {
        if i + 1 >= pr.len() {
            return false;
        }

        if let (PreReleaseIdentifier::String(label), PreReleaseIdentifier::Integer(num)) =
            (&pr[i], &pr[i + 1])
        {
            match label.as_str() {
                "epoch" if self.epoch.is_none() => {
                    self.epoch = Some(*num);
                    self.extra_core
                        .push(Component::VarField(ron_fields::EPOCH.to_string()));
                    true
                }
                "dev" if self.dev.is_none() => {
                    self.dev = Some(*num);
                    self.extra_core
                        .push(Component::VarField(ron_fields::DEV.to_string()));
                    true
                }
                "post" if self.post.is_none() => {
                    self.post = Some(*num);
                    self.extra_core
                        .push(Component::VarField(ron_fields::POST.to_string()));
                    true
                }
                _ => self.try_pre_release_pattern(label, Some(*num)),
            }
        } else {
            false
        }
    }

    fn try_pre_release_pattern(&mut self, label: &str, number: Option<u64>) -> bool {
        if let Some(normalized_label) = PreReleaseLabel::try_from_str(label)
            && self.pre_release.is_none()
        {
            self.pre_release = Some(PreReleaseVar {
                label: normalized_label,
                number,
            });
            self.extra_core
                .push(Component::VarField(ron_fields::PRE_RELEASE.to_string()));
            return true;
        }
        false
    }

    fn add_regular_component(&mut self, item: &PreReleaseIdentifier) {
        self.extra_core.push(match item {
            PreReleaseIdentifier::String(s) => Component::String(s.clone()),
            PreReleaseIdentifier::Integer(n) => Component::Integer(*n),
        });
    }

    fn process(mut self, pr: &[PreReleaseIdentifier]) -> ProcessResult {
        let mut i = 0;
        while i < pr.len() {
            if self.try_special_pattern(pr, i) {
                i += 2;
            } else if let PreReleaseIdentifier::String(label) = &pr[i] {
                if !self.try_pre_release_pattern(label, None) {
                    self.add_regular_component(&pr[i]);
                }
                i += 1;
            } else {
                self.add_regular_component(&pr[i]);
                i += 1;
            }
        }
        (
            self.pre_release,
            self.extra_core,
            self.epoch,
            self.post,
            self.dev,
        )
    }
}

impl From<SemVer> for Zerv {
    fn from(semver: SemVer) -> Self {
        let build = semver
            .build_metadata
            .as_ref()
            .map(|metadata| {
                metadata
                    .iter()
                    .map(|m| match m {
                        BuildMetadata::String(s) => Component::String(s.clone()),
                        BuildMetadata::Integer(i) => Component::Integer(*i),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let (pre_release, extra_core, epoch, post, dev): ProcessResult = semver
            .pre_release
            .as_ref()
            .map(|pr| PreReleaseProcessor::new().process(pr))
            .unwrap_or_default();

        Zerv {
            schema: ZervSchema {
                core: vec![
                    Component::VarField(ron_fields::MAJOR.to_string()),
                    Component::VarField(ron_fields::MINOR.to_string()),
                    Component::VarField(ron_fields::PATCH.to_string()),
                ],
                extra_core,
                build,
                precedence_order: PrecedenceOrder::default(),
            },
            vars: ZervVars {
                major: Some(semver.major),
                minor: Some(semver.minor),
                patch: Some(semver.patch),
                epoch,
                pre_release,
                post,
                dev,
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::zerv::zerv_semver::to;
    use crate::version::zerv::Zerv;

    #[rstest]
    #[case("1.2.3", to::v1_2_3().build())]
    #[case("1.0.0-alpha.1", to::v1_0_0_a1().build())]
    #[case("1.0.0-something.1", to::v1_0_0_something_1().build())]
    #[case("1.0.0+build.123", to::v1_0_0_build().build())]
    #[case("1.0.0-alpha.1+build.123", to::v1_0_0_a1_build().build())]
    #[case(
        "1.0.0-alpha.1.lowercase.4.UPPERCASE.5+build.123",
        to::v1_0_0_a1_complex().build()
    )]
    #[case("1.0.0-foo.bar.beta.2.baz", to::v1_0_0_foo_bar_beta_2_baz().build())]
    #[case("1.0.0-alpha.1.beta.2", to::v1_0_0_alpha_1_beta_2().build())]
    #[case("1.0.0-rc.1.alpha.2.beta.3", to::v1_0_0_rc_1_alpha_2_beta_3().build())]
    #[case("1.0.0-pre.alpha.1", to::v1_0_0_pre_alpha_1().build())]
    #[case("1.0.0-test.alpha.beta.rc.1", to::v1_0_0_test_alpha_beta_rc_1().build())]
    #[case("1.0.0-ALPHA.1", to::v1_0_0_alpha_1().build())]
    #[case("1.0.0-epoch.1", to::v1_0_0_epoch_1().build())]
    #[case("1.0.0-post.1", to::v1_0_0_post_1().build())]
    #[case("1.0.0-dev.1", to::v1_0_0_dev_1().build())]
    // Case variations - Beta
    #[case("1.0.0-BETA.2", to::v1_0_0_beta_2().build())]
    // Case variations - RC
    #[case("1.0.0-RC.3", to::v1_0_0_rc_3().build())]
    // Case variations - Preview (treated as regular string)
    #[case("1.0.0-Preview.4", to::v1_0_0_preview_4().build())]
    // Case variations - short forms
    #[case("1.0.0-a.1", to::v1_0_0_a_1().build())]
    #[case("1.0.0-b.2", to::v1_0_0_b_2().build())]
    #[case("1.0.0-c.3", to::v1_0_0_c_3().build())]
    // Case variations - without numbers
    #[case("1.0.0-alpha", to::v1_0_0_alpha().build())]
    #[case("1.0.0-beta", to::v1_0_0_beta().build())]
    #[case("1.0.0-rc", to::v1_0_0_rc().build())]
    // Case variations - with zero
    #[case("1.0.0-alpha.0", to::v1_0_0_alpha_0().build())]
    #[case("1.0.0-beta.0", to::v1_0_0_beta_0().build())]
    // Case variations - with prefix (alpha/beta found later in sequence)
    #[case("1.0.0-foo.1.alpha", to::v1_0_0_foo_1_alpha().build())]
    #[case("1.0.0-bar.2.beta", to::v1_0_0_bar_2_beta().build())]
    // Epoch handling
    #[case("1.0.0-epoch.1", to::v1_0_0_epoch_1().build())]
    #[case("1.0.0-epoch.5", to::v1_0_0_epoch_5().build())]
    #[case("1.0.0-epoch.0", to::v1_0_0_epoch_0().build())]
    #[case("1.0.0-epoch.999", to::v1_0_0_epoch_999().build())]
    // Post handling
    #[case("1.0.0-post.1", to::v1_0_0_post_1().build())]
    #[case("1.0.0-post.5", to::v1_0_0_post_5().build())]
    #[case("1.0.0-post.0", to::v1_0_0_post_0().build())]
    // Dev handling
    #[case("1.0.0-dev.1", to::v1_0_0_dev_1().build())]
    #[case("1.0.0-dev.0", to::v1_0_0_dev_0().build())]
    #[case("1.0.0-dev.10", to::v1_0_0_dev_10().build())]
    // Complex combinations
    #[case("1.0.0-epoch.2.alpha.1", to::v1_0_0_epoch_2_alpha_1().build())]
    #[case("1.0.0-epoch.3.beta.2", to::v1_0_0_epoch_3_beta_2().build())]
    #[case("1.0.0-epoch.1.rc.5", to::v1_0_0_epoch_1_rc_5().build())]
    #[case("1.0.0-epoch.4.alpha", to::v1_0_0_epoch_4_alpha().build())]
    #[case("1.0.0-post.1.dev.2", to::v1_0_0_post_1_dev_2().build())]
    #[case("1.0.0-dev.3.post.4", to::v1_0_0_dev_3_post_4().build())]
    #[case("1.0.0-alpha.1.post.2", to::v1_0_0_alpha_1_post_2().build())]
    #[case("1.0.0-beta.3.post.1", to::v1_0_0_beta_3_post_1().build())]
    #[case("1.0.0-rc.2.post.5", to::v1_0_0_rc_2_post_5().build())]
    #[case("1.0.0-alpha.1.dev.2", to::v1_0_0_alpha_1_dev_2().build())]
    #[case("1.0.0-beta.2.dev.1", to::v1_0_0_beta_2_dev_1().build())]
    #[case("1.0.0-rc.1.dev.3", to::v1_0_0_rc_1_dev_3().build())]
    #[case("1.0.0-alpha.1.post.2.dev.3", to::v1_0_0_alpha_1_post_2_dev_3().build())]
    #[case("1.0.0-beta.2.dev.1.post.3", to::v1_0_0_beta_2_dev_1_post_3().build())]
    #[case("1.0.0-rc.1.post.1.dev.1", to::v1_0_0_rc_1_post_1_dev_1().build())]
    #[case("1.0.0-epoch.2.post.1.dev.3", to::v1_0_0_epoch_2_post_1_dev_3().build())]
    #[case("1.0.0-epoch.1.dev.2.post.1", to::v1_0_0_epoch_1_dev_2_post_1().build())]
    #[case("1.0.0-epoch.3.alpha.1.post.2.dev.1", to::v1_0_0_epoch_3_alpha_1_post_2_dev_1().build())]
    #[case("1.0.0-epoch.1.beta.2.dev.3.post.1", to::v1_0_0_epoch_1_beta_2_dev_3_post_1().build())]
    // Build metadata combinations
    #[case("1.0.0-epoch.1+build.123", to::v1_0_0_epoch_1_build().build())]
    #[case("1.0.0-post.1+build.456", to::v1_0_0_post_1_build().build())]
    #[case("1.0.0-dev.2+build.789", to::v1_0_0_dev_2_build().build())]
    #[case("1.0.0-epoch.2.alpha.1+build.abc", to::v1_0_0_epoch_2_alpha_1_build().build())]
    // Mixed order cases
    #[case("1.0.0-foo.epoch.1.alpha.2", to::v1_0_0_foo_epoch_1_alpha_2().build())]
    #[case("1.0.0-epoch.1.foo.post.2", to::v1_0_0_epoch_1_foo_post_2().build())]
    #[case("1.0.0-bar.dev.1.epoch.2", to::v1_0_0_bar_dev_1_epoch_2().build())]
    fn test_semver_to_zerv_conversion(#[case] semver_str: &str, #[case] expected: Zerv) {
        let semver: SemVer = semver_str.parse().unwrap();
        let zerv: Zerv = semver.into();
        assert_eq!(zerv, expected);
    }

    #[rstest]
    #[case("1.0.0")]
    #[case("2.1.0-beta.1")]
    #[case("1.0.0+build.123")]
    #[case("2.1.0-beta.1+build.123")]
    #[case("1.0.0-alpha.1.post.2.dev.3")]
    fn test_round_trip_conversion(#[case] version_str: &str) {
        let original: SemVer = version_str.parse().unwrap();
        let zerv: Zerv = original.clone().into();
        let converted: SemVer = zerv.into();

        assert_eq!(original.to_string(), converted.to_string());
    }
}
