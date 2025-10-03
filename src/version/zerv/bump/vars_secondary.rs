use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::constants::{bump_types, shared_constants};
use crate::error::ZervError;
use crate::version::zerv::core::{PreReleaseLabel, PreReleaseVar};

impl Zerv {
    pub fn process_post(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.post {
            self.vars.post = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_post
            && increment > 0
        {
            self.vars.post = Some(self.vars.post.unwrap_or(0) + increment as u64);
            self.vars
                .reset_lower_precedence_components(shared_constants::POST)?;
        }

        Ok(())
    }

    pub fn process_dev(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.dev {
            self.vars.dev = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_dev
            && increment > 0
        {
            self.vars.dev = Some(self.vars.dev.unwrap_or(0) + increment as u64);
            self.vars
                .reset_lower_precedence_components(shared_constants::DEV)?;
        }

        Ok(())
    }

    pub fn process_pre_release_label(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(ref label) = args.pre_release_label {
            let existing_number = self.vars.pre_release.as_ref().and_then(|pr| pr.number);
            self.vars.pre_release = Some(PreReleaseVar {
                label: PreReleaseLabel::try_from_str(label).ok_or_else(|| {
                    ZervError::InvalidVersion(format!("Invalid pre-release label: {label}"))
                })?,
                number: args
                    .pre_release_num
                    .map(|n| n as u64)
                    .or(existing_number)
                    .or(Some(0)),
            });
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(ref label) = args.bump_pre_release_label {
            let pre_release_label = label.parse::<PreReleaseLabel>()?;
            self.vars
                .reset_lower_precedence_components(bump_types::PRE_RELEASE_LABEL)?;
            self.vars.pre_release = Some(PreReleaseVar {
                label: pre_release_label,
                number: Some(0),
            });
        }

        Ok(())
    }

    pub fn process_pre_release_num(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(pre_release_num) = args.pre_release_num {
            // Only process if label wasn't already handled
            if args.pre_release_label.is_none() {
                if self.vars.pre_release.is_none() {
                    self.vars.pre_release = Some(PreReleaseVar {
                        label: PreReleaseLabel::Alpha,
                        number: Some(pre_release_num as u64),
                    });
                } else if let Some(ref mut pre_release) = self.vars.pre_release {
                    pre_release.number = Some(pre_release_num as u64);
                }
            }
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_pre_release_num
            && increment > 0
        {
            if let Some(ref mut pre_release) = self.vars.pre_release {
                pre_release.number = Some(pre_release.number.unwrap_or(0) + increment as u64);
                self.vars
                    .reset_lower_precedence_components(bump_types::PRE_RELEASE_NUM)?;
            } else {
                // Create alpha label with the increment when no pre-release exists
                self.vars.pre_release = Some(PreReleaseVar {
                    label: PreReleaseLabel::Alpha,
                    number: Some(increment as u64),
                });
                self.vars
                    .reset_lower_precedence_components(bump_types::PRE_RELEASE_NUM)?;
            }
        }

        Ok(())
    }

    pub fn process_epoch(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.epoch {
            self.vars.epoch = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_epoch
            && increment > 0
        {
            self.vars.epoch = Some(self.vars.epoch.unwrap_or(0) + increment as u64);
            self.vars
                .reset_lower_precedence_components(shared_constants::EPOCH)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::VersionArgsFixture;
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::semver::SemVer;
    use crate::version::zerv::Component;
    use rstest::*;

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
            .with_extra_core(Component::VarField("post".to_string()))
            .build();
        let mut args_fixture = VersionArgsFixture::new();
        if let Some(override_val) = override_value {
            args_fixture = args_fixture.with_post(override_val);
        }
        if let Some(bump_val) = bump_increment {
            args_fixture = args_fixture.with_bump_post(bump_val);
        }
        let args = args_fixture.build();
        zerv.process_post(&args).unwrap();
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
            .with_extra_core(Component::VarField("dev".to_string()))
            .build();
        let mut args_fixture = VersionArgsFixture::new();
        if let Some(override_val) = override_value {
            args_fixture = args_fixture.with_dev(override_val);
        }
        if let Some(bump_val) = bump_increment {
            args_fixture = args_fixture.with_bump_dev(bump_val);
        }
        let args = args_fixture.build();
        zerv.process_dev(&args).unwrap();
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
            .with_extra_core(Component::VarField("epoch".to_string()))
            .build();
        let mut args_fixture = VersionArgsFixture::new();
        if let Some(override_val) = override_value {
            args_fixture = args_fixture.with_epoch(override_val);
        }
        if let Some(bump_val) = bump_increment {
            args_fixture = args_fixture.with_bump_epoch(bump_val);
        }
        let args = args_fixture.build();
        zerv.process_epoch(&args).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    // TODO: review up to this

    #[rstest]
    // Override only tests
    #[case("1.0.0-alpha.5", Some("beta"), None, "1.0.0-beta.5.beta.5")] // Override preserves number
    #[case("1.0.0-beta", Some("alpha"), None, "1.0.0-alpha.0.alpha.0")] // None number becomes 0
    #[case("1.0.0", Some("alpha"), None, "1.0.0-alpha.0")] // No pre-release defaults to 0
    // Bump only tests
    #[case("1.0.0", None, Some("beta"), "1.0.0-beta.0")] // Bump creates new label with reset
    #[case("1.0.0-alpha.5", None, Some("rc"), "1.0.0-rc.0.rc.0")] // Bump resets number to 0
    // No operation tests
    #[case("1.2.3-alpha.1", None, None, "1.2.3-alpha.1.alpha.1")]
    fn test_process_pre_release_label(
        #[case] starting_version: &str,
        #[case] override_label: Option<&str>,
        #[case] bump_label: Option<&str>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_extra_core(Component::VarField("pre_release".to_string()))
            .build();
        let mut args_fixture = VersionArgsFixture::new();
        if let Some(label) = override_label {
            args_fixture = args_fixture.with_pre_release_label(label);
        }
        if let Some(label) = bump_label {
            args_fixture = args_fixture.with_bump_pre_release_label(label);
        }
        let args = args_fixture.build();
        zerv.process_pre_release_label(&args).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    // ================================================================================================
    // #[rstest]
    // #[case("1.0.0", "alpha", "1.0.0-alpha.0")]
    // #[case("1.0.0", "beta", "1.0.0-beta.0")]
    // #[case("1.0.0", "rc", "1.0.0-rc.0")]
    // fn test_process_pre_release_label(
    //     #[case] starting_version: &str,
    //     #[case] label: &str,
    //     #[case] expected_version: &str,
    // ) {
    //     let mut zerv = ZervFixture::from_semver_str(starting_version)
    //         .with_extra_core(Component::VarField(
    //             "pre_release".to_string(),
    //         ))
    //         .build();
    //     let args = VersionArgsFixture::new()
    //         .with_bump_pre_release_label(label)
    //         .build();
    //     zerv.process_pre_release_label(&args).unwrap();
    //     zerv.process_pre_release_num(&args).unwrap();
    //     let result_version: SemVer = zerv.into();
    //     assert_eq!(result_version.to_string(), expected_version);
    // }

    #[rstest]
    #[case("1.0.0-alpha.1", 2, "1.0.0-alpha.3")]
    #[case("1.0.0", 1, "1.0.0-alpha.1")]
    #[case("1.0.0", 5, "1.0.0-alpha.5")]
    fn test_process_pre_release_num(
        #[case] starting_version: &str,
        #[case] increment: u64,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_extra_core(Component::VarField("pre_release".to_string()))
            .build();
        let args = VersionArgsFixture::new()
            .with_bump_pre_release_num(increment as u32)
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    #[rstest]
    #[case("1.0.0-beta.2", 7, "1.0.0-beta.7")] // Override number only
    #[case("1.0.0", 5, "1.0.0-alpha.5")] // Create alpha when none exists
    fn test_pre_release_num_override(
        #[case] starting_version: &str,
        #[case] number: u64,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_extra_core(Component::VarField("pre_release".to_string()))
            .build();
        let args = VersionArgsFixture::new()
            .with_pre_release_num(number as u32)
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }

    #[test]
    fn test_bump_pre_release_label_invalid() {
        let mut zerv = ZervFixture::from_semver_str("1.0.0").build();
        let args = VersionArgsFixture::new()
            .with_bump_pre_release_label("invalid")
            .build();
        let result = zerv.process_pre_release_label(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid pre-release label")
        );
    }

    #[test]
    fn test_bump_pre_release_label_resets_lower_precedence() {
        let mut zerv = ZervFixture::from_semver_str("1.0.0-post.5.dev.10")
            .with_extra_core(Component::VarField("pre_release".to_string()))
            .build();
        let args = VersionArgsFixture::new()
            .with_bump_pre_release_label("alpha")
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), "1.0.0-alpha.0");
    }
}
