use super::core::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::error::ZervError;

pub mod precedence;
pub mod reset;
pub mod schema_parsing;
pub mod schema_processing;
pub mod types;
pub mod vars_primary;
pub mod vars_secondary;
pub mod vars_timestamp;

impl Zerv {
    /// Apply component processing from VersionArgs in precedence order
    pub fn apply_component_processing(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        use crate::version::zerv::bump::precedence::Precedence;

        // Collect precedence order to avoid borrowing conflicts
        let precedence_order: Vec<Precedence> =
            self.schema.precedence_order.iter().cloned().collect();

        // Process components in precedence order
        for precedence in precedence_order {
            match precedence {
                Precedence::Epoch => {
                    self.process_epoch(args.overrides.epoch, args.bumps.bump_epoch.flatten())?
                }
                Precedence::Major => {
                    self.process_major(args.overrides.major, args.bumps.bump_major.flatten())?
                }
                Precedence::Minor => {
                    self.process_minor(args.overrides.minor, args.bumps.bump_minor.flatten())?
                }
                Precedence::Patch => {
                    self.process_patch(args.overrides.patch, args.bumps.bump_patch.flatten())?
                }
                Precedence::Core => self.process_schema_section(
                    "core",
                    &args.overrides.core,
                    &args.bumps.bump_core,
                )?,
                Precedence::PreReleaseLabel => self.process_pre_release_label(args)?,
                Precedence::PreReleaseNum => self.process_pre_release_num(
                    args.overrides.pre_release_num,
                    args.bumps.bump_pre_release_num.flatten(),
                )?,
                Precedence::Post => {
                    self.process_post(args.overrides.post, args.bumps.bump_post.flatten())?
                }
                Precedence::Dev => {
                    self.process_dev(args.overrides.dev, args.bumps.bump_dev.flatten())?
                }
                Precedence::ExtraCore => self.process_schema_section(
                    "extra_core",
                    &args.overrides.extra_core,
                    &args.bumps.bump_extra_core,
                )?,
                Precedence::Build => self.process_schema_section(
                    "build",
                    &args.overrides.build,
                    &args.bumps.bump_build,
                )?,
            }
        }

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
    // Doc 16 reset logic scenarios
    #[case("1.5.2-rc.1+build.456", vec![BumpType::Major(1)], "2.0.0")] // Major bump removes all lower precedence
    #[case("1.5.2-rc.1+build.456", vec![BumpType::Minor(1)], "1.6.0")] // Minor bump removes patch, pre-release, build
    #[case("1.5.2-rc.1+build.456", vec![BumpType::Patch(1)], "1.5.3")] // Patch bump removes pre-release, build
    #[case("1.0.0+build.123", vec![BumpType::Major(1)], "2.0.0")] // Build metadata removed on major bump
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
