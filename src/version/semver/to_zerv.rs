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

        let (pre_release, extra_core) = if let Some(pr) = &semver.pre_release {
            let mut pre_release = None;
            let mut extra_core = Vec::new();
            let mut skip_next = false;

            for (i, item) in pr.iter().enumerate() {
                if skip_next {
                    skip_next = false;
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

            (pre_release, extra_core)
        } else {
            (None, Vec::new())
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
                pre_release,
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
