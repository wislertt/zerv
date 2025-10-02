use super::Zerv;
use crate::error::ZervError;

impl Zerv {
    /// Bump post-release version by the specified increment
    pub fn bump_post(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.post = Some(self.vars.post.unwrap_or(0) + increment);
        Ok(())
    }

    /// Bump dev version by the specified increment
    pub fn bump_dev(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.dev = Some(self.vars.dev.unwrap_or(0) + increment);
        Ok(())
    }

    /// Bump pre-release number by the specified increment
    pub fn bump_pre_release(&mut self, increment: u64) -> Result<(), ZervError> {
        if let Some(ref mut pre_release) = self.vars.pre_release {
            pre_release.number = Some(pre_release.number.unwrap_or(0) + increment);
        } else {
            return Err(ZervError::InvalidVersion(
                "Cannot bump pre-release number: no pre-release exists".to_string(),
            ));
        }
        Ok(())
    }

    /// Bump epoch by the specified increment
    pub fn bump_epoch(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.epoch = Some(self.vars.epoch.unwrap_or(0) + increment);
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
        zerv.bump_post(increment).unwrap();
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
        zerv.bump_dev(increment).unwrap();
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
        zerv.bump_epoch(increment).unwrap();
        assert_eq!(zerv.vars.epoch, expected);
    }

    #[test]
    fn test_bump_pre_release_success() {
        let mut zerv = ZervFixture::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1));
        zerv.bump_pre_release(2).unwrap();
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().number, Some(3));
    }

    #[test]
    fn test_bump_pre_release_no_pre_release() {
        let mut zerv = ZervFixture::zerv_version(1, 0, 0);
        let result = zerv.bump_pre_release(1);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot bump pre-release number: no pre-release exists")
        );
    }
}
