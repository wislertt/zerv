use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::constants::{bump_types, shared_constants};
use crate::error::ZervError;
use crate::version::zerv::core::{PreReleaseLabel, PreReleaseVar};

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

    /// Process pre-release label component with override, bump, and reset logic
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

    /// Process pre-release number component with override, bump, and reset logic
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
        if let Some(Some(increment)) = args.bump_pre_release_num {
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
        let mut zerv = ZervFixture::new()
            .with_version(version.0, version.1, version.2)
            .build();
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
        let mut zerv = ZervFixture::new()
            .with_version(version.0, version.1, version.2)
            .build();
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
        let mut zerv = ZervFixture::new()
            .with_version(version.0, version.1, version.2)
            .build();
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_epoch_flag(increment as u32)
            .build();
        zerv.process_epoch(&args).unwrap();
        assert_eq!(zerv.vars.epoch, expected);
    }

    #[test]
    fn test_bump_pre_release_success() {
        let mut zerv = ZervFixture::new()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .build();
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_num_flag(2)
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().number, Some(3));
    }

    #[test]
    fn test_bump_pre_release_no_pre_release() {
        let mut zerv = ZervFixture::new().with_version(1, 0, 0).build();
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_num_flag(1)
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Should create alpha label with the increment when no pre-release exists
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Alpha);
        assert_eq!(pre_release.number, Some(1));
    }

    #[test]
    fn test_bump_pre_release_no_pre_release_large_increment() {
        let mut zerv = ZervFixture::new().with_version(1, 0, 0).build();
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_num_flag(5)
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Should create alpha label with the increment when no pre-release exists
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Alpha);
        assert_eq!(pre_release.number, Some(5));
    }

    #[test]
    fn test_bump_pre_release_label_alpha() {
        let mut zerv = ZervFixture::new().with_version(1, 0, 0).build();
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_label("alpha")
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Alpha);
        assert_eq!(pre_release.number, Some(0));
    }

    #[test]
    fn test_bump_pre_release_label_beta() {
        let mut zerv = ZervFixture::new().with_version(1, 0, 0).build();
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_label("beta")
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Beta);
        assert_eq!(pre_release.number, Some(0));
    }

    #[test]
    fn test_bump_pre_release_label_rc() {
        let mut zerv = ZervFixture::new().with_version(1, 0, 0).build();
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_label("rc")
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Rc);
        assert_eq!(pre_release.number, Some(0));
    }

    #[test]
    fn test_bump_pre_release_label_invalid() {
        let mut zerv = ZervFixture::new().with_version(1, 0, 0).build();
        let args = crate::test_utils::VersionArgsFixture::new()
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
        // Start with version 1.0.0 with post and dev components
        let mut zerv = ZervFixture::new().with_version(1, 0, 0).build();
        zerv.vars.post = Some(5);
        zerv.vars.dev = Some(10);

        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_label("alpha")
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Verify pre-release was set
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Alpha);
        assert_eq!(pre_release.number, Some(0));

        // Verify lower precedence components were reset
        assert_eq!(zerv.vars.post, None);
        assert_eq!(zerv.vars.dev, None);

        // Verify higher precedence components were preserved
        assert_eq!(zerv.vars.major, Some(1));
        assert_eq!(zerv.vars.minor, Some(0));
        assert_eq!(zerv.vars.patch, Some(0));
    }

    #[test]
    fn test_bump_pre_release_label_overwrites_existing() {
        // Start with existing pre-release
        let mut zerv = ZervFixture::new()
            .with_pre_release(PreReleaseLabel::Beta, Some(5))
            .build();

        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_pre_release_label("rc")
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Verify pre-release was overwritten
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Rc);
        assert_eq!(pre_release.number, Some(0)); // Reset to 0
    }

    #[test]
    fn test_override_pre_release_label_preserves_existing_number() {
        // Start with existing pre-release
        let mut zerv = ZervFixture::new()
            .with_pre_release(PreReleaseLabel::Alpha, Some(5))
            .build();

        let args = crate::test_utils::VersionArgsFixture::new()
            .with_pre_release_label("beta")
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Verify label changed but number preserved
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Beta);
        assert_eq!(pre_release.number, Some(5)); // Should preserve existing number
    }

    #[test]
    fn test_override_pre_release_label_uses_zero_when_no_existing_number() {
        // Start with no pre-release
        let mut zerv = ZervFixture::new().with_version(1, 0, 0).build();

        let args = crate::test_utils::VersionArgsFixture::new()
            .with_pre_release_label("alpha")
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Verify label set and number defaults to 0
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Alpha);
        assert_eq!(pre_release.number, Some(0)); // Should default to 0
    }

    #[test]
    fn test_override_pre_release_num_only() {
        // Start with existing pre-release
        let mut zerv = ZervFixture::new()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .build();

        let args = crate::test_utils::VersionArgsFixture::new()
            .with_pre_release_num(7)
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Verify number changed but label preserved
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Beta); // Should preserve existing label
        assert_eq!(pre_release.number, Some(7)); // Should use new number
    }

    #[test]
    fn test_override_both_pre_release_label_and_num() {
        // Start with existing pre-release
        let mut zerv = ZervFixture::new()
            .with_pre_release(PreReleaseLabel::Alpha, Some(3))
            .build();

        let args = crate::test_utils::VersionArgsFixture::new()
            .with_pre_release_label("rc")
            .with_pre_release_num(9)
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Verify both label and number changed
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Rc);
        assert_eq!(pre_release.number, Some(9));
    }

    #[test]
    fn test_override_pre_release_label_with_none_number() {
        // Start with pre-release that has None number
        let mut zerv = ZervFixture::new()
            .with_pre_release(PreReleaseLabel::Beta, None)
            .build();

        let args = crate::test_utils::VersionArgsFixture::new()
            .with_pre_release_label("alpha")
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Verify label changed and number defaults to 0
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Alpha);
        assert_eq!(pre_release.number, Some(0));
    }

    #[test]
    fn test_override_pre_release_num_creates_alpha_when_none_exists() {
        // Start with no pre-release
        let mut zerv = ZervFixture::new().with_version(1, 0, 0).build();

        let args = crate::test_utils::VersionArgsFixture::new()
            .with_pre_release_num(5)
            .build();
        zerv.process_pre_release_label(&args).unwrap();
        zerv.process_pre_release_num(&args).unwrap();

        // Verify alpha label created with specified number
        assert!(zerv.vars.pre_release.is_some());
        let pre_release = zerv.vars.pre_release.as_ref().unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Alpha);
        assert_eq!(pre_release.number, Some(5));
    }
}
