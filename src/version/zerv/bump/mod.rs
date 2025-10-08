use super::core::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::error::ZervError;

pub mod precedence;
pub mod reset;
pub mod schema;
pub mod types;
pub mod vars_primary;
pub mod vars_secondary;
pub mod vars_timestamp;

impl Zerv {
    /// Apply component processing from VersionArgs in precedence order
    pub fn apply_component_processing(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Process components in precedence order (epoch, major, minor, patch, pre_release_label, pre_release_num, post, dev)
        self.process_epoch(args.overrides.epoch, args.bumps.bump_epoch.flatten())?;
        self.process_major(args.overrides.major, args.bumps.bump_major.flatten())?;
        self.process_minor(args.overrides.minor, args.bumps.bump_minor.flatten())?;
        self.process_patch(args.overrides.patch, args.bumps.bump_patch.flatten())?;
        self.process_pre_release_label(args)?;
        self.process_pre_release_num(
            args.overrides.pre_release_num,
            args.bumps.bump_pre_release_num.flatten(),
        )?;
        self.process_post(args.overrides.post, args.bumps.bump_post.flatten())?;
        self.process_dev(args.overrides.dev, args.bumps.bump_dev.flatten())?;

        // Process schema-based components (both overrides and bumps)
        self.process_schema_core(&args.overrides.core, &args.bumps.bump_core)?;
        self.process_schema_extra_core(&args.overrides.extra_core, &args.bumps.bump_extra_core)?;
        self.process_schema_build(&args.overrides.build, &args.bumps.bump_build)?;

        self.process_bumped_timestamp(args)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use crate::test_utils::version_args::OverrideType;
    use crate::test_utils::{
        VersionArgsFixture,
        ZervFixture,
    };
    use crate::version::semver::SemVer;
    use crate::version::zerv::bump::types::BumpType;

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
