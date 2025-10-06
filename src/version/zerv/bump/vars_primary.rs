use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::constants::shared_constants;
use crate::error::ZervError;

impl Zerv {
    pub fn process_major(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.major {
            self.vars.major = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_major
            && increment > 0
        {
            self.vars.major = Some(self.vars.major.unwrap_or(0) + increment as u64);
            self.vars
                .reset_lower_precedence_components(shared_constants::MAJOR)?;
        }

        Ok(())
    }

    pub fn process_minor(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.minor {
            self.vars.minor = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_minor
            && increment > 0
        {
            self.vars.minor = Some(self.vars.minor.unwrap_or(0) + increment as u64);
            self.vars
                .reset_lower_precedence_components(shared_constants::MINOR)?;
        }

        Ok(())
    }

    pub fn process_patch(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.patch {
            self.vars.patch = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_patch
            && increment > 0
        {
            self.vars.patch = Some(self.vars.patch.unwrap_or(0) + increment as u64);
            self.vars
                .reset_lower_precedence_components(shared_constants::PATCH)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use crate::test_utils::VersionArgsFixture;
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::semver::SemVer;

    #[rstest]
    // Bump only tests
    #[case("1.0.0", None, Some(2), "3.0.0")]
    #[case("2.5.3", None, Some(1), "3.0.0")]
    #[case("0.0.0", None, Some(5), "5.0.0")]
    #[case("1.2.3-alpha.1", None, Some(1), "2.0.0")]
    #[case("2.1.0-beta.2", None, Some(2), "4.0.0")]
    // Override only tests
    #[case("1.0.0", Some(5), None, "5.0.0")]
    #[case("2.5.3", Some(0), None, "0.5.3")]
    #[case("1.2.3-alpha.1", Some(3), None, "3.2.3-alpha.1")]
    // Override + Bump tests (override first, then bump)
    #[case("1.0.0", Some(2), Some(1), "3.0.0")]
    #[case("2.5.3", Some(0), Some(3), "3.0.0")]
    // No operation tests
    #[case("1.2.3", None, None, "1.2.3")]
    // Bump with 0 - should be no-op (no reset logic applied)
    #[case("1.2.3", None, Some(0), "1.2.3")]
    // #[case("1.0.0+build.123", None, Some(1), "2.0.0+build.123")] // TODO: think about reset state apart from ZervVars later
    // #[case("1.5.2-rc.1+build.456", None, Some(1), "2.0.0+build.456")]
    fn test_process_major(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        let mut args_fixture = VersionArgsFixture::new();
        if let Some(override_val) = override_value {
            args_fixture = args_fixture.with_major(override_val);
        }
        if let Some(bump_val) = bump_increment {
            args_fixture = args_fixture.with_bump_major(bump_val);
        }
        let args = args_fixture.build();
        zerv.process_major(&args).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    #[rstest]
    // Bump only tests
    #[case("1.0.0", None, Some(3), "1.3.0")]
    #[case("1.2.3", None, Some(1), "1.3.0")]
    #[case("0.0.0", None, Some(7), "0.7.0")]
    #[case("1.2.3-alpha.1", None, Some(1), "1.3.0")]
    #[case("2.1.5-beta.2", None, Some(2), "2.3.0")]
    // Override only tests
    #[case("1.0.0", Some(5), None, "1.5.0")]
    #[case("2.5.3", Some(0), None, "2.0.3")]
    #[case("1.2.3-alpha.1", Some(8), None, "1.8.3-alpha.1")]
    // Override + Bump tests (override first, then bump)
    #[case("1.0.0", Some(2), Some(1), "1.3.0")]
    #[case("2.5.3", Some(0), Some(4), "2.4.0")]
    // No operation tests
    #[case("1.2.3", None, None, "1.2.3")]
    // Bump with 0 - should be no-op (no reset logic applied)
    #[case("1.2.3", None, Some(0), "1.2.3")]
    fn test_process_minor(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        let mut args_fixture = VersionArgsFixture::new();
        if let Some(override_val) = override_value {
            args_fixture = args_fixture.with_minor(override_val);
        }
        if let Some(bump_val) = bump_increment {
            args_fixture = args_fixture.with_bump_minor(bump_val);
        }
        let args = args_fixture.build();
        zerv.process_minor(&args).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    #[rstest]
    // Bump only tests
    #[case("1.0.0", None, Some(4), "1.0.4")]
    #[case("1.2.3", None, Some(2), "1.2.5")]
    #[case("0.0.0", None, Some(9), "0.0.9")]
    #[case("1.2.3-alpha.1", None, Some(1), "1.2.4")]
    #[case("2.1.5-beta.2", None, Some(3), "2.1.8")]
    // Override only tests
    #[case("1.0.0", Some(7), None, "1.0.7")]
    #[case("2.5.3", Some(0), None, "2.5.0")]
    #[case("1.2.3-alpha.1", Some(9), None, "1.2.9-alpha.1")]
    // Override + Bump tests (override first, then bump)
    #[case("1.0.0", Some(2), Some(3), "1.0.5")]
    #[case("2.5.3", Some(0), Some(6), "2.5.6")]
    // No operation tests
    #[case("1.2.3", None, None, "1.2.3")]
    // Bump with 0 - should be no-op (no reset logic applied)
    #[case("1.2.3", None, Some(0), "1.2.3")]
    fn test_process_patch(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        let mut args_fixture = VersionArgsFixture::new();
        if let Some(override_val) = override_value {
            args_fixture = args_fixture.with_patch(override_val);
        }
        if let Some(bump_val) = bump_increment {
            args_fixture = args_fixture.with_bump_patch(bump_val);
        }
        let args = args_fixture.build();
        zerv.process_patch(&args).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }
}
