use super::Zerv;
use crate::cli::version::args::ResolvedArgs;
use crate::error::ZervError;
use crate::version::zerv::bump::precedence::Precedence;
use crate::version::zerv::core::{
    PreReleaseLabel,
    PreReleaseVar,
};

impl Zerv {
    pub fn process_post(
        &mut self,
        override_value: Option<u32>,
        bump_value: Option<u32>,
    ) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_val) = override_value {
            self.vars.post = Some(override_val as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(increment) = bump_value
            && increment > 0
        {
            self.vars.post = Some(self.vars.post.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(&Precedence::Post)?;
        }

        Ok(())
    }

    pub fn process_dev(
        &mut self,
        override_value: Option<u32>,
        bump_value: Option<u32>,
    ) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_val) = override_value {
            self.vars.dev = Some(override_val as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(increment) = bump_value
            && increment > 0
        {
            self.vars.dev = Some(self.vars.dev.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(&Precedence::Dev)?;
        }

        Ok(())
    }

    pub fn process_pre_release_label(&mut self, args: &ResolvedArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(ref label) = args.overrides.pre_release_label {
            let existing_number = self.vars.pre_release.as_ref().and_then(|pr| pr.number);
            self.vars.pre_release = Some(PreReleaseVar {
                label: PreReleaseLabel::try_from_str(label).ok_or_else(|| {
                    ZervError::InvalidVersion(format!("Invalid pre-release label: {label}"))
                })?,
                number: args
                    .overrides
                    .pre_release_num
                    .map(|n| n as u64)
                    .or(existing_number)
                    .or(Some(0)),
            });
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(ref label) = args.bumps.bump_pre_release_label {
            let pre_release_label = label.parse::<PreReleaseLabel>()?;
            self.reset_lower_precedence_components(&Precedence::PreReleaseLabel)?;
            self.vars.pre_release = Some(PreReleaseVar {
                label: pre_release_label,
                number: Some(0),
            });
        }

        Ok(())
    }

    pub fn process_pre_release_num(
        &mut self,
        override_value: Option<u32>,
        bump_value: Option<u32>,
    ) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(pre_release_num) = override_value {
            if self.vars.pre_release.is_none() {
                self.vars.pre_release = Some(PreReleaseVar {
                    label: PreReleaseLabel::Alpha,
                    number: Some(pre_release_num as u64),
                });
            } else if let Some(ref mut pre_release) = self.vars.pre_release {
                pre_release.number = Some(pre_release_num as u64);
            }
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(increment) = bump_value
            && increment > 0
        {
            if let Some(ref mut pre_release) = self.vars.pre_release {
                pre_release.number = Some(pre_release.number.unwrap_or(0) + increment as u64);
                self.reset_lower_precedence_components(&Precedence::PreReleaseNum)?;
            } else {
                // Create alpha label with the increment when no pre-release exists
                self.vars.pre_release = Some(PreReleaseVar {
                    label: PreReleaseLabel::Alpha,
                    number: Some(increment as u64),
                });
                self.reset_lower_precedence_components(&Precedence::PreReleaseNum)?;
            }
        }

        Ok(())
    }

    pub fn process_epoch(
        &mut self,
        override_value: Option<u32>,
        bump_value: Option<u32>,
    ) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_val) = override_value {
            self.vars.epoch = Some(override_val as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(increment) = bump_value
            && increment > 0
        {
            self.vars.epoch = Some(self.vars.epoch.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(&Precedence::Epoch)?;
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
    #[case("1.0.0", None, Some(3), "1.0.0-post.3")]
    #[case("1.2.3-alpha.1", None, Some(2), "1.2.3-alpha.1.post.2")]
    // Override only tests
    #[case("1.0.0", Some(5), None, "1.0.0-post.5")]
    #[case("1.2.3-alpha.1", Some(0), None, "1.2.3-alpha.1.post.0")]
    // Override + Bump tests (override first, then bump)
    #[case("1.0.0", Some(2), Some(1), "1.0.0-post.3")]
    // No operation tests
    #[case("1.2.3", None, None, "1.2.3")]
    // Bump with 0 - should be no-op (no reset logic applied)
    #[case("1.2.3", None, Some(0), "1.2.3")]
    fn test_process_post(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_2()
            .build();
        zerv.process_post(override_value, bump_increment).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    #[rstest]
    // Bump only tests
    #[case("1.0.0", None, Some(2), "1.0.0-dev.2")]
    #[case("1.2.3-alpha.1", None, Some(3), "1.2.3-alpha.1.dev.3")]
    // Override only tests
    #[case("1.0.0", Some(7), None, "1.0.0-dev.7")]
    #[case("1.2.3-alpha.1", Some(0), None, "1.2.3-alpha.1.dev.0")]
    // Override + Bump tests (override first, then bump)
    #[case("1.0.0", Some(1), Some(2), "1.0.0-dev.3")]
    // No operation tests
    #[case("1.2.3", None, None, "1.2.3")]
    // Bump with 0 - should be no-op (no reset logic applied)
    #[case("1.2.3", None, Some(0), "1.2.3")]
    fn test_process_dev(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        zerv.process_dev(override_value, bump_increment).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    #[rstest]
    // Bump only tests
    #[case("1.0.0", None, Some(1), "0.0.0-epoch.1")]
    #[case("1.2.3-alpha.1", None, Some(2), "0.0.0-epoch.2")]
    // Override only tests
    #[case("1.0.0", Some(3), None, "1.0.0-epoch.3")]
    #[case("1.2.3-alpha.1", Some(0), None, "1.2.3-epoch.0.alpha.1")]
    // Override + Bump tests (override first, then bump)
    #[case("1.0.0", Some(1), Some(2), "0.0.0-epoch.3")]
    // No operation tests
    #[case("1.2.3", None, None, "1.2.3")]
    // Bump with 0 - should be no-op (no reset logic applied)
    #[case("1.2.3", None, Some(0), "1.2.3")]
    fn test_process_epoch(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        zerv.process_epoch(override_value, bump_increment).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    #[rstest]
    // Override only tests
    #[case("1.0.0-alpha.5", Some("beta"), None, "1.0.0-beta.5")] // Override preserves number
    #[case("1.0.0-beta", Some("alpha"), None, "1.0.0-alpha.0")] // None number becomes 0
    #[case("1.0.0", Some("alpha"), None, "1.0.0-alpha.0")] // No pre-release defaults to 0
    // Bump only tests
    #[case("1.0.0", None, Some("beta"), "1.0.0-beta.0")] // Bump creates new label with reset
    #[case("1.0.0-alpha.5", None, Some("rc"), "1.0.0-rc.0")] // Bump resets number to 0
    #[case("1.0.0-post.5.dev.10", None, Some("alpha"), "1.0.0-alpha.0")] // Bump resets lower precedence
    // No operation tests
    #[case("1.2.3-alpha.1", None, None, "1.2.3-alpha.1")]
    fn test_process_pre_release_label(
        #[case] starting_version: &str,
        #[case] override_label: Option<&str>,
        #[case] bump_label: Option<&str>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        let mut args_fixture = VersionArgsFixture::new();
        if let Some(label) = override_label {
            args_fixture = args_fixture.with_pre_release_label(label);
        }
        if let Some(label) = bump_label {
            args_fixture = args_fixture.with_bump_pre_release_label(label);
        }
        let args = args_fixture.build();
        let dummy_zerv = crate::test_utils::zerv::ZervFixture::new().build();
        let resolved_args =
            crate::cli::version::args::ResolvedArgs::resolve(&args, &dummy_zerv).unwrap();
        zerv.process_pre_release_label(&resolved_args).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    #[rstest]
    // Bump only tests
    #[case("1.0.0-alpha.1", None, Some(2), "1.0.0-alpha.3")]
    #[case("1.0.0", None, Some(1), "1.0.0-alpha.1")]
    #[case("1.0.0", None, Some(5), "1.0.0-alpha.5")]
    // Override only tests
    #[case("1.0.0-beta.2", Some(7), None, "1.0.0-beta.7")]
    #[case("1.0.0", Some(5), None, "1.0.0-alpha.5")]
    // Override + Bump tests (override first, then bump)
    #[case("1.0.0-alpha.1", Some(3), Some(2), "1.0.0-alpha.5")]
    // No operation tests
    #[case("1.2.3-alpha.1", None, None, "1.2.3-alpha.1")]
    // Bump with 0 - should be no-op (no reset logic applied)
    #[case("1.2.3-alpha.1", None, Some(0), "1.2.3-alpha.1")]
    fn test_process_pre_release_num(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_standard_tier_3()
            .build();
        zerv.process_pre_release_num(override_value, bump_increment)
            .unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    #[test]
    fn test_bump_pre_release_label_invalid() {
        let mut zerv = ZervFixture::from_semver_str("1.0.0").build();
        let args = VersionArgsFixture::new()
            .with_bump_pre_release_label("invalid")
            .build();
        let dummy_zerv = crate::test_utils::zerv::ZervFixture::new().build();
        let resolved_args =
            crate::cli::version::args::ResolvedArgs::resolve(&args, &dummy_zerv).unwrap();
        let result = zerv.process_pre_release_label(&resolved_args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid pre-release label")
        );
    }
}
