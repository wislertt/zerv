use super::{BuildMetadata, PreReleaseIdentifier, SemVer};
use crate::version::semver::utils::pre_release_label_to_semver_string;
use crate::version::zerv::{Component, Zerv, resolve_timestamp, utils::extract_core_values};

fn extract_version_numbers(core_values: &[u64]) -> (u64, u64, u64) {
    let major = core_values.first().copied().unwrap_or(0);
    let minor = core_values.get(1).copied().unwrap_or(0);
    let patch = core_values.get(2).copied().unwrap_or(0);
    (major, minor, patch)
}

fn extract_overflow_identifiers(core_values: &[u64]) -> Vec<PreReleaseIdentifier> {
    if core_values.len() > 3 {
        core_values[3..]
            .iter()
            .map(|&val| PreReleaseIdentifier::Integer(val))
            .collect()
    } else {
        Vec::new()
    }
}

fn add_epoch_identifiers(identifiers: &mut Vec<PreReleaseIdentifier>, epoch: Option<u64>) {
    if let Some(epoch) = epoch {
        identifiers.push(PreReleaseIdentifier::String("epoch".to_string()));
        identifiers.push(PreReleaseIdentifier::Integer(epoch));
    }
}

fn process_var_field(identifiers: &mut Vec<PreReleaseIdentifier>, field: &str, zerv: &Zerv) {
    match field {
        "pre_release" => {
            if let Some(pr) = &zerv.vars.pre_release {
                identifiers.push(PreReleaseIdentifier::String(
                    pre_release_label_to_semver_string(&pr.label).to_string(),
                ));
                if let Some(num) = pr.number {
                    identifiers.push(PreReleaseIdentifier::Integer(num));
                }
            }
        }
        "post" => {
            if let Some(post_num) = zerv.vars.post {
                identifiers.push(PreReleaseIdentifier::String("post".to_string()));
                identifiers.push(PreReleaseIdentifier::Integer(post_num));
            }
        }
        "dev" => {
            if let Some(dev_num) = zerv.vars.dev {
                identifiers.push(PreReleaseIdentifier::String("dev".to_string()));
                identifiers.push(PreReleaseIdentifier::Integer(dev_num));
            }
        }
        _ => {}
    }
}

fn build_pre_release_identifiers(
    zerv: &Zerv,
    core_values: &[u64],
) -> Option<Vec<PreReleaseIdentifier>> {
    let mut identifiers = extract_overflow_identifiers(core_values);
    add_epoch_identifiers(&mut identifiers, zerv.vars.epoch);

    for comp in &zerv.schema.extra_core {
        match comp {
            Component::VarField(field) => process_var_field(&mut identifiers, field, zerv),
            Component::String(s) => identifiers.push(PreReleaseIdentifier::String(s.clone())),
            Component::Integer(n) => identifiers.push(PreReleaseIdentifier::Integer(*n)),
            _ => {}
        }
    }

    if identifiers.is_empty() {
        None
    } else {
        Some(identifiers)
    }
}

fn build_metadata_from_components(
    components: &[Component],
    tag_timestamp: Option<u64>,
) -> Option<Vec<BuildMetadata>> {
    if components.is_empty() {
        None
    } else {
        Some(
            components
                .iter()
                .map(|comp| match comp {
                    Component::String(s) => BuildMetadata::String(s.clone()),
                    Component::Integer(i) => BuildMetadata::Integer(*i),
                    Component::VarTimestamp(pattern) => BuildMetadata::Integer(
                        resolve_timestamp(pattern, tag_timestamp).unwrap_or(0),
                    ),
                    _ => BuildMetadata::String("unknown".to_string()),
                })
                .collect(),
        )
    }
}

impl From<Zerv> for SemVer {
    fn from(zerv: Zerv) -> Self {
        let core_values = extract_core_values(&zerv);
        let (major, minor, patch) = extract_version_numbers(&core_values);
        let pre_release = build_pre_release_identifiers(&zerv, &core_values);
        let build_metadata =
            build_metadata_from_components(&zerv.schema.build, zerv.vars.tag_timestamp);

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

    use rstest::rstest;

    use crate::version::zerv::test_utils::*;
    use crate::version::zerv::{PreReleaseLabel, ZervSchema, ZervVars};

    // CalVer helper functions (demonstrating VarTimestamp usage)
    fn calver_yy_mm_patch() -> Zerv {
        Zerv {
            schema: ZervSchema {
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
            schema: ZervSchema {
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
            schema: ZervSchema {
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
    #[case(sem_zerv_1_2_3(), "1.2.3")]
    #[case(sem_zerv_1_0_0_alpha_1(), "1.0.0-alpha.1")]
    #[case(sem_zerv_1_0_0_something_1(), "1.0.0-something.1")]
    #[case(sem_zerv_1_0_0_build_123(), "1.0.0+build.123")]
    #[case(sem_zerv_1_0_0_alpha_1_build_123(), "1.0.0-alpha.1+build.123")]
    #[case(
        sem_zerv_1_0_0_alpha_1_lowercase_4_uppercase_5_build_123(),
        "1.0.0-alpha.1.lowercase.4.UPPERCASE.5+build.123"
    )]
    #[case(sem_zerv_1_0_0_foo_bar_beta_2_baz(), "1.0.0-foo.bar.beta.2.baz")]
    #[case(sem_zerv_1_0_0_alpha_1_beta_2(), "1.0.0-alpha.1.beta.2")]
    #[case(sem_zerv_1_0_0_rc_1_alpha_2_beta_3(), "1.0.0-rc.1.alpha.2.beta.3")]
    #[case(sem_zerv_1_0_0_rc_alpha_1(), "1.0.0-rc.alpha.1")]
    #[case(sem_zerv_1_0_0_test_alpha_beta_rc_1(), "1.0.0-test.alpha.beta.rc.1")]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1)),
        "1.0.0-alpha.1"
    )]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(2)),
        "1.0.0-beta.2"
    )]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, Some(3)),
        "1.0.0-rc.3"
    )]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, Some(4)),
        "1.0.0-rc.4"
    )]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1)),
        "1.0.0-alpha.1"
    )]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(2)),
        "1.0.0-beta.2"
    )]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, Some(3)),
        "1.0.0-rc.3"
    )]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, None),
        "1.0.0-alpha"
    )]
    #[case(zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, None), "1.0.0-beta")]
    #[case(zerv_1_0_0_with_pre_release(PreReleaseLabel::Rc, None), "1.0.0-rc")]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(0)),
        "1.0.0-alpha.0"
    )]
    #[case(
        zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(0)),
        "1.0.0-beta.0"
    )]
    #[case(sem_zerv_1_0_0_foo_1_alpha(), "1.0.0-foo.1.alpha")]
    #[case(sem_zerv_1_0_0_bar_2_beta(), "1.0.0-bar.2.beta")]
    // CalVer patterns
    #[case(calver_yy_mm_patch(), "24.3.5")]
    #[case(calver_yyyy_mm_patch(), "2024.3.1")]
    #[case(calver_with_timestamp_build(), "1.0.0+2024.3.16")]
    #[case(sem_zerv_core_overflow_1_2(), "1.2.0")]
    #[case(sem_zerv_core_overflow_1_2_3_4_5(), "1.2.3-4.5")]
    // Epoch handling
    #[case(zerv_1_0_0_with_epoch(1), "1.0.0-epoch.1")]
    #[case(zerv_1_0_0_with_epoch(5), "1.0.0-epoch.5")]
    #[case(zerv_1_0_0_with_epoch(0), "1.0.0-epoch.0")]
    #[case(zerv_1_0_0_with_epoch(999), "1.0.0-epoch.999")]
    // Post handling
    #[case(zerv_1_0_0_with_post(1), "1.0.0-post.1")]
    #[case(zerv_1_0_0_with_post(5), "1.0.0-post.5")]
    #[case(zerv_1_0_0_with_post(0), "1.0.0-post.0")]
    // Dev handling
    #[case(zerv_1_0_0_with_dev(1), "1.0.0-dev.1")]
    #[case(zerv_1_0_0_with_dev(0), "1.0.0-dev.0")]
    #[case(zerv_1_0_0_with_dev(10), "1.0.0-dev.10")]
    // Epoch + pre-release combinations
    #[case(
        zerv_1_0_0_with_epoch_and_pre_release(2, PreReleaseLabel::Alpha, Some(1)),
        "1.0.0-epoch.2.alpha.1"
    )]
    #[case(
        zerv_1_0_0_with_epoch_and_pre_release(3, PreReleaseLabel::Beta, Some(2)),
        "1.0.0-epoch.3.beta.2"
    )]
    #[case(
        zerv_1_0_0_with_epoch_and_pre_release(1, PreReleaseLabel::Rc, Some(5)),
        "1.0.0-epoch.1.rc.5"
    )]
    #[case(
        zerv_1_0_0_with_epoch_and_pre_release(4, PreReleaseLabel::Alpha, None),
        "1.0.0-epoch.4.alpha"
    )]
    // Post + dev combinations
    #[case(zerv_1_0_0_with_post_and_dev(1, 2), "1.0.0-post.1.dev.2")]
    #[case(zerv_1_0_0_with_dev_and_post(3, 4), "1.0.0-dev.3.post.4")]
    // Pre-release + post combinations
    #[case(
        zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Alpha, Some(1), 2),
        "1.0.0-alpha.1.post.2"
    )]
    #[case(
        zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Beta, Some(3), 1),
        "1.0.0-beta.3.post.1"
    )]
    #[case(
        zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Rc, Some(2), 5),
        "1.0.0-rc.2.post.5"
    )]
    // Pre-release + dev combinations
    #[case(
        zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Alpha, Some(1), 2),
        "1.0.0-alpha.1.dev.2"
    )]
    #[case(
        zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Beta, Some(2), 1),
        "1.0.0-beta.2.dev.1"
    )]
    #[case(
        zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Rc, Some(1), 3),
        "1.0.0-rc.1.dev.3"
    )]
    // Triple combinations
    #[case(
        zerv_1_0_0_with_pre_release_post_and_dev(PreReleaseLabel::Alpha, Some(1), 2, 3),
        "1.0.0-alpha.1.post.2.dev.3"
    )]
    #[case(
        zerv_1_0_0_with_pre_release_dev_and_post(PreReleaseLabel::Beta, Some(2), 1, 3),
        "1.0.0-beta.2.dev.1.post.3"
    )]
    #[case(
        zerv_1_0_0_with_pre_release_post_and_dev(PreReleaseLabel::Rc, Some(1), 1, 1),
        "1.0.0-rc.1.post.1.dev.1"
    )]
    // Epoch + post + dev combinations
    #[case(
        zerv_1_0_0_with_epoch_post_and_dev(2, 1, 3),
        "1.0.0-epoch.2.post.1.dev.3"
    )]
    #[case(
        zerv_1_0_0_with_epoch_dev_and_post(1, 2, 1),
        "1.0.0-epoch.1.dev.2.post.1"
    )]
    // All components together
    #[case(
        zerv_1_0_0_with_all_components(3, PreReleaseLabel::Alpha, Some(1), 2, 1),
        "1.0.0-epoch.3.alpha.1.post.2.dev.1"
    )]
    #[case(
        zerv_1_0_0_with_all_components_reordered(1, PreReleaseLabel::Beta, Some(2), 3, 1),
        "1.0.0-epoch.1.beta.2.dev.3.post.1"
    )]
    // With build metadata
    #[case(zerv_1_0_0_with_epoch_and_build(1), "1.0.0-epoch.1+build.123")]
    #[case(zerv_1_0_0_with_post_and_build(1), "1.0.0-post.1+build.456")]
    #[case(zerv_1_0_0_with_dev_and_build(2), "1.0.0-dev.2+build.789")]
    #[case(
        zerv_1_0_0_with_epoch_pre_release_and_build(2, PreReleaseLabel::Alpha, Some(1)),
        "1.0.0-epoch.2.alpha.1+build.abc"
    )]
    // Mixed with other identifiers
    #[case(zerv_1_0_0_with_foo_epoch_and_alpha(1, 2), "1.0.0-epoch.1.foo.alpha.2")]
    #[case(zerv_1_0_0_with_epoch_foo_and_post(1, 2), "1.0.0-epoch.1.foo.post.2")]
    #[case(zerv_1_0_0_with_bar_dev_and_epoch(1, 2), "1.0.0-epoch.2.bar.dev.1")]
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
