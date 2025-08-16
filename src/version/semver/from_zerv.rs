use super::{BuildMetadata, PreReleaseIdentifier, SemVer};
use crate::version::semver::utils::pre_release_label_to_semver_string;
use crate::version::zerv::{Component, Zerv, resolve_timestamp};

impl From<Zerv> for SemVer {
    fn from(zerv: Zerv) -> Self {
        // Extract values from core components
        let mut core_values = Vec::new();
        for comp in &zerv.format.core {
            let val = match comp {
                Component::VarField(field) => match field.as_str() {
                    "major" => zerv.vars.major.unwrap_or(0),
                    "minor" => zerv.vars.minor.unwrap_or(0),
                    "patch" => zerv.vars.patch.unwrap_or(0),
                    _ => 0,
                },
                Component::VarTimestamp(pattern) => {
                    resolve_timestamp(pattern, zerv.vars.tag_timestamp).unwrap_or(0)
                }
                Component::Integer(n) => *n,
                _ => 0,
            };
            core_values.push(val);
        }

        // First 3 indices become major.minor.patch, pad with 0 if less than 3
        let major = core_values.first().copied().unwrap_or(0);
        let minor = core_values.get(1).copied().unwrap_or(0);
        let patch = core_values.get(2).copied().unwrap_or(0);

        // If core has more than 3 components, overflow goes to pre-release (at front)
        let overflow_identifiers = if core_values.len() > 3 {
            core_values[3..]
                .iter()
                .map(|&val| PreReleaseIdentifier::Integer(val))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // Build pre-release: overflow from core (at front) + extra_core components
        let mut identifiers = overflow_identifiers;

        for comp in &zerv.format.extra_core {
            match comp {
                Component::VarField(field) if field == "pre_release" => {
                    if let Some(pr) = &zerv.vars.pre_release {
                        identifiers.push(PreReleaseIdentifier::String(
                            pre_release_label_to_semver_string(&pr.label).to_string(),
                        ));
                        if let Some(num) = pr.number {
                            identifiers.push(PreReleaseIdentifier::Integer(num));
                        }
                    }
                }
                Component::String(s) => {
                    identifiers.push(PreReleaseIdentifier::String(s.clone()));
                }
                Component::Integer(n) => {
                    identifiers.push(PreReleaseIdentifier::Integer(*n));
                }
                _ => {}
            }
        }

        let pre_release = if identifiers.is_empty() {
            None
        } else {
            Some(identifiers)
        };

        let build_metadata = if zerv.format.build.is_empty() {
            None
        } else {
            Some(
                zerv.format
                    .build
                    .iter()
                    .map(|comp| match comp {
                        Component::String(s) => BuildMetadata::String(s.clone()),
                        Component::Integer(i) => BuildMetadata::Integer(*i),
                        Component::VarTimestamp(pattern) => BuildMetadata::Integer(
                            resolve_timestamp(pattern, zerv.vars.tag_timestamp).unwrap_or(0),
                        ),
                        _ => BuildMetadata::String("unknown".to_string()),
                    })
                    .collect(),
            )
        };

        SemVer {
            major,
            minor,
            patch,
            pre_release,
            build_metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::PreReleaseLabel;
    use crate::version::zerv::{Component, ZervFormat, ZervVars};
    use rstest::rstest;

    use crate::version::zerv::test_utils::*;

    // CalVer helper functions (demonstrating VarTimestamp usage)
    fn calver_yy_mm_patch() -> Zerv {
        Zerv {
            format: ZervFormat {
                core: vec![
                    Component::VarTimestamp("YY".to_string()),
                    Component::VarTimestamp("MM".to_string()),
                    Component::VarField("patch".to_string()),
                ],
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars {
                patch: Some(5),
                tag_timestamp: Some(1710547200), // 2024-03-15
                ..Default::default()
            },
        }
    }

    fn calver_yyyy_mm_patch() -> Zerv {
        Zerv {
            format: ZervFormat {
                core: vec![
                    Component::VarTimestamp("YYYY".to_string()),
                    Component::VarTimestamp("MM".to_string()),
                    Component::VarField("patch".to_string()),
                ],
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars {
                patch: Some(1),
                tag_timestamp: Some(1710547200),
                ..Default::default()
            },
        }
    }

    fn calver_with_timestamp_build() -> Zerv {
        Zerv {
            format: ZervFormat {
                core: vec![
                    Component::VarField("major".to_string()),
                    Component::VarField("minor".to_string()),
                    Component::VarField("patch".to_string()),
                ],
                extra_core: vec![],
                build: vec![
                    Component::VarTimestamp("YYYY".to_string()),
                    Component::VarTimestamp("MM".to_string()),
                    Component::VarTimestamp("DD".to_string()),
                ],
            },
            vars: ZervVars {
                major: Some(1),
                minor: Some(0),
                patch: Some(0),
                tag_timestamp: Some(1710547200),
                ..Default::default()
            },
        }
    }

    #[rstest]
    // Basic version
    #[case({
        let mut zerv = base_zerv();
        zerv.vars.major = Some(1);
        zerv.vars.minor = Some(2);
        zerv.vars.patch = Some(3);
        zerv
    }, "1.2.3")]
    // Simple pre-release
    #[case(with_pre_release(PreReleaseLabel::Alpha, Some(1)), "1.0.0-alpha.1")]
    // Non-keyword pre-release
    #[case(with_extra_core(vec![
        Component::String("something".to_string()),
        Component::Integer(1)
    ]), "1.0.0-something.1")]
    // Build only
    #[case(with_build(vec![
        Component::String("build".to_string()),
        Component::Integer(123)
    ]), "1.0.0+build.123")]
    // Pre-release with build
    #[case(with_pre_release_and_build(
        PreReleaseLabel::Alpha, Some(1),
        vec![Component::String("build".to_string()), Component::Integer(123)]
    ), "1.0.0-alpha.1+build.123")]
    // Complex pre-release with extra and build
    #[case({
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
    }, "1.0.0-alpha.1.lowercase.4.UPPERCASE.5+build.123")]
    // Keyword in middle
    #[case({
        let mut zerv = with_pre_release(PreReleaseLabel::Beta, Some(2));
        zerv.format.extra_core = vec![
            Component::String("foo".to_string()),
            Component::String("bar".to_string()),
            Component::VarField("pre_release".to_string()),
            Component::String("baz".to_string())
        ];
        zerv
    }, "1.0.0-foo.bar.beta.2.baz")]
    // Multiple keywords - first wins
    #[case(with_pre_release_and_extra(
        PreReleaseLabel::Alpha, Some(1),
        vec![Component::String("beta".to_string()), Component::Integer(2)]
    ), "1.0.0-alpha.1.beta.2")]
    #[case(with_pre_release_and_extra(
        PreReleaseLabel::Rc, Some(1),
        vec![
            Component::String("alpha".to_string()),
            Component::Integer(2),
            Component::String("beta".to_string()),
            Component::Integer(3)
        ]
    ), "1.0.0-rc.1.alpha.2.beta.3")]
    // Keyword without number
    #[case(with_pre_release_and_extra(
        PreReleaseLabel::Rc, None,
        vec![Component::String("alpha".to_string()), Component::Integer(1)]
    ), "1.0.0-rc.alpha.1")]
    #[case({
        let mut zerv = with_pre_release(PreReleaseLabel::Alpha, None);
        zerv.format.extra_core = vec![
            Component::String("test".to_string()),
            Component::VarField("pre_release".to_string()),
            Component::String("beta".to_string()),
            Component::String("rc".to_string()),
            Component::Integer(1)
        ];
        zerv
    }, "1.0.0-test.alpha.beta.rc.1")]
    // Uppercase keywords
    #[case(with_pre_release(PreReleaseLabel::Alpha, Some(1)), "1.0.0-alpha.1")]
    #[case(with_pre_release(PreReleaseLabel::Beta, Some(2)), "1.0.0-beta.2")]
    #[case(with_pre_release(PreReleaseLabel::Rc, Some(3)), "1.0.0-rc.3")]
    #[case(with_pre_release(PreReleaseLabel::Rc, Some(4)), "1.0.0-rc.4")]
    // Single-letter aliases
    #[case(with_pre_release(PreReleaseLabel::Alpha, Some(1)), "1.0.0-alpha.1")]
    #[case(with_pre_release(PreReleaseLabel::Beta, Some(2)), "1.0.0-beta.2")]
    #[case(with_pre_release(PreReleaseLabel::Rc, Some(3)), "1.0.0-rc.3")]
    // Keywords without numbers
    #[case(with_pre_release(PreReleaseLabel::Alpha, None), "1.0.0-alpha")]
    #[case(with_pre_release(PreReleaseLabel::Beta, None), "1.0.0-beta")]
    #[case(with_pre_release(PreReleaseLabel::Rc, None), "1.0.0-rc")]
    // Zero numbers
    #[case(with_pre_release(PreReleaseLabel::Alpha, Some(0)), "1.0.0-alpha.0")]
    #[case(with_pre_release(PreReleaseLabel::Beta, Some(0)), "1.0.0-beta.0")]
    // Keywords at end
    #[case({
        let mut zerv = with_pre_release(PreReleaseLabel::Alpha, None);
        zerv.format.extra_core = vec![
            Component::String("foo".to_string()),
            Component::Integer(1),
            Component::VarField("pre_release".to_string())
        ];
        zerv
    }, "1.0.0-foo.1.alpha")]
    #[case({
        let mut zerv = with_pre_release(PreReleaseLabel::Beta, None);
        zerv.format.extra_core = vec![
            Component::String("bar".to_string()),
            Component::Integer(2),
            Component::VarField("pre_release".to_string())
        ];
        zerv
    }, "1.0.0-bar.2.beta")]
    // CalVer patterns (VarTimestamp examples)
    #[case(calver_yy_mm_patch(), "24.3.5")]
    #[case(calver_yyyy_mm_patch(), "2024.3.1")]
    #[case(calver_with_timestamp_build(), "1.0.0+2024.3.16")]
    // Core overflow tests
    #[case({
        Zerv {
            format: ZervFormat {
                core: vec![Component::Integer(1), Component::Integer(2)], // Only 2 components
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars::default(),
        }
    }, "1.2.0")] // Pad with 0
    #[case({
        Zerv {
            format: ZervFormat {
                core: vec![
                    Component::Integer(1),
                    Component::Integer(2),
                    Component::Integer(3),
                    Component::Integer(4),
                    Component::Integer(5)
                ], // 5 components - overflow 4,5 to pre-release
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars::default(),
        }
    }, "1.2.3-4.5")] // Overflow to pre-release
    fn test_zerv_to_semver_conversion(#[case] zerv: Zerv, #[case] expected_semver_str: &str) {
        let semver: SemVer = zerv.into();
        assert_eq!(semver.to_string(), expected_semver_str);
    }

    #[test]
    fn test_round_trip_conversion() {
        let original: SemVer = "2.1.0-beta.1".parse().unwrap();
        let zerv: Zerv = original.clone().into();
        let converted: SemVer = zerv.into();

        assert_eq!(original.to_string(), converted.to_string());
    }
}
