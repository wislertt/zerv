use crate::constants::bump_types;
use crate::error::ZervError;
use crate::version::zerv::bump::types::BumpType;
use crate::version::zerv::vars::ZervVars;

impl ZervVars {
    /// Reset all components with lower precedence than the given component
    pub fn reset_lower_precedence_components(&mut self, component: &str) -> Result<(), ZervError> {
        let current_precedence = BumpType::precedence_from_str(component);

        // Loop through all bump types in precedence order and reset those with lower precedence
        for (index, &name) in BumpType::PRECEDENCE_NAMES.iter().enumerate() {
            if index > current_precedence {
                self.reset_component_by_name(name);
            }
        }

        Ok(())
    }

    /// Reset a specific component based on its name
    fn reset_component_by_name(&mut self, component_name: &str) {
        match component_name {
            bump_types::EPOCH => self.epoch = Some(0),
            bump_types::MAJOR => self.major = Some(0),
            bump_types::MINOR => self.minor = Some(0),
            bump_types::PATCH => self.patch = Some(0),
            bump_types::PRE_RELEASE_LABEL => self.pre_release = None,
            bump_types::PRE_RELEASE_NUM => {
                if let Some(ref mut pre_release) = self.pre_release {
                    pre_release.number = Some(0);
                }
            }
            bump_types::POST => self.post = None,
            bump_types::DEV => self.dev = None,
            _ => panic!("Unknown component name for reset: {component_name}"),
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
        ZervVarsFixture::new()
            .with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(5)
            .with_dev(6)
    }

    #[rstest]
    #[case(
        bump_types::EPOCH,
        full_vars_fixture(),
        ZervVarsFixture::new().with_version(0, 0, 0).with_epoch(1)
    )]
    #[case(
        bump_types::MAJOR,
        full_vars_fixture(),
        ZervVarsFixture::new().with_version(2, 0, 0).with_epoch(1)
    )]
    #[case(
        bump_types::MINOR,
        full_vars_fixture(),
        ZervVarsFixture::new().with_version(2, 3, 0).with_epoch(1)
    )]
    #[case(
        bump_types::PATCH,
        full_vars_fixture(),
        ZervVarsFixture::new().with_version(2, 3, 4).with_epoch(1)
    )]
    #[case(
        bump_types::PRE_RELEASE_LABEL,
        full_vars_fixture(),
        ZervVarsFixture::new()
            .with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(0))
    )]
    #[case(
        bump_types::PRE_RELEASE_NUM,
        full_vars_fixture(),
        ZervVarsFixture::new()
            .with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
    )]
    #[case(
        bump_types::POST,
        full_vars_fixture(),
        ZervVarsFixture::new()
            .with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(5)
    )]
    #[case(
        bump_types::DEV,
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

        // Reset from pre-release number should reset the number part
        vars.reset_component_by_name(bump_types::PRE_RELEASE_NUM);

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
        vars.reset_component_by_name(bump_types::PRE_RELEASE_NUM);

        assert_eq!(vars.pre_release, None);
    }

    #[test]
    #[should_panic(expected = "Unknown component name for reset: unknown")]
    fn test_reset_component_by_name_invalid() {
        let mut vars = ZervVars::default();
        vars.reset_component_by_name("unknown");
    }
}
