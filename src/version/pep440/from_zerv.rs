use super::PEP440;
use super::utils::LocalSegment;
use crate::version::pep440::core::{
    DevLabel,
    PostLabel,
};
use crate::version::zerv::core::Zerv;
use crate::version::zerv::utils::extract_core_values;
use crate::version::zerv::{
    Component,
    PreReleaseLabel,
    Var,
    resolve_timestamp,
};

struct PEP440Components {
    epoch: u32,
    pre_label: Option<PreReleaseLabel>,
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

fn process_var_field_pep440(var: &Var, zerv: &Zerv, components: &mut PEP440Components) {
    match var {
        Var::PreRelease => {
            if let Some(pr) = &zerv.vars.pre_release {
                components.pre_label = Some(pr.label.clone());
                components.pre_number = pr.number.map(|n| n as u32);
            }
        }
        Var::Epoch => {
            components.epoch = zerv.vars.epoch.unwrap_or(0) as u32;
        }
        Var::Post => {
            if let Some(post_num) = zerv.vars.post {
                components.post_label = Some(PostLabel::Post);
                components.post_number = Some(post_num as u32);
            }
        }
        Var::Dev => {
            if let Some(dev_num) = zerv.vars.dev {
                components.dev_label = Some(DevLabel::Dev);
                components.dev_number = Some(dev_num as u32);
            }
        }
        Var::Custom(name) => {
            components
                .local_overflow
                .push(LocalSegment::Str(name.clone()));
        }
        _ => {
            add_var_field_to_local(var, zerv, &mut components.local_overflow);
        }
    }
}

fn add_integer_to_local(value: u64, local_overflow: &mut Vec<LocalSegment>) {
    if value <= u32::MAX as u64 {
        local_overflow.push(LocalSegment::UInt(value as u32));
    } else {
        local_overflow.push(LocalSegment::Str(value.to_string()));
    }
}

fn add_var_field_to_local(var: &Var, zerv: &Zerv, local_overflow: &mut Vec<LocalSegment>) {
    match var {
        Var::BumpedBranch => {
            if let Some(branch) = &zerv.vars.bumped_branch {
                local_overflow.push(LocalSegment::Str(branch.clone()));
            }
        }
        Var::Distance => {
            if let Some(distance) = zerv.vars.distance {
                local_overflow.push(LocalSegment::UInt(distance as u32));
            }
        }
        Var::BumpedCommitHashShort => {
            if let Some(hash) = zerv.vars.get_bumped_commit_hash_short() {
                local_overflow.push(LocalSegment::Str(hash));
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
        Component::Str(s) => {
            local_overflow.push(LocalSegment::Str(s.clone()));
        }
        Component::Int(n) => {
            add_integer_to_local(*n, local_overflow);
        }
        Component::Var(Var::Timestamp(pattern)) => {
            if let Some(ts) = last_timestamp
                && let Ok(result) = resolve_timestamp(pattern, ts)
                && let Ok(val) = result.parse::<u64>()
            {
                add_integer_to_local(val, local_overflow);
            }
        }
        Component::Var(var) => {
            add_var_field_to_local(var, zerv, local_overflow);
        }
    }
}

fn process_extra_core_components(zerv: &Zerv) -> PEP440Components {
    let mut components = PEP440Components {
        epoch: zerv.vars.epoch.unwrap_or(0) as u32,
        pre_label: None,
        pre_number: None,
        post_label: None,
        post_number: None,
        dev_label: None,
        dev_number: None,
        local_overflow: Vec::new(),
    };

    // Process pre_release from vars
    if let Some(pr) = &zerv.vars.pre_release {
        components.pre_label = Some(pr.label.clone());
        components.pre_number = pr.number.map(|n| n as u32);
    }

    // Process post from vars
    if let Some(post_num) = zerv.vars.post {
        components.post_label = Some(PostLabel::Post);
        components.post_number = Some(post_num as u32);
    }

    // Process dev from vars
    if let Some(dev_num) = zerv.vars.dev {
        components.dev_label = Some(DevLabel::Dev);
        components.dev_number = Some(dev_num as u32);
    }

    for comp in &zerv.schema.extra_core {
        match comp {
            Component::Var(var) => {
                process_var_field_pep440(var, zerv, &mut components);
            }
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
        if let Err(e) = zerv.schema.validate() {
            panic!("Invalid schema in PEP440::from(Zerv): {e}");
        }
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
        .normalize()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::zerv::zerv_calver;
    use crate::test_utils::zerv::zerv_pep440::from;
    use crate::version::zerv::PreReleaseLabel;

    #[rstest]
    // Basic conversions
    #[case(from::v1_2_3().build(), "1.2.3")]
    #[case(from::v1_2_3_e2().build(), "2!1.2.3")]
    #[case(from::v1_2_3_a1().build(), "1.2.3a1")]
    #[case(from::v1_2_3_post1().build(), "1.2.3.post1")]
    #[case(from::v1_2_3_dev1().build(), "1.2.3.dev1")]
    #[case(from::v1_2_3_ubuntu_build().build(), "1.2.3+ubuntu.20.4")]
    #[case(from::v1_2_3_e2_a1_post1_dev1_local().build(), "2!1.2.3a1.post1.dev1+local.1")]
    // Epoch handling
    #[case(from::v1_0_0_e1().build(), "1!1.0.0")]
    #[case(from::v1_0_0().with_epoch(5).build(), "5!1.0.0")]
    #[case(from::v1_0_0().with_epoch(999).build(), "999!1.0.0")]
    // Post handling
    #[case(from::v1_0_0_post5().build(), "1.0.0.post5")]
    #[case(from::v1_0_0().with_post(0).build(), "1.0.0.post0")]
    // Dev handling
    #[case(from::v1_0_0_dev0().build(), "1.0.0.dev0")]
    #[case(from::v1_0_0_dev10().build(), "1.0.0.dev10")]
    // Epoch + pre-release combinations
    #[case(from::v1_0_0_e2_a1().build(), "2!1.0.0a1")]
    #[case(from::v1_0_0_e3_b2().build(), "3!1.0.0b2")]
    #[case(from::v1_0_0_e1_rc5().build(), "1!1.0.0rc5")]
    #[case(from::v1_0_0().with_epoch(4).with_pre_release(PreReleaseLabel::Alpha, None).build(), "4!1.0.0a0")]
    // Post + dev combinations
    #[case(from::v1_0_0_post1_dev2().build(), "1.0.0.post1.dev2")]
    // Pre-release + post combinations
    #[case(from::v1_0_0_a1_post2().build(), "1.0.0a1.post2")]
    #[case(from::v1_0_0_b3_post1().build(), "1.0.0b3.post1")]
    #[case(from::v1_0_0_rc2_post5().build(), "1.0.0rc2.post5")]
    // Pre-release + dev combinations
    #[case(from::v1_0_0_a1_dev2().build(), "1.0.0a1.dev2")]
    #[case(from::v1_0_0_b2_dev1().build(), "1.0.0b2.dev1")]
    #[case(from::v1_0_0_rc1_dev3().build(), "1.0.0rc1.dev3")]
    // Triple combinations
    #[case(from::v1_0_0_a1_post2_dev3().build(), "1.0.0a1.post2.dev3")]
    #[case(from::v1_0_0_b2_post3_dev1().build(), "1.0.0b2.post3.dev1")]
    #[case(from::v1_0_0_rc1_post1_dev1().build(), "1.0.0rc1.post1.dev1")]
    // Epoch + post + dev combinations
    #[case(from::v1_0_0_e2_post1_dev3().build(), "2!1.0.0.post1.dev3")]
    #[case(from::v1_0_0_e1_post1_dev2().build(), "1!1.0.0.post1.dev2")]
    // All components together
    #[case(from::v1_0_0_e3_a1_post2_dev1().build(), "3!1.0.0a1.post2.dev1")]
    #[case(from::v1_0_0_e1_b2_post1_dev3().build(), "1!1.0.0b2.post1.dev3")]
    // With build metadata
    #[case(from::v1_0_0_e1_build().build(), "1!1.0.0+build.123")]
    #[case(from::v1_0_0_post1_build().build(), "1.0.0.post1+build.456")]
    #[case(from::v1_0_0_dev2_build().build(), "1.0.0.dev2+build.789")]
    #[case(from::v1_0_0_e2_a1_build().build(), "2!1.0.0a1+build.abc")]
    // Complex local version identifiers
    #[case(from::v1_0_0_complex_build().build(), "1.0.0+foo.bar.123")]
    #[case(from::v1_0_0_e1_a1_post1_dev1_complex().build(), "1!1.0.0a1.post1.dev1+complex.local.456")]
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
    // Custom field handling
    #[case(from::v1_0_0_custom_field().build(), "1.0.0+custom_field")]
    fn test_zerv_to_pep440_conversion(#[case] zerv: Zerv, #[case] expected_pep440_str: &str) {
        let pep440: PEP440 = zerv.into();
        assert_eq!(pep440.to_string(), expected_pep440_str);
    }
}
