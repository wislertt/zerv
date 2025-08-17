use super::{BuildMetadata, PreReleaseIdentifier, SemVer};
use crate::version::zerv::{
    Component, PreReleaseVar, Zerv, ZervSchema, ZervVars, normalize_pre_release_label,
};

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

        let (pre_release, extra_core, epoch, post, dev) = if let Some(pr) = &semver.pre_release {
            let mut pre_release = None;
            let mut extra_core = Vec::new();
            let mut epoch = None;
            let mut post = None;
            let mut dev = None;
            let mut skip_next = false;

            for (i, item) in pr.iter().enumerate() {
                if skip_next {
                    skip_next = false;
                    continue;
                }

                // Check for epoch+number pattern
                if i + 1 < pr.len()
                    && let (PreReleaseIdentifier::String(label), PreReleaseIdentifier::Integer(num)) =
                        (item, &pr[i + 1])
                    && label == "epoch"
                    && epoch.is_none()
                {
                    epoch = Some(*num);
                    extra_core.push(Component::VarField("epoch".to_string()));
                    skip_next = true;
                    continue;
                }

                // Check for dev+number pattern
                if i + 1 < pr.len()
                    && let (PreReleaseIdentifier::String(label), PreReleaseIdentifier::Integer(num)) =
                        (item, &pr[i + 1])
                    && label == "dev"
                    && dev.is_none()
                {
                    dev = Some(*num);
                    extra_core.push(Component::VarField("dev".to_string()));
                    skip_next = true;
                    continue;
                }

                // Check for post+number pattern
                if i + 1 < pr.len()
                    && let (PreReleaseIdentifier::String(label), PreReleaseIdentifier::Integer(num)) =
                        (item, &pr[i + 1])
                    && label == "post"
                    && post.is_none()
                {
                    post = Some(*num);
                    extra_core.push(Component::VarField("post".to_string()));
                    skip_next = true;
                    continue;
                }

                // Check for keyword+number pattern
                if i + 1 < pr.len()
                    && let (PreReleaseIdentifier::String(label), PreReleaseIdentifier::Integer(num)) =
                        (item, &pr[i + 1])
                    && let Some(normalized_label) = normalize_pre_release_label(label)
                    && pre_release.is_none()
                {
                    pre_release = Some(PreReleaseVar {
                        label: normalized_label,
                        number: Some(*num),
                    });
                    extra_core.push(Component::VarField("pre_release".to_string()));
                    skip_next = true;
                    continue;
                }

                // Check for keyword-only pattern
                if let PreReleaseIdentifier::String(label) = item
                    && let Some(normalized_label) = normalize_pre_release_label(label)
                    && pre_release.is_none()
                {
                    pre_release = Some(PreReleaseVar {
                        label: normalized_label,
                        number: None,
                    });
                    extra_core.push(Component::VarField("pre_release".to_string()));
                    continue;
                }

                // Regular component
                extra_core.push(match item {
                    PreReleaseIdentifier::String(s) => Component::String(s.clone()),
                    PreReleaseIdentifier::Integer(n) => Component::Integer(*n),
                });
            }

            (pre_release, extra_core, epoch, post, dev)
        } else {
            (None, Vec::new(), None, None, None)
        };

        Zerv {
            schema: ZervSchema {
                core: vec![
                    Component::VarField("major".to_string()),
                    Component::VarField("minor".to_string()),
                    Component::VarField("patch".to_string()),
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

    use crate::version::zerv::test_utils::*;

    #[rstest]
    // Basic conversions
    #[case("1.2.3", sem_zerv_1_2_3())]
    #[case("1.0.0-alpha.1", sem_zerv_1_0_0_alpha_1())]
    #[case("1.0.0-something.1", sem_zerv_1_0_0_something_1())]
    #[case("1.0.0+build.123", sem_zerv_1_0_0_build_123())]
    #[case("1.0.0-alpha.1+build.123", sem_zerv_1_0_0_alpha_1_build_123())]
    #[case(
        "1.0.0-alpha.1.lowercase.4.UPPERCASE.5+build.123",
        sem_zerv_1_0_0_alpha_1_lowercase_4_uppercase_5_build_123()
    )]
    #[case("1.0.0-foo.bar.beta.2.baz", sem_zerv_1_0_0_foo_bar_beta_2_baz())]
    #[case("1.0.0-alpha.1.beta.2", sem_zerv_1_0_0_alpha_1_beta_2())]
    #[case("1.0.0-rc.1.alpha.2.beta.3", sem_zerv_1_0_0_rc_1_alpha_2_beta_3())]
    #[case("1.0.0-pre.alpha.1", sem_zerv_1_0_0_rc_alpha_1())]
    #[case("1.0.0-test.alpha.beta.rc.1", sem_zerv_1_0_0_test_alpha_beta_rc_1())]
    // Case variations
    #[case(
        "1.0.0-ALPHA.1",
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1))
    )]
    #[case(
        "1.0.0-BETA.2",
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(2))
    )]
    #[case(
        "1.0.0-RC.3",
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, Some(3))
    )]
    #[case(
        "1.0.0-Preview.4",
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, Some(4))
    )]
    #[case(
        "1.0.0-a.1",
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1))
    )]
    #[case(
        "1.0.0-b.2",
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(2))
    )]
    #[case("1.0.0-c.3", zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, Some(3)))]
    #[case(
        "1.0.0-alpha",
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, None)
    )]
    #[case("1.0.0-beta", zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, None))]
    #[case("1.0.0-rc", zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, None))]
    #[case(
        "1.0.0-alpha.0",
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(0))
    )]
    #[case(
        "1.0.0-beta.0",
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(0))
    )]
    #[case("1.0.0-foo.1.alpha", sem_zerv_1_0_0_foo_1_alpha())]
    #[case("1.0.0-bar.2.beta", sem_zerv_1_0_0_bar_2_beta())]
    // Epoch handling
    #[case("1.0.0-epoch.1", zerv_1_0_0_with_epoch(1))]
    #[case("1.0.0-epoch.5", zerv_1_0_0_with_epoch(5))]
    #[case("1.0.0-epoch.0", zerv_1_0_0_with_epoch(0))]
    #[case("1.0.0-epoch.999", zerv_1_0_0_with_epoch(999))]
    // Post handling
    #[case("1.0.0-post.1", zerv_1_0_0_with_post(1))]
    #[case("1.0.0-post.5", zerv_1_0_0_with_post(5))]
    #[case("1.0.0-post.0", zerv_1_0_0_with_post(0))]
    // Dev handling
    #[case("1.0.0-dev.1", zerv_1_0_0_with_dev(1))]
    #[case("1.0.0-dev.0", zerv_1_0_0_with_dev(0))]
    #[case("1.0.0-dev.10", zerv_1_0_0_with_dev(10))]
    // Epoch + pre-release combinations
    #[case(
        "1.0.0-epoch.2.alpha.1",
        zerv_1_0_0_with_epoch_and_pre_release(2, PreReleaseLabel::Alpha, Some(1))
    )]
    #[case(
        "1.0.0-epoch.3.beta.2",
        zerv_1_0_0_with_epoch_and_pre_release(3, PreReleaseLabel::Beta, Some(2))
    )]
    #[case(
        "1.0.0-epoch.1.rc.5",
        zerv_1_0_0_with_epoch_and_pre_release(1, PreReleaseLabel::Rc, Some(5))
    )]
    #[case(
        "1.0.0-epoch.4.alpha",
        zerv_1_0_0_with_epoch_and_pre_release(4, PreReleaseLabel::Alpha, None)
    )]
    // Post + dev combinations
    #[case("1.0.0-post.1.dev.2", zerv_1_0_0_with_post_and_dev(1, 2))]
    #[case("1.0.0-dev.3.post.4", zerv_1_0_0_with_dev_and_post(3, 4))]
    // Pre-release + post combinations
    #[case(
        "1.0.0-alpha.1.post.2",
        zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Alpha, Some(1), 2)
    )]
    #[case(
        "1.0.0-beta.3.post.1",
        zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Beta, Some(3), 1)
    )]
    #[case(
        "1.0.0-rc.2.post.5",
        zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Rc, Some(2), 5)
    )]
    // Pre-release + dev combinations
    #[case(
        "1.0.0-alpha.1.dev.2",
        zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Alpha, Some(1), 2)
    )]
    #[case(
        "1.0.0-beta.2.dev.1",
        zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Beta, Some(2), 1)
    )]
    #[case(
        "1.0.0-rc.1.dev.3",
        zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Rc, Some(1), 3)
    )]
    // Triple combinations
    #[case(
        "1.0.0-alpha.1.post.2.dev.3",
        zerv_1_0_0_with_pre_release_post_and_dev(PreReleaseLabel::Alpha, Some(1), 2, 3)
    )]
    #[case(
        "1.0.0-beta.2.dev.1.post.3",
        zerv_1_0_0_with_pre_release_dev_and_post(PreReleaseLabel::Beta, Some(2), 1, 3)
    )]
    #[case(
        "1.0.0-rc.1.post.1.dev.1",
        zerv_1_0_0_with_pre_release_post_and_dev(PreReleaseLabel::Rc, Some(1), 1, 1)
    )]
    // Epoch + post + dev combinations
    #[case(
        "1.0.0-epoch.2.post.1.dev.3",
        zerv_1_0_0_with_epoch_post_and_dev(2, 1, 3)
    )]
    #[case(
        "1.0.0-epoch.1.dev.2.post.1",
        zerv_1_0_0_with_epoch_dev_and_post(1, 2, 1)
    )]
    // All components together
    #[case(
        "1.0.0-epoch.3.alpha.1.post.2.dev.1",
        zerv_1_0_0_with_all_components(3, PreReleaseLabel::Alpha, Some(1), 2, 1)
    )]
    #[case(
        "1.0.0-epoch.1.beta.2.dev.3.post.1",
        zerv_1_0_0_with_all_components_reordered(1, PreReleaseLabel::Beta, Some(2), 3, 1)
    )]
    // With build metadata
    #[case("1.0.0-epoch.1+build.123", zerv_1_0_0_with_epoch_and_build(1))]
    #[case("1.0.0-post.1+build.456", zerv_1_0_0_with_post_and_build(1))]
    #[case("1.0.0-dev.2+build.789", zerv_1_0_0_with_dev_and_build(2))]
    #[case(
        "1.0.0-epoch.2.alpha.1+build.abc",
        zerv_1_0_0_with_epoch_pre_release_and_build(2, PreReleaseLabel::Alpha, Some(1))
    )]
    // Mixed with other identifiers
    #[case(
        "1.0.0-foo.epoch.1.alpha.2",
        zerv_1_0_0_with_foo_epoch_and_alpha_original_order(1, 2)
    )]
    #[case("1.0.0-epoch.1.foo.post.2", zerv_1_0_0_with_epoch_foo_and_post(1, 2))]
    #[case(
        "1.0.0-bar.dev.1.epoch.2",
        zerv_1_0_0_with_bar_dev_and_epoch_original_order(1, 2)
    )]
    fn test_semver_to_zerv_conversion(#[case] semver_str: &str, #[case] expected: Zerv) {
        let semver: SemVer = semver_str.parse().unwrap();
        let zerv: Zerv = semver.into();
        assert_eq!(zerv, expected);
    }

    #[test]
    fn test_round_trip_conversion() {
        let original: SemVer = "2.1.0-beta.1+build.123".parse().unwrap();
        let zerv: Zerv = original.clone().into();
        let converted: SemVer = zerv.into();

        assert_eq!(original.to_string(), converted.to_string());
    }
}
