use super::{BuildMetadata, PreReleaseIdentifier, SemVer};
use crate::version::zerv::{
    Component, PreReleaseVar, Zerv, ZervFormat, ZervVars, normalize_pre_release_label,
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
            format: ZervFormat {
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
    // Basic version
    #[case("1.2.3", {
        let mut zerv = base_zerv();
        zerv.vars.major = Some(1);
        zerv.vars.minor = Some(2);
        zerv.vars.patch = Some(3);
        zerv
    })]
    // Simple pre-release
    #[case("1.0.0-alpha.1", with_pre_release(PreReleaseLabel::Alpha, Some(1)))]
    // Non-keyword pre-release
    #[case("1.0.0-something.1", with_extra_core(vec![
        Component::String("something".to_string()),
        Component::Integer(1)
    ]))]
    // Build only
    #[case("1.0.0+build.123", with_build(vec![
        Component::String("build".to_string()),
        Component::Integer(123)
    ]))]
    // Pre-release with build
    #[case("1.0.0-alpha.1+build.123", with_pre_release_and_build(
        PreReleaseLabel::Alpha, Some(1),
        vec![Component::String("build".to_string()), Component::Integer(123)]
    ))]
    // Complex pre-release with extra and build
    #[case("1.0.0-alpha.1.lowercase.4.UPPERCASE.5+build.123", {
        let mut zerv = with_pre_release_and_build(
            PreReleaseLabel::Alpha, Some(1),
            vec![Component::String("build".to_string()), Component::Integer(123)]
        );
        zerv.format.extra_core = vec![
            Component::VarField("pre_release".to_string()),
            Component::String("lowercase".to_string()),
            Component::Integer(4),
            Component::String("UPPERCASE".to_string()),
            Component::Integer(5)
        ];
        zerv
    })]
    // Keyword in middle
    #[case("1.0.0-foo.bar.beta.2.baz", {
        let mut zerv = with_pre_release(PreReleaseLabel::Beta, Some(2));
        zerv.format.extra_core = vec![
            Component::String("foo".to_string()),
            Component::String("bar".to_string()),
            Component::VarField("pre_release".to_string()),
            Component::String("baz".to_string())
        ];
        zerv
    })]
    // Multiple keywords - first wins
    #[case("1.0.0-alpha.1.beta.2", with_pre_release_and_extra(
        PreReleaseLabel::Alpha, Some(1),
        vec![Component::String("beta".to_string()), Component::Integer(2)]
    ))]
    #[case("1.0.0-rc.1.alpha.2.beta.3", with_pre_release_and_extra(
        PreReleaseLabel::Rc, Some(1),
        vec![
            Component::String("alpha".to_string()),
            Component::Integer(2),
            Component::String("beta".to_string()),
            Component::Integer(3)
        ]
    ))]
    // Keyword without number
    #[case("1.0.0-pre.alpha.1", with_pre_release_and_extra(
        PreReleaseLabel::Rc, None,
        vec![Component::String("alpha".to_string()), Component::Integer(1)]
    ))]
    #[case("1.0.0-test.alpha.beta.rc.1", {
        let mut zerv = with_pre_release(PreReleaseLabel::Alpha, None);
        zerv.format.extra_core = vec![
            Component::String("test".to_string()),
            Component::VarField("pre_release".to_string()),
            Component::String("beta".to_string()),
            Component::String("rc".to_string()),
            Component::Integer(1)
        ];
        zerv
    })]
    // Uppercase keywords
    #[case("1.0.0-ALPHA.1", with_pre_release(PreReleaseLabel::Alpha, Some(1)))]
    #[case("1.0.0-BETA.2", with_pre_release(PreReleaseLabel::Beta, Some(2)))]
    #[case("1.0.0-RC.3", with_pre_release(PreReleaseLabel::Rc, Some(3)))]
    #[case("1.0.0-Preview.4", with_pre_release(PreReleaseLabel::Rc, Some(4)))]
    // Single-letter aliases
    #[case("1.0.0-a.1", with_pre_release(PreReleaseLabel::Alpha, Some(1)))]
    #[case("1.0.0-b.2", with_pre_release(PreReleaseLabel::Beta, Some(2)))]
    #[case("1.0.0-c.3", with_pre_release(PreReleaseLabel::Rc, Some(3)))]
    // Keywords without numbers
    #[case("1.0.0-alpha", with_pre_release(PreReleaseLabel::Alpha, None))]
    #[case("1.0.0-beta", with_pre_release(PreReleaseLabel::Beta, None))]
    #[case("1.0.0-rc", with_pre_release(PreReleaseLabel::Rc, None))]
    // Zero numbers
    #[case("1.0.0-alpha.0", with_pre_release(PreReleaseLabel::Alpha, Some(0)))]
    #[case("1.0.0-beta.0", with_pre_release(PreReleaseLabel::Beta, Some(0)))]
    // Keywords at end
    #[case("1.0.0-foo.1.alpha", {
        let mut zerv = with_pre_release(PreReleaseLabel::Alpha, None);
        zerv.format.extra_core = vec![
            Component::String("foo".to_string()),
            Component::Integer(1),
            Component::VarField("pre_release".to_string())
        ];
        zerv
    })]
    #[case("1.0.0-bar.2.beta", {
        let mut zerv = with_pre_release(PreReleaseLabel::Beta, None);
        zerv.format.extra_core = vec![
            Component::String("bar".to_string()),
            Component::Integer(2),
            Component::VarField("pre_release".to_string())
        ];
        zerv
    })]
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
