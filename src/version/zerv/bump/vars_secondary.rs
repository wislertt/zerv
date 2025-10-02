use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::constants::shared_constants;
use crate::error::ZervError;

impl Zerv {
    /// Process post-release version bump with reset logic
    pub fn process_post(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Apply bump if requested
        if let Some(Some(increment)) = args.bump_post {
            self.vars.post = Some(self.vars.post.unwrap_or(0) + increment as u64);

            // Apply reset logic for lower precedence components
            self.vars
                .reset_lower_precedence_components(shared_constants::POST)?;
        }

        Ok(())
    }

    /// Process dev version bump with reset logic
    pub fn process_dev(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Apply bump if requested
        if let Some(Some(increment)) = args.bump_dev {
            self.vars.dev = Some(self.vars.dev.unwrap_or(0) + increment as u64);

            // Apply reset logic for lower precedence components
            self.vars
                .reset_lower_precedence_components(shared_constants::DEV)?;
        }

        Ok(())
    }

    /// Process pre-release number bump with reset logic
    pub fn process_pre_release(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Apply bump if requested
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

    /// Process epoch bump with reset logic
    pub fn process_epoch(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Apply bump if requested
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
        let args = crate::test_utils::VersionArgsFixture::with_bump_post(increment as u32);
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
        let args = crate::test_utils::VersionArgsFixture::with_bump_dev(increment as u32);
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
        let args = crate::test_utils::VersionArgsFixture::with_bump_epoch(increment as u32);
        zerv.process_epoch(&args).unwrap();
        assert_eq!(zerv.vars.epoch, expected);
    }

    #[test]
    fn test_bump_pre_release_success() {
        let mut zerv = ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1));
        let args = crate::test_utils::VersionArgsFixture::with_bump_pre_release_num(2);
        zerv.process_pre_release(&args).unwrap();
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().number, Some(3));
    }

    #[test]
    fn test_bump_pre_release_no_pre_release() {
        let mut zerv = ZervFixture::zerv_version(1, 0, 0);
        let args = crate::test_utils::VersionArgsFixture::with_bump_pre_release_num(1);
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
