use super::{LocalSegment, PEP440};
use crate::version::pep440::core::{DevLabel, PostLabel};
use crate::version::zerv::{Component, Zerv, resolve_timestamp, utils::extract_core_values};

struct PEP440Components {
    epoch: u32,
    pre_label: Option<crate::version::zerv::PreReleaseLabel>,
    pre_number: Option<u32>,
    post_label: Option<PostLabel>,
    post_number: Option<u32>,
    dev_label: Option<DevLabel>,
    dev_number: Option<u32>,
    local_overflow: Vec<LocalSegment>,
}

fn extract_release_values(core_values: &[u64]) -> Vec<u32> {
    let mut release: Vec<u32> = core_values.iter().map(|&v| v as u32).collect();
    if release.is_empty() {
        release.push(0);
    }
    release
}

fn process_var_field_pep440(field: &str, zerv: &Zerv, components: &mut PEP440Components) {
    match field {
        "pre_release" => {
            if let Some(pr) = &zerv.vars.pre_release {
                components.pre_label = Some(pr.label.clone());
                components.pre_number = pr.number.map(|n| n as u32);
            }
        }
        "epoch" => {
            components.epoch = zerv.vars.epoch.unwrap_or(0) as u32;
        }
        "post" => {
            if let Some(post_num) = zerv.vars.post {
                components.post_label = Some(PostLabel::Post);
                components.post_number = Some(post_num as u32);
            }
        }
        "dev" => {
            if let Some(dev_num) = zerv.vars.dev {
                components.dev_label = Some(DevLabel::Dev);
                components.dev_number = Some(dev_num as u32);
            }
        }
        _ => {
            components
                .local_overflow
                .push(LocalSegment::String(field.to_string()));
        }
    }
}

fn add_integer_to_local(value: u64, local_overflow: &mut Vec<LocalSegment>) {
    if value <= u32::MAX as u64 {
        local_overflow.push(LocalSegment::Integer(value as u32));
    } else {
        local_overflow.push(LocalSegment::String(value.to_string()));
    }
}

fn add_var_field_to_local(field: &str, zerv: &Zerv, local_overflow: &mut Vec<LocalSegment>) {
    match field {
        crate::constants::template_vars::BUMPED_BRANCH => {
            if let Some(branch) = &zerv.vars.bumped_branch {
                local_overflow.push(LocalSegment::String(branch.clone()));
            }
        }
        crate::constants::ron_fields::DISTANCE => {
            if let Some(distance) = zerv.vars.distance {
                local_overflow.push(LocalSegment::Integer(distance as u32));
            }
        }
        crate::constants::template_vars::BUMPED_COMMIT_HASH => {
            if let Some(hash) = &zerv.vars.bumped_commit_hash {
                local_overflow.push(LocalSegment::String(hash.clone()));
            }
        }
        _ => {}
    }
}

fn add_component_to_local(
    comp: &Component,
    local_overflow: &mut Vec<LocalSegment>,
    last_timestamp: Option<u64>,
    zerv: &Zerv,
) {
    match comp {
        Component::String(s) => {
            local_overflow.push(LocalSegment::String(s.clone()));
        }
        Component::Integer(n) => {
            add_integer_to_local(*n, local_overflow);
        }
        Component::VarTimestamp(pattern) => {
            let val = resolve_timestamp(pattern, last_timestamp).unwrap_or(0);
            add_integer_to_local(val, local_overflow);
        }
        Component::VarField(field) => {
            add_var_field_to_local(field, zerv, local_overflow);
        }
    }
}

fn process_extra_core_components(zerv: &Zerv) -> PEP440Components {
    let mut components = PEP440Components {
        epoch: 0,
        pre_label: None,
        pre_number: None,
        post_label: None,
        post_number: None,
        dev_label: None,
        dev_number: None,
        local_overflow: Vec::new(),
    };

    for comp in &zerv.schema.extra_core {
        match comp {
            Component::VarField(field) => process_var_field_pep440(field, zerv, &mut components),
            _ => add_component_to_local(
                comp,
                &mut components.local_overflow,
                zerv.vars.last_timestamp,
                zerv,
            ),
        }
    }

    components
}

fn process_build_components(
    components: &[Component],
    last_timestamp: Option<u64>,
    zerv: &Zerv,
) -> Vec<LocalSegment> {
    let mut local_overflow = Vec::new();
    for comp in components {
        add_component_to_local(comp, &mut local_overflow, last_timestamp, zerv);
    }
    local_overflow
}

impl From<Zerv> for PEP440 {
    fn from(zerv: Zerv) -> Self {
        let core_values = extract_core_values(&zerv);
        let release = extract_release_values(&core_values);
        let mut components = process_extra_core_components(&zerv);

        components.local_overflow.extend(process_build_components(
            &zerv.schema.build,
            zerv.vars.last_timestamp,
            &zerv,
        ));

        let local = if components.local_overflow.is_empty() {
            None
        } else {
            Some(components.local_overflow)
        };

        PEP440 {
            epoch: components.epoch,
            release,
            pre_label: components.pre_label,
            pre_number: components.pre_number,
            post_label: components.post_label,
            post_number: components.post_number,
            dev_label: components.dev_label,
            dev_number: components.dev_number,
            local,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::test_utils::*;

    use rstest::rstest;

    #[rstest]
    // Basic conversions
    #[case(pep_zerv_1_2_3(), "1.2.3")]
    #[case(pep_zerv_1_2_3_epoch_2(), "2!1.2.3")]
    #[case(pep_zerv_1_2_3_alpha_1(), "1.2.3a1")]
    #[case(pep_zerv_1_2_3_post_1(), "1.2.3.post1")]
    #[case(pep_zerv_1_2_3_dev_1(), "1.2.3.dev1")]
    #[case(pep_zerv_1_2_3_ubuntu_build(), "1.2.3+ubuntu.20.4")]
    #[case(
        pep_zerv_complex_2_1_2_3_alpha_1_post_1_dev_1_local_1(),
        "2!1.2.3a1.post1.dev1+local.1"
    )]
    // Epoch handling
    #[case(pep_zerv_1_0_0_epoch_1(), "1!1.0.0")]
    #[case(pep_zerv_1_0_0_epoch_5(), "5!1.0.0")]
    #[case(pep_zerv_1_0_0_epoch_999(), "999!1.0.0")]
    // Post handling
    #[case(pep_zerv_1_0_0_post_5(), "1.0.0.post5")]
    #[case(pep_zerv_1_0_0_post_0(), "1.0.0.post0")]
    // Dev handling
    #[case(pep_zerv_1_0_0_dev_0(), "1.0.0.dev0")]
    #[case(pep_zerv_1_0_0_dev_10(), "1.0.0.dev10")]
    // Epoch + pre-release combinations
    #[case(pep_zerv_1_0_0_epoch_2_alpha_1(), "2!1.0.0a1")]
    #[case(pep_zerv_1_0_0_epoch_3_beta_2(), "3!1.0.0b2")]
    #[case(pep_zerv_1_0_0_epoch_1_rc_5(), "1!1.0.0rc5")]
    #[case(pep_zerv_1_0_0_epoch_4_alpha(), "4!1.0.0a0")]
    // Post + dev combinations
    #[case(pep_zerv_1_0_0_post_1_dev_2(), "1.0.0.post1.dev2")]
    // Pre-release + post combinations
    #[case(pep_zerv_1_0_0_alpha_1_post_2(), "1.0.0a1.post2")]
    #[case(pep_zerv_1_0_0_beta_3_post_1(), "1.0.0b3.post1")]
    #[case(pep_zerv_1_0_0_rc_2_post_5(), "1.0.0rc2.post5")]
    // Pre-release + dev combinations
    #[case(pep_zerv_1_0_0_alpha_1_dev_2(), "1.0.0a1.dev2")]
    #[case(pep_zerv_1_0_0_beta_2_dev_1(), "1.0.0b2.dev1")]
    #[case(pep_zerv_1_0_0_rc_1_dev_3(), "1.0.0rc1.dev3")]
    // Triple combinations
    #[case(pep_zerv_1_0_0_alpha_1_post_2_dev_3(), "1.0.0a1.post2.dev3")]
    #[case(pep_zerv_1_0_0_beta_2_post_3_dev_1(), "1.0.0b2.post3.dev1")]
    #[case(pep_zerv_1_0_0_rc_1_post_1_dev_1(), "1.0.0rc1.post1.dev1")]
    // Epoch + post + dev combinations
    #[case(pep_zerv_1_0_0_epoch_2_post_1_dev_3(), "2!1.0.0.post1.dev3")]
    #[case(pep_zerv_1_0_0_epoch_1_post_1_dev_2(), "1!1.0.0.post1.dev2")]
    // All components together
    #[case(pep_zerv_1_0_0_epoch_3_alpha_1_post_2_dev_1(), "3!1.0.0a1.post2.dev1")]
    #[case(pep_zerv_1_0_0_epoch_1_beta_2_post_1_dev_3(), "1!1.0.0b2.post1.dev3")]
    // With build metadata
    #[case(pep_zerv_1_0_0_epoch_1_build(), "1!1.0.0+build.123")]
    #[case(pep_zerv_1_0_0_post_1_build(), "1.0.0.post1+build.456")]
    #[case(pep_zerv_1_0_0_dev_2_build(), "1.0.0.dev2+build.789")]
    #[case(pep_zerv_1_0_0_epoch_2_alpha_1_build(), "2!1.0.0a1+build.abc")]
    // Complex local version identifiers
    #[case(pep_zerv_1_0_0_complex_local(), "1.0.0+foo.bar.123")]
    #[case(
        pep_zerv_1_0_0_all_components_complex_local(),
        "1!1.0.0a1.post1.dev1+complex.local.456"
    )]
    // VarField build metadata tests
    #[case(sem_zerv_1_0_0_with_branch(), "1.0.0+dev")]
    #[case(pep_zerv_1_0_0_with_distance(), "1.0.0+5")]
    #[case(pep_zerv_1_0_0_with_commit_hash(), "1.0.0+abc123")]
    #[case(pep_zerv_1_0_0_with_branch_distance_hash(), "1.0.0+dev.3.def456")]
    #[case(pep_zerv_1_0_0_with_none_varfields(), "1.0.0")]
    fn test_zerv_to_pep440_conversion(#[case] zerv: Zerv, #[case] expected_pep440_str: &str) {
        let pep440: PEP440 = zerv.into();
        assert_eq!(pep440.to_string(), expected_pep440_str);
    }
}
