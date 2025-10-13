use super::{
    BuildMetadata,
    PreReleaseIdentifier,
    SemVer,
};
use crate::version::semver::utils::pre_release_label_to_semver_string;
use crate::version::zerv::utils::extract_core_values;
use crate::version::zerv::{
    Component,
    Var,
    Zerv,
    resolve_timestamp,
};

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
            .map(|&val| PreReleaseIdentifier::UInt(val))
            .collect()
    } else {
        Vec::new()
    }
}

fn add_epoch_identifiers(identifiers: &mut Vec<PreReleaseIdentifier>, epoch: Option<u64>) {
    if let Some(epoch) = epoch {
        identifiers.push(PreReleaseIdentifier::String("epoch".to_string()));
        identifiers.push(PreReleaseIdentifier::UInt(epoch));
    }
}

fn process_var_field(identifiers: &mut Vec<PreReleaseIdentifier>, var: &Var, zerv: &Zerv) {
    match var {
        Var::PreRelease => {
            if let Some(pr) = &zerv.vars.pre_release {
                identifiers.push(PreReleaseIdentifier::String(
                    pre_release_label_to_semver_string(&pr.label).to_string(),
                ));
                if let Some(num) = pr.number {
                    identifiers.push(PreReleaseIdentifier::UInt(num));
                }
            }
        }
        Var::Post => {
            if let Some(post_num) = zerv.vars.post {
                identifiers.push(PreReleaseIdentifier::String("post".to_string()));
                identifiers.push(PreReleaseIdentifier::UInt(post_num));
            }
        }
        Var::Dev => {
            if let Some(dev_num) = zerv.vars.dev {
                identifiers.push(PreReleaseIdentifier::String("dev".to_string()));
                identifiers.push(PreReleaseIdentifier::UInt(dev_num));
            }
        }
        Var::Custom(name) => {
            identifiers.push(PreReleaseIdentifier::String(name.clone()));
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
            Component::Var(var) => {
                process_var_field(&mut identifiers, var, zerv);
            }
            Component::Str(s) => identifiers.push(PreReleaseIdentifier::String(s.clone())),
            Component::Int(n) => identifiers.push(PreReleaseIdentifier::UInt(*n)),
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
    last_timestamp: Option<u64>,
    zerv: &Zerv,
) -> Option<Vec<BuildMetadata>> {
    if components.is_empty() {
        None
    } else {
        let metadata: Vec<BuildMetadata> = components
            .iter()
            .filter_map(|comp| match comp {
                Component::Str(s) => Some(BuildMetadata::String(s.clone())),
                Component::Int(i) => Some(BuildMetadata::UInt(*i)),
                Component::Var(var) => match var {
                    Var::Timestamp(pattern) => last_timestamp
                        .and_then(|ts| resolve_timestamp(pattern, ts).ok())
                        .and_then(|result| result.parse::<u64>().ok())
                        .map(BuildMetadata::UInt),
                    Var::BumpedBranch => zerv
                        .vars
                        .bumped_branch
                        .as_ref()
                        .map(|s| BuildMetadata::String(s.clone())),
                    Var::Distance => zerv.vars.distance.map(BuildMetadata::UInt),
                    Var::BumpedCommitHashShort => zerv
                        .vars
                        .get_bumped_commit_hash_short()
                        .map(BuildMetadata::String),
                    _ => None,
                },
            })
            .collect();

        if metadata.is_empty() {
            None
        } else {
            Some(metadata)
        }
    }
}

impl From<Zerv> for SemVer {
    fn from(zerv: Zerv) -> Self {
        if let Err(e) = zerv.schema.validate() {
            panic!("Invalid schema in SemVer::from(Zerv): {e}");
        }
        let core_values = extract_core_values(&zerv);
        let (major, minor, patch) = extract_version_numbers(&core_values);
        let pre_release = build_pre_release_identifiers(&zerv, &core_values);
        let build_metadata =
            build_metadata_from_components(&zerv.schema.build, zerv.vars.last_timestamp, &zerv);

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
    use rstest::rstest;

    use super::*;
    use crate::test_utils::zerv::zerv_calver;
    use crate::test_utils::zerv::zerv_semver::from;
    use crate::version::zerv::core::PreReleaseLabel;

    #[rstest]
    #[case(from::v1_2_3().build(), "1.2.3")]
    #[case(from::v1_0_0_a1().build(), "1.0.0-alpha.1")]
    #[case(from::v1_0_0_extra_something().build(), "1.0.0-something.1")]
    #[case(from::v1_0_0_build().build(), "1.0.0+build.123")]
    #[case(from::v1_0_0_a1_build().build(), "1.0.0-alpha.1+build.123")]
    #[case(from::v1_0_0_a1_extra_complex().build(), "1.0.0-alpha.1.lowercase.4.UPPERCASE.5+build.123")]
    #[case(from::v1_0_0_foo_bar_b2_baz().build(), "1.0.0-foo.bar.beta.2.baz")]
    #[case(from::v1_0_0_a1_b2().build(), "1.0.0-alpha.1.beta.2")]
    #[case(from::v1_0_0_rc1_a2_b3().build(), "1.0.0-rc.1.alpha.2.beta.3")]
    #[case(from::v1_0_0_rc_none_a1().build(), "1.0.0-rc.alpha.1")]
    #[case(from::v1_0_0_test_alpha_beta_rc1().build(), "1.0.0-test.alpha.beta.rc.1")]
    #[case(from::v1_0_0_a1().build(), "1.0.0-alpha.1")]
    #[case(from::v1_0_0_b2().build(), "1.0.0-beta.2")]
    #[case(from::v1_0_0_rc3().build(), "1.0.0-rc.3")]
    #[case(from::v1_0_0().with_pre_release(PreReleaseLabel::Rc, Some(4)).build(), "1.0.0-rc.4")]
    #[case(from::v1_0_0_a1().build(), "1.0.0-alpha.1")]
    #[case(from::v1_0_0_b2().build(), "1.0.0-beta.2")]
    #[case(from::v1_0_0_rc3().build(), "1.0.0-rc.3")]
    #[case(from::v1_0_0_a_none().build(), "1.0.0-alpha")]
    #[case(from::v1_0_0_b_none().build(), "1.0.0-beta")]
    #[case(from::v1_0_0_rc_none().build(), "1.0.0-rc")]
    #[case(from::v1_0_0_a0().build(), "1.0.0-alpha.0")]
    #[case(from::v1_0_0_b0().build(), "1.0.0-beta.0")]
    #[case(from::v1_0_0_foo_alpha().build(), "1.0.0-foo.1.alpha")]
    #[case(from::v1_0_0_bar_beta().build(), "1.0.0-bar.2.beta")]
    #[case(from::v1_2_0().build(), "1.2.0")]
    #[case(from::v1_2_3_4_5().build(), "1.2.3-4.5")]
    // Epoch handling
    #[case(from::v1_0_0_e1().build(), "1.0.0-epoch.1")]
    #[case(from::v1_0_0_e5().build(), "1.0.0-epoch.5")]
    #[case(from::v1_0_0_e0().build(), "1.0.0-epoch.0")]
    #[case(from::v1_0_0_e999().build(), "1.0.0-epoch.999")]
    // Post handling
    #[case(from::v1_0_0_post1().build(), "1.0.0-post.1")]
    #[case(from::v1_0_0_post5().build(), "1.0.0-post.5")]
    #[case(from::v1_0_0_post0().build(), "1.0.0-post.0")]
    // Dev handling
    #[case(from::v1_0_0_dev1().build(), "1.0.0-dev.1")]
    #[case(from::v1_0_0_dev0().build(), "1.0.0-dev.0")]
    #[case(from::v1_0_0_dev10().build(), "1.0.0-dev.10")]
    // Epoch + pre-release combinations
    #[case(from::v1_0_0_e2_a1().build(), "1.0.0-epoch.2.alpha.1")]
    #[case(from::v1_0_0_e3_b2().build(), "1.0.0-epoch.3.beta.2")]
    #[case(from::v1_0_0_e1_rc5().build(), "1.0.0-epoch.1.rc.5")]
    #[case(from::v1_0_0_e4_a_none().build(), "1.0.0-epoch.4.alpha")]
    // Post + dev combinations
    #[case(from::v1_0_0_post1_dev2().build(), "1.0.0-post.1.dev.2")]
    #[case(from::v1_0_0_dev3_post4().build(), "1.0.0-dev.3.post.4")]
    // Pre-release + post combinations
    #[case(from::v1_0_0_a1_post2().build(), "1.0.0-alpha.1.post.2")]
    #[case(from::v1_0_0_b3_post1().build(), "1.0.0-beta.3.post.1")]
    #[case(from::v1_0_0_rc2_post5().build(), "1.0.0-rc.2.post.5")]
    // Pre-release + dev combinations
    #[case(from::v1_0_0_a1_dev2().build(), "1.0.0-alpha.1.dev.2")]
    #[case(from::v1_0_0_b2_dev1().build(), "1.0.0-beta.2.dev.1")]
    #[case(from::v1_0_0_rc1_dev3().build(), "1.0.0-rc.1.dev.3")]
    // Triple combinations
    #[case(from::v1_0_0_a1_post2_dev3().build(), "1.0.0-alpha.1.post.2.dev.3")]
    #[case(from::v1_0_0_b2_dev1_post3().build(), "1.0.0-beta.2.dev.1.post.3")]
    #[case(from::v1_0_0_rc1_post1_dev1().build(), "1.0.0-rc.1.post.1.dev.1")]
    // Custom field handling
    #[case(from::v1_0_0_custom_field().build(), "1.0.0-custom_field")]
    // Epoch + post + dev combinations
    #[case(from::v1_0_0_e2_post1_dev3().build(), "1.0.0-epoch.2.post.1.dev.3")]
    #[case(from::v1_0_0_e1_dev2_post1().build(), "1.0.0-epoch.1.dev.2.post.1")]
    // All components together
    #[case(from::v1_0_0_e3_a1_post2_dev1().build(), "1.0.0-epoch.3.alpha.1.post.2.dev.1")]
    #[case(from::v1_0_0_e1_b2_dev3_post1().build(), "1.0.0-epoch.1.beta.2.dev.3.post.1")]
    // With build metadata
    #[case(from::v1_0_0_e1_build().build(), "1.0.0-epoch.1+build.123")]
    #[case(from::v1_0_0_post1_build().build(), "1.0.0-post.1+build.456")]
    #[case(from::v1_0_0_dev2_build().build(), "1.0.0-dev.2+build.789")]
    #[case(from::v1_0_0_e2_a1_build().build(), "1.0.0-epoch.2.alpha.1+build.abc")]
    // Mixed with other identifiers
    #[case(from::v1_0_0_e1_foo_a2().build(), "1.0.0-epoch.1.foo.alpha.2")]
    #[case(from::v1_0_0_e1_foo_post2().build(), "1.0.0-epoch.1.foo.post.2")]
    #[case(from::v1_0_0_e2_bar_dev1().build(), "1.0.0-epoch.2.bar.dev.1")]
    // VarField build metadata tests
    #[case(from::v1_0_0_branch_dev().build(), "1.0.0+dev")]
    #[case(from::v1_0_0_distance_5().build(), "1.0.0+5")]
    #[case(from::v1_0_0_commit_abc123().build(), "1.0.0+abc123")]
    #[case(from::v1_0_0_branch_distance_commit().build(), "1.0.0+dev.3.def456")]
    #[case(from::v1_0_0().build(), "1.0.0")]
    // CalVer patterns
    #[case(zerv_calver::calver_yy_mm_patch(), "24.3.5")]
    #[case(zerv_calver::calver_yyyy_mm_patch(), "2024.3.1")]
    #[case(zerv_calver::calver_with_timestamp_build(), "1.0.0+2024.3.16")]
    fn test_zerv_to_semver_conversion(#[case] zerv: Zerv, #[case] expected_semver_str: &str) {
        let semver: SemVer = zerv.into();
        assert_eq!(semver.to_string(), expected_semver_str);
    }
}
