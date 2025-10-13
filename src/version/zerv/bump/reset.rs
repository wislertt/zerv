use crate::error::ZervError;
use crate::version::zerv::bump::precedence::Precedence;
use crate::version::zerv::core::Zerv;

impl Zerv {
    /// Reset all components with lower precedence than the given precedence
    pub fn reset_lower_precedence_components(
        &mut self,
        precedence: &Precedence,
    ) -> Result<(), ZervError> {
        let current_precedence_index = self
            .schema
            .precedence_order()
            .get_index(precedence)
            .ok_or_else(|| {
                ZervError::InvalidBumpTarget(format!("Unknown precedence: {precedence:?}"))
            })?;

        // Use a for loop over the precedence order
        for (index, precedence_item) in self.schema.precedence_order().iter().enumerate() {
            if index > current_precedence_index {
                // Map precedence to field reset
                match precedence_item {
                    Precedence::Epoch => {
                        self.vars.epoch = Some(0);
                    }
                    Precedence::Major => {
                        self.vars.major = Some(0);
                    }
                    Precedence::Minor => {
                        self.vars.minor = Some(0);
                    }
                    Precedence::Patch => {
                        self.vars.patch = Some(0);
                    }
                    Precedence::PreReleaseLabel => {
                        self.vars.pre_release = None;
                    }
                    Precedence::PreReleaseNum => {
                        if let Some(ref mut pre_release) = self.vars.pre_release {
                            pre_release.number = Some(0);
                        }
                    }
                    Precedence::Post => {
                        self.vars.post = None;
                    }
                    Precedence::Dev => {
                        self.vars.dev = None;
                    }
                    // Skip schema-based precedences for now
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use crate::test_utils::zerv::{
        ZervFixture,
        ZervVarsFixture,
    };
    use crate::version::zerv::bump::precedence::Precedence;
    use crate::version::zerv::core::PreReleaseLabel;
    use crate::version::zerv::vars::ZervVars;

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
        Precedence::Epoch,
        full_vars_fixture(),
        ZervVarsFixture::new().with_version(0, 0, 0).with_epoch(1)
    )]
    #[case(
        Precedence::Major,
        full_vars_fixture(),
        ZervVarsFixture::new().with_version(2, 0, 0).with_epoch(1)
    )]
    #[case(
        Precedence::Minor,
        full_vars_fixture(),
        ZervVarsFixture::new().with_version(2, 3, 0).with_epoch(1)
    )]
    #[case(
        Precedence::Patch,
        full_vars_fixture(),
        ZervVarsFixture::new().with_version(2, 3, 4).with_epoch(1)
    )]
    #[case(
        Precedence::PreReleaseLabel,
        full_vars_fixture(),
        ZervVarsFixture::new()
            .with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(0))
    )]
    #[case(
        Precedence::PreReleaseNum,
        full_vars_fixture(),
        ZervVarsFixture::new()
            .with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
    )]
    #[case(
        Precedence::Post,
        full_vars_fixture(),
        ZervVarsFixture::new()
            .with_version(2, 3, 4)
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(5)
    )]
    #[case(
        Precedence::Dev,
        full_vars_fixture(),
        full_vars_fixture() // DEV has lowest precedence, so nothing gets reset
    )]
    fn test_reset_with_fixtures(
        #[case] precedence: Precedence,
        #[case] start_fixture: ZervVarsFixture,
        #[case] expected_fixture: ZervVarsFixture,
    ) {
        // Create starting state
        let start_vars: ZervVars = start_fixture.into();
        let expected_vars: ZervVars = expected_fixture.into();

        // Create Zerv with the starting vars
        let mut zerv = ZervFixture::new()
            .with_version(
                start_vars.major.unwrap_or(0),
                start_vars.minor.unwrap_or(0),
                start_vars.patch.unwrap_or(0),
            )
            .with_epoch(start_vars.epoch.unwrap_or(0))
            .with_pre_release(
                start_vars
                    .pre_release
                    .as_ref()
                    .map(|pr| pr.label.clone())
                    .unwrap_or(PreReleaseLabel::Alpha),
                start_vars.pre_release.as_ref().and_then(|pr| pr.number),
            )
            .with_post(start_vars.post.unwrap_or(0))
            .with_dev(start_vars.dev.unwrap_or(0))
            .build();

        // Apply the reset
        zerv.reset_lower_precedence_components(&precedence).unwrap();

        // Compare the entire objects - much cleaner!
        assert_eq!(
            zerv.vars, expected_vars,
            "Reset result mismatch for precedence: {precedence:?}"
        );
    }
}
