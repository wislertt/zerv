use crate::error::ZervError;
use crate::version::zerv::bump::types::BumpType;
use crate::version::zerv::vars::ZervVars;

impl ZervVars {
    /// Reset all components with lower precedence than the given component
    pub fn reset_lower_precedence_components(&mut self, component: &str) -> Result<(), ZervError> {
        let current_precedence = BumpType::precedence_from_str(component);

        // Loop through all bump types in precedence order and reset those with lower precedence
        for (index, bump_type) in BumpType::PRECEDENCE_ORDER.iter().enumerate() {
            if index > current_precedence {
                self.reset_component(bump_type);
            }
        }

        Ok(())
    }

    /// Reset a specific component based on its bump type
    fn reset_component(&mut self, bump_type: &BumpType) {
        match bump_type {
            BumpType::Epoch(_) => self.epoch = Some(0),
            BumpType::Major(_) => self.major = Some(0),
            BumpType::Minor(_) => self.minor = Some(0),
            BumpType::Patch(_) => self.patch = Some(0),
            BumpType::PreReleaseLabel(_) => self.pre_release = None,
            BumpType::PreReleaseNum(_) => {
                if let Some(ref mut pre_release) = self.pre_release {
                    pre_release.number = Some(0);
                }
            }
            BumpType::Post(_) => self.post = None,
            BumpType::Dev(_) => self.dev = None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::zerv::ZervVarsFixture;
    use crate::version::zerv::core::{PreReleaseLabel, PreReleaseVar};
    use rstest::*;

    /// Helper function to create the standard starting fixture for reset tests
    fn full_vars_fixture() -> ZervVarsFixture {
        ZervVarsFixture::with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(5)
            .with_dev(6)
    }

    #[rstest]
    #[case(
        crate::constants::bump_types::EPOCH,
        full_vars_fixture(),
        ZervVarsFixture::with_version(0, 0, 0).with_epoch(1)
    )]
    #[case(
        crate::constants::bump_types::MAJOR,
        full_vars_fixture(),
        ZervVarsFixture::with_version(2, 0, 0).with_epoch(1)
    )]
    #[case(
        crate::constants::bump_types::MINOR,
        full_vars_fixture(),
        ZervVarsFixture::with_version(2, 3, 0).with_epoch(1)
    )]
    #[case(
        crate::constants::bump_types::PATCH,
        full_vars_fixture(),
        ZervVarsFixture::with_version(2, 3, 4).with_epoch(1)
    )]
    #[case(
        crate::constants::bump_types::PRE_RELEASE_LABEL,
        full_vars_fixture(),
        ZervVarsFixture::with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(0))
    )]
    #[case(
        crate::constants::bump_types::PRE_RELEASE_NUM,
        full_vars_fixture(),
        ZervVarsFixture::with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
    )]
    #[case(
        crate::constants::bump_types::POST,
        full_vars_fixture(),
        ZervVarsFixture::with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(5)
    )]
    #[case(
        crate::constants::bump_types::DEV,
        full_vars_fixture(),
        full_vars_fixture() // DEV has lowest precedence, so nothing gets reset
    )]
    fn test_reset_with_fixtures(
        #[case] component: &str,
        #[case] start_fixture: ZervVarsFixture,
        #[case] expected_fixture: ZervVarsFixture,
    ) {
        // Create starting state
        let mut vars: ZervVars = start_fixture.into();

        // Apply the reset
        vars.reset_lower_precedence_components(component).unwrap();

        // Create expected state
        let expected: ZervVars = expected_fixture.into();

        // Compare the entire objects - much cleaner!
        assert_eq!(
            vars, expected,
            "Reset result mismatch for component: {component}"
        );
    }

    // The following tests cover edge cases not covered by the main parametrized test

    #[test]
    fn test_reset_pre_release_num_only() {
        let mut vars = ZervVars {
            pre_release: Some(PreReleaseVar {
                label: PreReleaseLabel::Beta,
                number: Some(5),
            }),
            ..Default::default()
        };

        // Reset from pre-release label should reset the number part
        vars.reset_component(&BumpType::PreReleaseNum(0));

        assert_eq!(
            vars.pre_release,
            Some(PreReleaseVar {
                label: PreReleaseLabel::Beta,
                number: Some(0),
            })
        );
    }

    #[test]
    fn test_reset_pre_release_num_when_none() {
        let mut vars = ZervVars {
            pre_release: None,
            ..Default::default()
        };

        // Reset pre-release number when no pre-release exists should do nothing
        vars.reset_component(&BumpType::PreReleaseNum(0));

        assert_eq!(vars.pre_release, None);
    }
}
