use super::Zerv;
use crate::error::ZervError;

impl Zerv {
    /// Bump major version by the specified increment
    pub fn bump_major(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.major = Some(self.vars.major.unwrap_or(0) + increment);
        Ok(())
    }

    /// Bump minor version by the specified increment
    pub fn bump_minor(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.minor = Some(self.vars.minor.unwrap_or(0) + increment);
        Ok(())
    }

    /// Bump patch version by the specified increment
    pub fn bump_patch(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.patch = Some(self.vars.patch.unwrap_or(0) + increment);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::zerv::ZervFixture;
    use rstest::*;

    #[rstest]
    #[case((1, 0, 0), 2, Some(3))]
    #[case((2, 5, 3), 1, Some(3))]
    #[case((0, 0, 0), 5, Some(5))]
    fn test_bump_major(
        #[case] version: (u64, u64, u64),
        #[case] increment: u64,
        #[case] expected: Option<u64>,
    ) {
        let mut zerv = ZervFixture::zerv_version(version.0, version.1, version.2);
        zerv.bump_major(increment).unwrap();
        assert_eq!(zerv.vars.major, expected);
    }

    #[rstest]
    #[case((1, 0, 0), 3, Some(3))]
    #[case((1, 2, 3), 1, Some(3))]
    #[case((0, 0, 0), 7, Some(7))]
    fn test_bump_minor(
        #[case] version: (u64, u64, u64),
        #[case] increment: u64,
        #[case] expected: Option<u64>,
    ) {
        let mut zerv = ZervFixture::zerv_version(version.0, version.1, version.2);
        zerv.bump_minor(increment).unwrap();
        assert_eq!(zerv.vars.minor, expected);
    }

    #[rstest]
    #[case((1, 0, 0), 4, Some(4))]
    #[case((1, 2, 3), 2, Some(5))]
    #[case((0, 0, 0), 9, Some(9))]
    fn test_bump_patch(
        #[case] version: (u64, u64, u64),
        #[case] increment: u64,
        #[case] expected: Option<u64>,
    ) {
        let mut zerv = ZervFixture::zerv_version(version.0, version.1, version.2);
        zerv.bump_patch(increment).unwrap();
        assert_eq!(zerv.vars.patch, expected);
    }
}
