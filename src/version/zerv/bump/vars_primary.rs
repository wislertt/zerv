use super::Zerv;
use crate::error::ZervError;
use crate::version::zerv::bump::precedence::Precedence;

impl Zerv {
    pub fn process_major(
        &mut self,
        override_value: Option<u32>,
        bump_value: Option<u32>,
    ) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_val) = override_value {
            self.vars.major = Some(override_val as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(increment) = bump_value {
            self.vars.major = Some(self.vars.major.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(&Precedence::Major)?;
        }

        Ok(())
    }

    pub fn process_minor(
        &mut self,
        override_value: Option<u32>,
        bump_value: Option<u32>,
    ) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_val) = override_value {
            self.vars.minor = Some(override_val as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(increment) = bump_value {
            self.vars.minor = Some(self.vars.minor.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(&Precedence::Minor)?;
        }

        Ok(())
    }

    pub fn process_patch(
        &mut self,
        override_value: Option<u32>,
        bump_value: Option<u32>,
    ) -> Result<(), ZervError> {
        // 1. Override step - set absolute value if specified
        if let Some(override_val) = override_value {
            self.vars.patch = Some(override_val as u64);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(increment) = bump_value {
            self.vars.patch = Some(self.vars.patch.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(&Precedence::Patch)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use crate::schema::ZervSchemaPreset;
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
    // Bump with 0 - should apply reset logic (adds 0 but resets lower components)
    #[case("1.2.3-alpha.1", None, Some(0), "1.0.0")]
    #[case("1.2.3", None, Some(0), "1.0.0")]
    #[case("1.0.0+build.123", None, Some(1), "2.0.0")]
    #[case("1.5.2-rc.1+build.456", None, Some(1), "2.0.0")]
    fn test_process_major(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostDevContext)
            .build();
        zerv.process_major(override_value, bump_increment).unwrap();
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
    // Bump with 0 - should apply reset logic (adds 0 but resets lower components)
    #[case("1.2.3-alpha.1", None, Some(0), "1.2.0")]
    #[case("1.2.3", None, Some(0), "1.2.0")]
    fn test_process_minor(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostDevContext)
            .build();
        zerv.process_minor(override_value, bump_increment).unwrap();
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
    // Bump with 0 - should apply reset logic (adds 0 but resets lower components)
    #[case("1.2.3-alpha.1", None, Some(0), "1.2.3")]
    #[case("1.2.3", None, Some(0), "1.2.3")]
    fn test_process_patch(
        #[case] starting_version: &str,
        #[case] override_value: Option<u32>,
        #[case] bump_increment: Option<u32>,
        #[case] expected_version: &str,
    ) {
        let mut zerv = ZervFixture::from_semver_str(starting_version)
            .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostDevContext)
            .build();
        zerv.process_patch(override_value, bump_increment).unwrap();
        let result_version: SemVer = zerv.into();
        assert_eq!(result_version.to_string(), expected_version);
    }
}
