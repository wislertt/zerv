use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::constants::bump_types;
use crate::error::ZervError;

pub mod reset;
pub mod types;
pub mod vars_primary;
pub mod vars_secondary;
pub mod vars_timestamp;

impl Zerv {
    /// Apply component processing from VersionArgs following BumpType::PRECEDENCE_NAMES order
    pub fn apply_component_processing(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        use types::BumpType;

        // Process components in BumpType::PRECEDENCE_NAMES order
        for &component_name in BumpType::PRECEDENCE_NAMES {
            match component_name {
                bump_types::EPOCH => self.process_epoch(args)?,
                bump_types::MAJOR => self.process_major(args)?,
                bump_types::MINOR => self.process_minor(args)?,
                bump_types::PATCH => self.process_patch(args)?,
                bump_types::PRE_RELEASE_LABEL => self.process_pre_release_label(args)?,
                bump_types::PRE_RELEASE_NUM => self.process_pre_release_num(args)?,
                bump_types::POST => self.process_post(args)?,
                bump_types::DEV => self.process_dev(args)?,
                _ => unreachable!("Unknown component in PRECEDENCE_NAMES: {}", component_name),
            }
        }

        self.process_bumped_timestamp(args)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::version_args::OverrideType;
    use crate::test_utils::{VersionArgsFixture, ZervFixture};
    use crate::version::semver::SemVer;
    use crate::version::zerv::bump::types::BumpType;
    use rstest::*;

    // Test multiple bump combinations with reset logic
    #[rstest]
    #[case("1.2.3", vec![BumpType::Major(1), BumpType::Minor(2)], "2.2.0")]
    #[case("2.5.3", vec![BumpType::Patch(7)], "2.5.10")]
    #[case("0.0.0", vec![BumpType::Major(3), BumpType::Minor(2), BumpType::Patch(1)], "3.2.1")]
    #[case("1.2.3-alpha.1", vec![BumpType::Major(1)], "2.0.0")]
    fn test_apply_component_processing_multiple_bumps(
        #[case] starting_version: &str,
        #[case] bumps: Vec<BumpType>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        let args = VersionArgsFixture::new().with_bump_specs(bumps).build();

        zerv.apply_component_processing(&args).unwrap();

        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    // Test combined bump and override specifications
    #[rstest]
    #[case(
        "1.2.3",
        vec![BumpType::Major(1), BumpType::Minor(2)],
        vec![OverrideType::Major(2), OverrideType::Minor(3)],
        "3.5.0"
    )]
    #[case(
        "0.1.0",
        vec![BumpType::Patch(5)],
        vec![OverrideType::Major(1), OverrideType::Patch(10)],
        "1.1.15"
    )]
    fn test_apply_component_processing_bump_and_override(
        #[case] starting_version: &str,
        #[case] bumps: Vec<BumpType>,
        #[case] overrides: Vec<OverrideType>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        let args = VersionArgsFixture::new()
            .with_bump_specs(bumps)
            .with_override_specs(overrides)
            .build();

        // Apply context overrides first, then component processing
        zerv.vars.apply_context_overrides(&args).unwrap();
        zerv.apply_component_processing(&args).unwrap();

        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    // Test override-only specifications (no bumps)
    #[rstest]
    #[case(
        "1.2.3",
        vec![OverrideType::Major(5), OverrideType::Minor(0), OverrideType::Patch(9)],
        "5.0.9"
    )]
    #[case(
        "0.1.0-alpha.1",
        vec![OverrideType::Major(2), OverrideType::PreReleaseNum(5)],
        "2.1.0-alpha.5"
    )]
    fn test_apply_component_processing_override_only(
        #[case] starting_version: &str,
        #[case] overrides: Vec<OverrideType>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        let args = VersionArgsFixture::new()
            .with_override_specs(overrides)
            .build();

        // Apply context overrides first, then component processing
        zerv.vars.apply_context_overrides(&args).unwrap();
        zerv.apply_component_processing(&args).unwrap();

        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }
}
