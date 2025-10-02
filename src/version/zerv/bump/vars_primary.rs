use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::constants::shared_constants;
use crate::error::ZervError;

impl Zerv {
    /// Process major version component with override, bump, and reset logic
    pub fn process_major(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.major {
            self.vars.major = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_major {
            self.vars.major = Some(self.vars.major.unwrap_or(0) + increment as u64);

            // Apply reset logic for lower precedence components
            self.vars
                .reset_lower_precedence_components(shared_constants::MAJOR)?;
        }

        Ok(())
    }

    /// Process minor version component with override, bump, and reset logic
    pub fn process_minor(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.minor {
            self.vars.minor = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_minor {
            self.vars.minor = Some(self.vars.minor.unwrap_or(0) + increment as u64);

            // Apply reset logic for lower precedence components
            self.vars
                .reset_lower_precedence_components(shared_constants::MINOR)?;
        }

        Ok(())
    }

    /// Process patch version component with override, bump, and reset logic
    pub fn process_patch(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_value) = args.patch {
            self.vars.patch = Some(override_value as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_patch {
            self.vars.patch = Some(self.vars.patch.unwrap_or(0) + increment as u64);

            // Apply reset logic for lower precedence components
            self.vars
                .reset_lower_precedence_components(shared_constants::PATCH)?;
        }

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
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_major_flag(increment as u32)
            .build();
        zerv.process_major(&args).unwrap();
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
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_minor_flag(increment as u32)
            .build();
        zerv.process_minor(&args).unwrap();
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
        let args = crate::test_utils::VersionArgsFixture::new()
            .with_bump_patch_flag(increment as u32)
            .build();
        zerv.process_patch(&args).unwrap();
        assert_eq!(zerv.vars.patch, expected);
    }
}
