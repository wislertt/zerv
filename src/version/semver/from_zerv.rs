use super::{
    BuildMetadata,
    PreReleaseIdentifier,
    SemVer,
};
use crate::utils::sanitize::Sanitizer;
use crate::version::zerv::Component;
use crate::version::zerv::core::Zerv;

impl SemVer {
    fn add_flattened_to_prerelease(&mut self, value: String) {
        for part in value.split('.') {
            if !part.is_empty() {
                let identifier = if let Ok(num) = part.parse::<u32>() {
                    PreReleaseIdentifier::UInt(num as u64)
                } else {
                    PreReleaseIdentifier::String(part.to_string())
                };
                self.pre_release
                    .get_or_insert_with(Vec::new)
                    .push(identifier);
            }
        }
    }

    fn add_flattened_to_build(&mut self, value: String) {
        for part in value.split('.') {
            if !part.is_empty() {
                let metadata = if let Ok(num) = part.parse::<u32>() {
                    BuildMetadata::UInt(num as u64)
                } else {
                    BuildMetadata::String(part.to_string())
                };
                self.build_metadata
                    .get_or_insert_with(Vec::new)
                    .push(metadata);
            }
        }
    }

    fn process_core(
        &mut self,
        components: &[Component],
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        int_sanitizer: &Sanitizer,
        semver_sanitizer: &Sanitizer,
    ) {
        let mut core_count = 0;

        for component in components {
            if let Some(value) = component.resolve_value(zerv_vars, int_sanitizer)
                && !value.is_empty()
                && let Ok(num) = value.parse::<u32>()
                && core_count < 3
            {
                match core_count {
                    0 => self.major = num as u64,
                    1 => self.minor = num as u64,
                    2 => self.patch = num as u64,
                    _ => unreachable!(),
                }
                core_count += 1;
                continue;
            }

            // All remaining components go to pre-release
            if let Some(value) = component.resolve_value(zerv_vars, semver_sanitizer)
                && !value.is_empty()
            {
                self.add_flattened_to_prerelease(value);
            }
        }
    }

    fn process_secondary_var(
        &mut self,
        var: &crate::version::zerv::Var,
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        semver_sanitizer: &Sanitizer,
    ) {
        let expanded = var.resolve_expanded_values(zerv_vars, semver_sanitizer);
        for value in expanded {
            if !value.is_empty() {
                let identifier = if let Ok(num) = value.parse::<u32>() {
                    PreReleaseIdentifier::UInt(num as u64)
                } else {
                    PreReleaseIdentifier::String(value)
                };
                self.pre_release
                    .get_or_insert_with(Vec::new)
                    .push(identifier);
            }
        }
    }

    fn process_extra_core(
        &mut self,
        components: &[Component],
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        semver_sanitizer: &Sanitizer,
    ) {
        for component in components {
            match component {
                Component::Var(var) if var.is_secondary_component() => {
                    self.process_secondary_var(var, zerv_vars, semver_sanitizer);
                }
                _ => {
                    if let Some(value) = component.resolve_value(zerv_vars, semver_sanitizer)
                        && !value.is_empty()
                    {
                        self.add_flattened_to_prerelease(value);
                    }
                }
            }
        }
    }

    fn process_build(
        &mut self,
        components: &[Component],
        zerv_vars: &crate::version::zerv::vars::ZervVars,
        semver_sanitizer: &Sanitizer,
    ) {
        for component in components {
            if let Some(value) = component.resolve_value(zerv_vars, semver_sanitizer)
                && !value.is_empty()
            {
                self.add_flattened_to_build(value);
            }
        }
    }
}

impl From<Zerv> for SemVer {
    fn from(zerv: Zerv) -> Self {
        let mut semver = SemVer {
            major: 0,
            minor: 0,
            patch: 0,
            pre_release: None,
            build_metadata: None,
        };
        let int_sanitizer = Sanitizer::uint();
        let semver_sanitizer = Sanitizer::semver_str();

        // Process core - first 3 parsable ints go to major/minor/patch, rest to pre-release
        semver.process_core(
            zerv.schema.core(),
            &zerv.vars,
            &int_sanitizer,
            &semver_sanitizer,
        );

        // Process extra_core - secondary components get labeled, others go to pre-release
        semver.process_extra_core(zerv.schema.extra_core(), &zerv.vars, &semver_sanitizer);

        // Process build - all components go to build metadata
        semver.process_build(zerv.schema.build(), &zerv.vars, &semver_sanitizer);

        semver
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
    // Custom field handling - core (goes to pre-release)
    #[case(from::v1_0_0_custom_core_field("core.field").build(), "1.0.0-core.field")]
    #[case(from::v1_0_0_custom_core_field("simple").build(), "1.0.0-simple")]
    #[case(from::v1_0_0_custom_core_field("multi.part.value").build(), "1.0.0-multi.part.value")]
    #[case(from::v1_0_0_custom_core_field("test_value").build(), "1.0.0-test.value")]
    #[case(from::v1_0_0_custom_core_field("Feature/API-v2").build(), "1.0.0-Feature.API.v2")]
    // Custom field handling - extra_core (goes to pre-release)
    #[case(from::v1_0_0_custom_extra_field("custom.field").build(), "1.0.0-custom.field")]
    #[case(from::v1_0_0_custom_extra_field("simple").build(), "1.0.0-simple")]
    #[case(from::v1_0_0_custom_extra_field("multi.part.value").build(), "1.0.0-multi.part.value")]
    #[case(from::v1_0_0_custom_extra_field("test_value").build(), "1.0.0-test.value")]
    #[case(from::v1_0_0_custom_extra_field("Feature/API-v2").build(), "1.0.0-Feature.API.v2")]
    // Custom field handling - build (goes to build metadata)
    #[case(from::v1_0_0_custom_build_field("build.field").build(), "1.0.0+build.field")]
    #[case(from::v1_0_0_custom_build_field("simple").build(), "1.0.0+simple")]
    #[case(from::v1_0_0_custom_build_field("multi.part.value").build(), "1.0.0+multi.part.value")]
    #[case(from::v1_0_0_custom_build_field("test_value").build(), "1.0.0+test.value")]
    #[case(from::v1_0_0_custom_build_field("Feature/API-v2").build(), "1.0.0+Feature.API.v2")]
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
    // Maximum complexity test
    #[case(from::v2_3_4_max_complexity().build(), "2.3.4-core.value.99.epoch.5.alpha.1.post.2.extra.value.literal.42+feature.complex.test.7.abcdef1.true.build.value.build.123")]
    fn test_zerv_to_semver_conversion(#[case] zerv: Zerv, #[case] expected_semver_str: &str) {
        let semver: SemVer = zerv.into();
        assert_eq!(semver.to_string(), expected_semver_str);
    }
}
