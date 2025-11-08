use super::PEP440;
use super::utils::LocalSegment;
use crate::utils::sanitize::Sanitizer;
use crate::version::pep440::core::{
    DevLabel,
    PostLabel,
};
use crate::version::zerv::core::Zerv;
use crate::version::zerv::{
    Component,
    Var,
};

impl PEP440 {
    fn add_flattened_to_local(&mut self, value: String) {
        for part in value.split('.') {
            if !part.is_empty() {
                let segment = if let Ok(num) = part.parse::<u32>() {
                    LocalSegment::new_uint(num)
                } else {
                    LocalSegment::try_new_str(part.to_string()).unwrap()
                };
                self.local.get_or_insert_with(Vec::new).push(segment);
            }
        }
    }

    fn process_core(
        &mut self,
        components: &[Component],
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        int_sanitizer: &Sanitizer,
        local_sanitizer: &Sanitizer,
    ) {
        for component in components {
            if let Some(value) = component.resolve_value(zerv_vars, int_sanitizer)
                && !value.is_empty()
                && let Ok(num) = value.parse::<u32>()
            {
                self.release.push(num);
                continue;
            }
            // If component doesn't resolve to a valid integer, try as local
            if let Some(local_value) = component.resolve_value(zerv_vars, local_sanitizer)
                && !local_value.is_empty()
            {
                self.add_flattened_to_local(local_value);
            }
        }
    }

    fn process_epoch(
        &mut self,
        component: &Component,
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        int_sanitizer: &Sanitizer,
    ) {
        if let Some(value) = component.resolve_value(zerv_vars, int_sanitizer)
            && !value.is_empty()
            && let Ok(epoch) = value.parse::<u32>()
        {
            self.epoch = epoch;
        }
    }

    fn process_prerelease(
        &mut self,
        var: &Var,
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        local_sanitizer: &Sanitizer,
    ) {
        let expanded = var.resolve_expanded_values(zerv_vars, local_sanitizer);
        if !expanded.is_empty() && !expanded[0].is_empty() {
            if let Ok(label) = expanded[0].parse() {
                self.pre_label = Some(label);
            }
            if expanded.len() >= 2
                && !expanded[1].is_empty()
                && let Ok(num) = expanded[1].parse::<u32>()
            {
                self.pre_number = Some(num);
            }
        }
    }

    fn process_post(
        &mut self,
        component: &Component,
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        int_sanitizer: &Sanitizer,
    ) {
        if let Some(value) = component.resolve_value(zerv_vars, int_sanitizer)
            && !value.is_empty()
            && let Ok(num) = value.parse::<u32>()
        {
            self.post_label = Some(PostLabel::Post);
            self.post_number = Some(num);
        }
    }

    fn process_dev(
        &mut self,
        component: &Component,
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        int_sanitizer: &Sanitizer,
    ) {
        if let Some(value) = component.resolve_value(zerv_vars, int_sanitizer)
            && !value.is_empty()
            && let Ok(num) = value.parse::<u32>()
        {
            self.dev_label = Some(DevLabel::Dev);
            self.dev_number = Some(num);
        }
    }

    fn add_to_local_if_valid(
        &mut self,
        component: &Component,
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        local_sanitizer: &Sanitizer,
    ) {
        if let Some(value) = component.resolve_value(zerv_vars, local_sanitizer)
            && !value.is_empty()
        {
            self.add_flattened_to_local(value);
        }
    }

    fn process_extra_core(
        &mut self,
        components: &[Component],
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        int_sanitizer: &Sanitizer,
        local_sanitizer: &Sanitizer,
    ) {
        for component in components {
            match component {
                Component::Var(var) if var.is_secondary_component() => match var {
                    Var::Epoch => self.process_epoch(component, zerv_vars, int_sanitizer),
                    Var::PreRelease => self.process_prerelease(var, zerv_vars, local_sanitizer),
                    Var::Post => self.process_post(component, zerv_vars, int_sanitizer),
                    Var::Dev => self.process_dev(component, zerv_vars, int_sanitizer),
                    _ => {}
                },
                _ => self.add_to_local_if_valid(component, zerv_vars, local_sanitizer),
            }
        }
    }

    fn process_build(
        &mut self,
        components: &[Component],
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        local_sanitizer: &Sanitizer,
    ) {
        for component in components {
            if let Some(value) = component.resolve_value(zerv_vars, local_sanitizer)
                && !value.is_empty()
            {
                self.add_flattened_to_local(value);
            }
        }
    }
}

impl From<Zerv> for PEP440 {
    fn from(zerv: Zerv) -> Self {
        let mut pep440 = PEP440::new(vec![]);
        let int_sanitizer = Sanitizer::uint();
        let local_sanitizer = Sanitizer::pep440_local_str();

        // Process core - append integers to release, overflow to local
        pep440.process_core(
            zerv.schema.core(),
            &zerv.vars,
            &int_sanitizer,
            &local_sanitizer,
        );

        // Ensure at least one release component
        if pep440.release.is_empty() {
            pep440.release.push(0);
        }

        // Process extra_core - handle secondary components, overflow to local
        pep440.process_extra_core(
            zerv.schema.extra_core(),
            &zerv.vars,
            &int_sanitizer,
            &local_sanitizer,
        );

        // Process build - all components go to local
        pep440.process_build(zerv.schema.build(), &zerv.vars, &local_sanitizer);

        pep440.normalize()
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
    // Maximum complexity test
    #[case(from::v2_3_4_max_complexity().build(), "5!2.3.4.99a1.post2+core.value.extra.value.literal.42.feature.complex.test.7.abcdef12.true.build.value.build.123")]
    // Custom field handling - build (goes to local)
    #[case(from::v1_0_0_custom_build_field("custom.field").build(), "1.0.0+custom.field")]
    #[case(from::v1_0_0_custom_build_field("simple").build(), "1.0.0+simple")]
    #[case(from::v1_0_0_custom_build_field("multi.part.value").build(), "1.0.0+multi.part.value")]
    #[case(from::v1_0_0_custom_build_field("test_value").build(), "1.0.0+test.value")]
    #[case(from::v1_0_0_custom_build_field("Feature/API-v2").build(), "1.0.0+feature.api.v2")]
    // Custom field handling - core (goes to local)
    #[case(from::v1_0_0_custom_core_field("core.field").build(), "1.0.0+core.field")]
    #[case(from::v1_0_0_custom_core_field("simple").build(), "1.0.0+simple")]
    #[case(from::v1_0_0_custom_core_field("multi.part.value").build(), "1.0.0+multi.part.value")]
    #[case(from::v1_0_0_custom_core_field("test_value").build(), "1.0.0+test.value")]
    #[case(from::v1_0_0_custom_core_field("Feature/API-v2").build(), "1.0.0+feature.api.v2")]
    // Custom field handling - extra_core (goes to local)
    #[case(from::v1_0_0_custom_extra_field("extra.field").build(), "1.0.0+extra.field")]
    #[case(from::v1_0_0_custom_extra_field("simple").build(), "1.0.0+simple")]
    #[case(from::v1_0_0_custom_extra_field("multi.part.value").build(), "1.0.0+multi.part.value")]
    #[case(from::v1_0_0_custom_extra_field("test_value").build(), "1.0.0+test.value")]
    #[case(from::v1_0_0_custom_extra_field("Feature/API-v2").build(), "1.0.0+feature.api.v2")]
    // Test case for duplicate epoch handling - second epoch should go to local
    #[case(from::v1_0_0_duplicate_epoch().build(), "1.0.0rc1+1.2.3.epoch.epoch")]
    fn test_zerv_to_pep440_conversion(#[case] zerv: Zerv, #[case] expected_pep440_str: &str) {
        let pep440: PEP440 = zerv.into();
        assert_eq!(pep440.to_string(), expected_pep440_str);
    }
}
