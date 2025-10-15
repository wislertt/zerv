use super::{
    BuildMetadata,
    PreReleaseIdentifier,
    SemVer,
};
use crate::error::ZervError;
use crate::version::zerv::core::PreReleaseLabel;
use crate::version::zerv::{
    Component,
    PreReleaseVar,
    Var,
    Zerv,
    ZervSchema,
    ZervVars,
};

struct PreReleaseProcessor<'a> {
    vars: &'a mut ZervVars,
    schema: &'a mut ZervSchema,
    pending_var: Option<Var>,
}

impl<'a> PreReleaseProcessor<'a> {
    fn new(vars: &'a mut ZervVars, schema: &'a mut ZervSchema) -> Self {
        Self {
            vars,
            schema,
            pending_var: None,
        }
    }

    fn is_var_set(&self, var: &Var) -> bool {
        match var {
            Var::PreRelease => self.vars.pre_release.is_some(),
            Var::Epoch => self.vars.epoch.is_some(),
            Var::Post => self.vars.post.is_some(),
            Var::Dev => self.vars.dev.is_some(),
            _ => false,
        }
    }

    fn set_var_value(&mut self, var: Var, value: Option<u64>) -> Result<(), ZervError> {
        match var {
            Var::Epoch => self.vars.epoch = value,
            Var::Post => self.vars.post = value,
            Var::Dev => self.vars.dev = value,
            Var::PreRelease => {
                if let Some(ref mut pr) = self.vars.pre_release {
                    pr.number = value;
                }
            }
            _ => {}
        }
        self.schema.push_extra_core(Component::Var(var))?;
        Ok(())
    }

    fn process_string(&mut self, s: &str) -> Result<(), ZervError> {
        if let Some(var) = Var::try_from_secondary_label(s) {
            if self.is_var_set(&var) {
                self.schema.push_extra_core(Component::Str(s.to_string()))?;
            } else if var == Var::PreRelease {
                if let Some(label) = PreReleaseLabel::try_from_str(s) {
                    self.vars.pre_release = Some(PreReleaseVar {
                        label,
                        number: None,
                    });
                    self.pending_var = Some(var);
                } else {
                    self.schema.push_extra_core(Component::Str(s.to_string()))?;
                }
            } else {
                self.pending_var = Some(var);
            }
        } else {
            self.schema.push_extra_core(Component::Str(s.to_string()))?;
        }
        Ok(())
    }

    fn process_uint(&mut self, n: u64) -> Result<(), ZervError> {
        self.schema.push_extra_core(Component::Int(n))
    }

    fn finalize(&mut self) -> Result<(), ZervError> {
        if let Some(var) = self.pending_var.take() {
            self.schema.push_extra_core(Component::Var(var))?;
        }
        Ok(())
    }
}

impl From<SemVer> for Zerv {
    fn from(semver: SemVer) -> Self {
        let schema = ZervSchema::semver_default().expect("SemVer default schema should be valid");
        semver
            .to_zerv_with_schema(&schema)
            .expect("SemVer default conversion should work")
    }
}

impl SemVer {
    /// Convert SemVer to Zerv format while preserving all semantic information for round-trip conversion.
    pub fn to_zerv_with_schema(&self, schema: &ZervSchema) -> Result<Zerv, ZervError> {
        if *schema != ZervSchema::semver_default()? {
            return Err(ZervError::NotImplemented(
                "Custom schemas not yet implemented for SemVer conversion".to_string(),
            ));
        }

        let mut vars = ZervVars {
            major: Some(self.major),
            minor: Some(self.minor),
            patch: Some(self.patch),
            ..Default::default()
        };

        let mut schema = schema.clone();
        let mut processor = PreReleaseProcessor::new(&mut vars, &mut schema);

        // Process pre-release identifiers
        if let Some(pre_release) = &self.pre_release {
            for identifier in pre_release {
                match identifier {
                    PreReleaseIdentifier::String(s) => {
                        // Special case: if we have a pending PreRelease var and encounter another string,
                        // finalize the PreRelease var and treat the new string as literal
                        if processor.pending_var == Some(Var::PreRelease) {
                            processor
                                .schema
                                .push_extra_core(Component::Var(Var::PreRelease))?;
                            processor.pending_var = None;
                            processor
                                .schema
                                .push_extra_core(Component::Str(s.to_string()))?;
                            continue;
                        }

                        // Check if this var type is already set or pending
                        if let Some(var) = Var::try_from_secondary_label(s) {
                            // If same var type is pending, finalize it and treat this as duplicate
                            if processor.pending_var.as_ref() == Some(&var) {
                                let pending_var = processor.pending_var.take().unwrap();
                                processor.set_var_value(pending_var, None)?;
                                processor
                                    .schema
                                    .push_extra_core(Component::Str(s.to_string()))?;
                                continue;
                            }
                            // If var is already set, finalize any pending var and treat as duplicate
                            if processor.is_var_set(&var) {
                                if let Some(pending_var) = processor.pending_var.take() {
                                    processor.set_var_value(pending_var, None)?;
                                }
                                processor
                                    .schema
                                    .push_extra_core(Component::Str(s.to_string()))?;
                                continue;
                            }
                        }

                        // Finalize any pending var before processing new var
                        if let Some(var) = processor.pending_var.take() {
                            processor.set_var_value(var, None)?;
                        }

                        // Process as new var
                        processor.process_string(s)?;
                    }
                    PreReleaseIdentifier::UInt(n) => {
                        if let Some(var) = processor.pending_var.take() {
                            processor.set_var_value(var, Some(*n))?;
                        } else {
                            processor.process_uint(*n)?;
                        }
                    }
                }
            }
        }

        processor.finalize()?;

        // Handle build metadata
        if let Some(build_metadata) = &self.build_metadata {
            for metadata in build_metadata {
                let component = match metadata {
                    BuildMetadata::String(s) => Component::Str(s.clone()),
                    BuildMetadata::UInt(n) => Component::Int(*n),
                };
                schema.push_build(component)?;
            }
        }

        Ok(Zerv { vars, schema })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::zerv::zerv_semver::to;
    use crate::version::zerv::Zerv;

    #[rstest]
    #[case("1.2.3", to::v1_2_3().build())]
    #[case("1.0.0-alpha.1", to::v1_0_0_a1().build())]
    #[case("1.0.0-something.1", to::v1_0_0_something_1().build())]
    #[case("1.0.0+build.123", to::v1_0_0_build().build())]
    #[case("1.0.0-alpha.1+build.123", to::v1_0_0_a1_build().build())]
    #[case(
        "1.0.0-alpha.1.lowercase.4.UPPERCASE.5+build.123",
        to::v1_0_0_a1_complex().build()
    )]
    #[case("1.0.0-foo.bar.beta.2.baz", to::v1_0_0_foo_bar_beta_2_baz().build())]
    #[case("1.0.0-alpha.1.beta.2", to::v1_0_0_alpha_1_beta_2().build())]
    #[case("1.0.0-rc.1.alpha.2.beta.3", to::v1_0_0_rc_1_alpha_2_beta_3().build())]
    #[case("1.0.0-pre.alpha.1", to::v1_0_0_pre_alpha_1().build())]
    #[case("1.0.0-test.alpha.beta.rc.1", to::v1_0_0_test_alpha_beta_rc_1().build())]
    #[case("1.0.0-ALPHA.1", to::v1_0_0_alpha_1().build())]
    #[case("1.0.0-epoch.1", to::v1_0_0_epoch_1().build())]
    #[case("1.0.0-post.1", to::v1_0_0_post_1().build())]
    #[case("1.0.0-dev.1", to::v1_0_0_dev_1().build())]
    // Case variations - Beta
    #[case("1.0.0-BETA.2", to::v1_0_0_beta_2().build())]
    // Case variations - RC
    #[case("1.0.0-RC.3", to::v1_0_0_rc_3().build())]
    // Case variations - Preview (treated as regular string)
    #[case("1.0.0-Preview.4", to::v1_0_0_preview_4().build())]
    // Case variations - short forms
    #[case("1.0.0-a.1", to::v1_0_0_a_1().build())]
    #[case("1.0.0-b.2", to::v1_0_0_b_2().build())]
    #[case("1.0.0-c.3", to::v1_0_0_c_3().build())]
    // Case variations - without numbers
    #[case("1.0.0-alpha", to::v1_0_0_alpha().build())]
    #[case("1.0.0-beta", to::v1_0_0_beta().build())]
    #[case("1.0.0-rc", to::v1_0_0_rc().build())]
    // Case variations - with zero
    #[case("1.0.0-alpha.0", to::v1_0_0_alpha_0().build())]
    #[case("1.0.0-beta.0", to::v1_0_0_beta_0().build())]
    // Case variations - with prefix (alpha/beta found later in sequence)
    #[case("1.0.0-foo.1.alpha", to::v1_0_0_foo_1_alpha().build())]
    #[case("1.0.0-bar.2.beta", to::v1_0_0_bar_2_beta().build())]
    // Epoch handling
    #[case("1.0.0-epoch.1", to::v1_0_0_epoch_1().build())]
    #[case("1.0.0-epoch.5", to::v1_0_0_epoch_5().build())]
    #[case("1.0.0-epoch.0", to::v1_0_0_epoch_0().build())]
    #[case("1.0.0-epoch.999", to::v1_0_0_epoch_999().build())]
    // Post handling
    #[case("1.0.0-post.1", to::v1_0_0_post_1().build())]
    #[case("1.0.0-post.5", to::v1_0_0_post_5().build())]
    #[case("1.0.0-post.0", to::v1_0_0_post_0().build())]
    // Dev handling
    #[case("1.0.0-dev.1", to::v1_0_0_dev_1().build())]
    #[case("1.0.0-dev.0", to::v1_0_0_dev_0().build())]
    #[case("1.0.0-dev.10", to::v1_0_0_dev_10().build())]
    // Complex combinations
    #[case("1.0.0-epoch.2.alpha.1", to::v1_0_0_epoch_2_alpha_1().build())]
    #[case("1.0.0-epoch.3.beta.2", to::v1_0_0_epoch_3_beta_2().build())]
    #[case("1.0.0-epoch.1.rc.5", to::v1_0_0_epoch_1_rc_5().build())]
    #[case("1.0.0-epoch.4.alpha", to::v1_0_0_epoch_4_alpha().build())]
    #[case("1.0.0-post.1.dev.2", to::v1_0_0_post_1_dev_2().build())]
    #[case("1.0.0-dev.3.post.4", to::v1_0_0_dev_3_post_4().build())]
    #[case("1.0.0-alpha.1.post.2", to::v1_0_0_alpha_1_post_2().build())]
    #[case("1.0.0-beta.3.post.1", to::v1_0_0_beta_3_post_1().build())]
    #[case("1.0.0-rc.2.post.5", to::v1_0_0_rc_2_post_5().build())]
    #[case("1.0.0-alpha.1.dev.2", to::v1_0_0_alpha_1_dev_2().build())]
    #[case("1.0.0-beta.2.dev.1", to::v1_0_0_beta_2_dev_1().build())]
    #[case("1.0.0-rc.1.dev.3", to::v1_0_0_rc_1_dev_3().build())]
    #[case("1.0.0-alpha.1.post.2.dev.3", to::v1_0_0_alpha_1_post_2_dev_3().build())]
    #[case("1.0.0-beta.2.dev.1.post.3", to::v1_0_0_beta_2_dev_1_post_3().build())]
    #[case("1.0.0-rc.1.post.1.dev.1", to::v1_0_0_rc_1_post_1_dev_1().build())]
    #[case("1.0.0-epoch.2.post.1.dev.3", to::v1_0_0_epoch_2_post_1_dev_3().build())]
    #[case("1.0.0-epoch.1.dev.2.post.1", to::v1_0_0_epoch_1_dev_2_post_1().build())]
    #[case("1.0.0-epoch.3.alpha.1.post.2.dev.1", to::v1_0_0_epoch_3_alpha_1_post_2_dev_1().build())]
    #[case("1.0.0-epoch.1.beta.2.dev.3.post.1", to::v1_0_0_epoch_1_beta_2_dev_3_post_1().build())]
    // Build metadata combinations
    #[case("1.0.0-epoch.1+build.123", to::v1_0_0_epoch_1_build().build())]
    #[case("1.0.0-post.1+build.456", to::v1_0_0_post_1_build().build())]
    #[case("1.0.0-dev.2+build.789", to::v1_0_0_dev_2_build().build())]
    #[case("1.0.0-epoch.2.alpha.1+build.abc", to::v1_0_0_epoch_2_alpha_1_build().build())]
    // Mixed order cases
    #[case("1.0.0-foo.epoch.1.alpha.2", to::v1_0_0_foo_epoch_1_alpha_2().build())]
    #[case("1.0.0-epoch.1.foo.post.2", to::v1_0_0_epoch_1_foo_post_2().build())]
    #[case("1.0.0-bar.dev.1.epoch.2", to::v1_0_0_bar_dev_1_epoch_2().build())]
    // Complex duplicate
    #[case("1.0.0-epoch.1.epoch.2.post.3.post.4.dev.5.dev.6.alpha.7.alpha.8", to::v1_0_0_duplicate_vars().build())]
    #[case("1.0.0-epoch.epoch.rc.rc.post.post.dev.dev", to::v1_0_0_duplicate_vars_without_num().build())]
    #[case("1.2.3-10.a.rc.epoch.rc.3", to::v1_2_3_complex_duplicate().build())]
    fn test_semver_to_zerv_conversion(#[case] semver_str: &str, #[case] expected: Zerv) {
        let semver: SemVer = semver_str.parse().unwrap();
        let zerv: Zerv = semver.into();
        assert_eq!(zerv, expected);
    }

    #[rstest]
    #[case("1.0.0")]
    #[case("2.1.0-beta.1")]
    #[case("1.0.0+build.123")]
    #[case("2.1.0-beta.1+build.123")]
    #[case("1.0.0-alpha.1.post.2.dev.3")]
    fn test_round_trip_conversion(#[case] version_str: &str) {
        let original: SemVer = version_str.parse().unwrap();
        let zerv: Zerv = original.clone().into();
        let converted: SemVer = zerv.into();

        assert_eq!(original.to_string(), converted.to_string());
    }
}
