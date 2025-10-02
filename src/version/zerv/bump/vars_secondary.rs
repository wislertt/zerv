use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::constants::shared_constants;
use crate::error::ZervError;

impl Zerv {
    /// Process post-release component with override, bump, and reset logic
    pub fn process_post(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.post {
            self.vars.post = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_post {
            self.vars.post = Some(self.vars.post.unwrap_or(0) + increment as u64);

            // Apply reset logic for lower precedence components
            self.vars
                .reset_lower_precedence_components(shared_constants::POST)?;
        }

        Ok(())
    }

    /// Process dev component with override, bump, and reset logic
    pub fn process_dev(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.dev {
            self.vars.dev = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_dev {
            self.vars.dev = Some(self.vars.dev.unwrap_or(0) + increment as u64);

            // Apply reset logic for lower precedence components
            self.vars
                .reset_lower_precedence_components(shared_constants::DEV)?;
        }

        Ok(())
    }

    /// Process pre-release component with override, bump, and reset logic
    pub fn process_pre_release(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - handle pre-release label and number overrides
        if let Some(ref label) = args.pre_release_label {
            // Import the normalize function
            use crate::version::zerv::utils::general::normalize_pre_release_label;

            self.vars.pre_release = Some(crate::version::zerv::core::PreReleaseVar {
                label: normalize_pre_release_label(label).ok_or_else(|| {
                    ZervError::InvalidVersion(format!("Invalid pre-release label: {label}"))
                })?,
                number: args.pre_release_num.map(|n| n as u64),
            });
        } else if let Some(override_num) = args.pre_release_num {
            // If only number is specified, create alpha label if none exists
            if self.vars.pre_release.is_none() {
                self.vars.pre_release = Some(crate::version::zerv::core::PreReleaseVar {
                    label: crate::version::zerv::core::PreReleaseLabel::Alpha,
                    number: Some(override_num as u64),
                });
            } else if let Some(ref mut pre_release) = self.vars.pre_release {
                pre_release.number = Some(override_num as u64);
            }
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_pre_release_num {
            if let Some(ref mut pre_release) = self.vars.pre_release {
                pre_release.number = Some(pre_release.number.unwrap_or(0) + increment as u64);

                // Apply reset logic for lower precedence components
                self.vars
                    .reset_lower_precedence_components(shared_constants::PRE_RELEASE)?;
            } else {
                return Err(ZervError::InvalidVersion(
                    "Cannot bump pre-release number: no pre-release exists".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Process epoch component with override, bump, and reset logic
    pub fn process_epoch(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.epoch {
            self.vars.epoch = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_epoch {
            self.vars.epoch = Some(self.vars.epoch.unwrap_or(0) + increment as u64);

            // Apply reset logic for lower precedence components
            self.vars
                .reset_lower_precedence_components(shared_constants::EPOCH)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::zerv::core::PreReleaseLabel;
    use rstest::*;

    #[rstest]
    #[case((1, 0, 0), 3, Some(3))]
    #[case((1, 0, 0), 0, Some(0))]
    fn test_bump_post(
        #[case] version: (u64, u64, u64),
        #[case] increment: u64,
        #[case] expected: Option<u64>,
    ) {
        let mut zerv = ZervFixture::zerv_version(version.0, version.1, version.2);
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_post_flag(increment as u32)
            .build();
        zerv.process_post(&args).unwrap();
        assert_eq!(zerv.vars.post, expected);
    }

    #[rstest]
    #[case((1, 0, 0), 2, Some(2))]
    #[case((1, 0, 0), 0, Some(0))]
    fn test_bump_dev(
        #[case] version: (u64, u64, u64),
        #[case] increment: u64,
        #[case] expected: Option<u64>,
    ) {
        let mut zerv = ZervFixture::zerv_version(version.0, version.1, version.2);
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_dev_flag(increment as u32)
            .build();
        zerv.process_dev(&args).unwrap();
        assert_eq!(zerv.vars.dev, expected);
    }

    #[rstest]
    #[case((1, 0, 0), 1, Some(1))]
    #[case((1, 0, 0), 0, Some(0))]
    fn test_bump_epoch(
        #[case] version: (u64, u64, u64),
        #[case] increment: u64,
        #[case] expected: Option<u64>,
    ) {
        let mut zerv = ZervFixture::zerv_version(version.0, version.1, version.2);
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_epoch_flag(increment as u32)
            .build();
        zerv.process_epoch(&args).unwrap();
        assert_eq!(zerv.vars.epoch, expected);
    }

    #[test]
    fn test_bump_pre_release_success() {
        let mut zerv = ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1));
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_num_flag(2)
            .build();
        zerv.process_pre_release(&args).unwrap();
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().number, Some(3));
    }

    #[test]
    fn test_bump_pre_release_no_pre_release() {
        let mut zerv = ZervFixture::zerv_version(1, 0, 0);
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_num_flag(1)
            .build();
        let result = zerv.process_pre_release(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot bump pre-release number: no pre-release exists")
        );
    }
}
