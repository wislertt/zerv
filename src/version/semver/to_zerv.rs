use super::{BuildMetadata, PreReleaseIdentifier, SemVer};
use crate::constants::ron_fields;
use crate::version::zerv::core::PreReleaseLabel;
use crate::version::zerv::{Component, PreReleaseVar, Zerv, ZervSchema, ZervVars};

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
    use super::*;
    use crate::version::zerv::PreReleaseLabel;
    use rstest::rstest;

    use crate::test_utils::zerv::ZervFixture;

    #[rstest]
    // Basic conversions
    #[case("1.2.3", ZervFixture::sem_zerv_1_2_3())]
    #[case("1.0.0-alpha.1", ZervFixture::sem_zerv_1_0_0_alpha_1())]
    #[case("1.0.0-something.1", ZervFixture::sem_zerv_1_0_0_something_1())]
    #[case("1.0.0+build.123", ZervFixture::sem_zerv_1_0_0_build_123())]
    #[case(
        "1.0.0-alpha.1+build.123",
        ZervFixture::sem_zerv_1_0_0_alpha_1_build_123()
    )]
    #[case(
        "1.0.0-alpha.1.lowercase.4.UPPERCASE.5+build.123",
        ZervFixture::sem_zerv_1_0_0_alpha_1_lowercase_4_uppercase_5_build_123()
    )]
    #[case(
        "1.0.0-foo.bar.beta.2.baz",
        ZervFixture::sem_zerv_1_0_0_foo_bar_beta_2_baz()
    )]
    #[case("1.0.0-alpha.1.beta.2", ZervFixture::sem_zerv_1_0_0_alpha_1_beta_2())]
    #[case(
        "1.0.0-rc.1.alpha.2.beta.3",
        ZervFixture::sem_zerv_1_0_0_rc_1_alpha_2_beta_3()
    )]
    #[case("1.0.0-pre.alpha.1", ZervFixture::sem_zerv_1_0_0_rc_alpha_1())]
    #[case(
        "1.0.0-test.alpha.beta.rc.1",
        ZervFixture::sem_zerv_1_0_0_test_alpha_beta_rc_1()
    )]
    // Case variations
    #[case(
        "1.0.0-ALPHA.1",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1))
    )]
    #[case(
        "1.0.0-BETA.2",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(2))
    )]
    #[case(
        "1.0.0-RC.3",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, Some(3))
    )]
    #[case(
        "1.0.0-Preview.4",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, Some(4))
    )]
    #[case(
        "1.0.0-a.1",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1))
    )]
    #[case(
        "1.0.0-b.2",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(2))
    )]
    #[case(
        "1.0.0-c.3",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, Some(3))
    )]
    #[case(
        "1.0.0-alpha",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, None)
    )]
    #[case(
        "1.0.0-beta",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, None)
    )]
    #[case(
        "1.0.0-rc",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, None)
    )]
    #[case(
        "1.0.0-alpha.0",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(0))
    )]
    #[case(
        "1.0.0-beta.0",
        ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(0))
    )]
    #[case("1.0.0-foo.1.alpha", ZervFixture::sem_zerv_1_0_0_foo_1_alpha())]
    #[case("1.0.0-bar.2.beta", ZervFixture::sem_zerv_1_0_0_bar_2_beta())]
    // Epoch handling
    #[case("1.0.0-epoch.1", ZervFixture::zerv_1_0_0_with_epoch(1))]
    #[case("1.0.0-epoch.5", ZervFixture::zerv_1_0_0_with_epoch(5))]
    #[case("1.0.0-epoch.0", ZervFixture::zerv_1_0_0_with_epoch(0))]
    #[case("1.0.0-epoch.999", ZervFixture::zerv_1_0_0_with_epoch(999))]
    // Post handling
    #[case("1.0.0-post.1", ZervFixture::zerv_1_0_0_with_post(1))]
    #[case("1.0.0-post.5", ZervFixture::zerv_1_0_0_with_post(5))]
    #[case("1.0.0-post.0", ZervFixture::zerv_1_0_0_with_post(0))]
    // Dev handling
    #[case("1.0.0-dev.1", ZervFixture::zerv_1_0_0_with_dev(1))]
    #[case("1.0.0-dev.0", ZervFixture::zerv_1_0_0_with_dev(0))]
    #[case("1.0.0-dev.10", ZervFixture::zerv_1_0_0_with_dev(10))]
    // Epoch + pre-release combinations
    #[case(
        "1.0.0-epoch.2.alpha.1",
        ZervFixture::zerv_1_0_0_with_epoch_and_pre_release(2, PreReleaseLabel::Alpha, Some(1))
    )]
    #[case(
        "1.0.0-epoch.3.beta.2",
        ZervFixture::zerv_1_0_0_with_epoch_and_pre_release(3, PreReleaseLabel::Beta, Some(2))
    )]
    #[case(
        "1.0.0-epoch.1.rc.5",
        ZervFixture::zerv_1_0_0_with_epoch_and_pre_release(1, PreReleaseLabel::Rc, Some(5))
    )]
    #[case(
        "1.0.0-epoch.4.alpha",
        ZervFixture::zerv_1_0_0_with_epoch_and_pre_release(4, PreReleaseLabel::Alpha, None)
    )]
    // Post + dev combinations
    #[case("1.0.0-post.1.dev.2", ZervFixture::zerv_1_0_0_with_post_and_dev(1, 2))]
    #[case("1.0.0-dev.3.post.4", ZervFixture::zerv_1_0_0_with_dev_and_post(3, 4))]
    // Pre-release + post combinations
    #[case(
        "1.0.0-alpha.1.post.2",
        ZervFixture::zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Alpha, Some(1), 2)
    )]
    #[case(
        "1.0.0-beta.3.post.1",
        ZervFixture::zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Beta, Some(3), 1)
    )]
    #[case(
        "1.0.0-rc.2.post.5",
        ZervFixture::zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Rc, Some(2), 5)
    )]
    // Pre-release + dev combinations
    #[case(
        "1.0.0-alpha.1.dev.2",
        ZervFixture::zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Alpha, Some(1), 2)
    )]
    #[case(
        "1.0.0-beta.2.dev.1",
        ZervFixture::zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Beta, Some(2), 1)
    )]
    #[case(
        "1.0.0-rc.1.dev.3",
        ZervFixture::zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Rc, Some(1), 3)
    )]
    // Triple combinations
    #[case(
        "1.0.0-alpha.1.post.2.dev.3",
        ZervFixture::zerv_1_0_0_with_pre_release_post_and_dev(
            PreReleaseLabel::Alpha,
            Some(1),
            2,
            3
        )
    )]
    #[case(
        "1.0.0-beta.2.dev.1.post.3",
        ZervFixture::zerv_1_0_0_with_pre_release_dev_and_post(
            PreReleaseLabel::Beta,
            Some(2),
            1,
            3
        )
    )]
    #[case(
        "1.0.0-rc.1.post.1.dev.1",
        ZervFixture::zerv_1_0_0_with_pre_release_post_and_dev(PreReleaseLabel::Rc, Some(1), 1, 1)
    )]
    // Epoch + post + dev combinations
    #[case(
        "1.0.0-epoch.2.post.1.dev.3",
        ZervFixture::zerv_1_0_0_with_epoch_post_and_dev(2, 1, 3)
    )]
    #[case(
        "1.0.0-epoch.1.dev.2.post.1",
        ZervFixture::zerv_1_0_0_with_epoch_dev_and_post(1, 2, 1)
    )]
    // All components together
    #[case(
        "1.0.0-epoch.3.alpha.1.post.2.dev.1",
        ZervFixture::zerv_1_0_0_with_all_components(3, PreReleaseLabel::Alpha, Some(1), 2, 1)
    )]
    #[case(
        "1.0.0-epoch.1.beta.2.dev.3.post.1",
        ZervFixture::zerv_1_0_0_with_all_components_reordered(
            1,
            PreReleaseLabel::Beta,
            Some(2),
            3,
            1
        )
    )]
    // With build metadata
    #[case(
        "1.0.0-epoch.1+build.123",
        ZervFixture::zerv_1_0_0_with_epoch_and_build(1)
    )]
    #[case(
        "1.0.0-post.1+build.456",
        ZervFixture::zerv_1_0_0_with_post_and_build(1)
    )]
    #[case("1.0.0-dev.2+build.789", ZervFixture::zerv_1_0_0_with_dev_and_build(2))]
    #[case(
        "1.0.0-epoch.2.alpha.1+build.abc",
        ZervFixture::zerv_1_0_0_with_epoch_pre_release_and_build(
            2,
            PreReleaseLabel::Alpha,
            Some(1)
        )
    )]
    // Mixed with other identifiers
    #[case(
        "1.0.0-foo.epoch.1.alpha.2",
        ZervFixture::zerv_1_0_0_with_foo_epoch_and_alpha_original_order(1, 2)
    )]
    #[case(
        "1.0.0-epoch.1.foo.post.2",
        ZervFixture::zerv_1_0_0_with_epoch_foo_and_post(1, 2)
    )]
    #[case(
        "1.0.0-bar.dev.1.epoch.2",
        ZervFixture::zerv_1_0_0_with_bar_dev_and_epoch_original_order(1, 2)
    )]
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
