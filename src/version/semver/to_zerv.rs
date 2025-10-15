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
    ///
    /// Goals:
    /// - Map SemVer components (major.minor.patch, pre-release, build) to Zerv vars and schema
    /// - Detect secondary labels (alpha, beta, rc, epoch, post, dev) and map to appropriate Zerv vars
    /// - Ensure round-trip fidelity: SemVer → Zerv → SemVer produces semantically equivalent version
    /// - Populate schema only for complex cases, keep simple pre-release in vars only
    pub fn to_zerv_with_schema(&self, schema: &ZervSchema) -> Result<Zerv, ZervError> {
        // Only support default SemVer schema for now
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

        // Handle pre-release - process each identifier for secondary labels
        let mut schema = schema.clone();
        let mut current_pre_release_var: Option<Var> = None;

        if let Some(pre_release) = &self.pre_release {
            // Debug for case 33
            if self.major == 1 && self.minor == 2 && self.patch == 3 {
                println!("Processing pre_release: {pre_release:?}");
            }

            for identifier in pre_release {
                // Debug for case 33
                if self.major == 1 && self.minor == 2 && self.patch == 3 {
                    println!(
                        "Processing identifier: {identifier:?}, current_pre_release_var: {current_pre_release_var:?}"
                    );
                }
                // Handle pending PreRelease var at the start of each iteration
                if let Some(pending_var) = &current_pre_release_var
                    && *pending_var == Var::PreRelease
                    && let PreReleaseIdentifier::String(_) = identifier
                {
                    // Add pending PreRelease var to schema when encountering a string
                    schema.set_extra_core({
                        let mut current = schema.extra_core().clone();
                        current.push(Component::Var(pending_var.clone()));
                        current
                    })?;
                    current_pre_release_var = None;
                }

                // Handle pending var first
                if let Some(var) = current_pre_release_var {
                    // Check if current identifier is a duplicate secondary label
                    if let PreReleaseIdentifier::String(s) = identifier
                        && let Some(current_var) = Var::try_from_secondary_label(s)
                    {
                        if current_var == var {
                            // This is a duplicate of the pending var, add pending var to schema first
                            schema.set_extra_core({
                                let mut current = schema.extra_core().clone();
                                current.push(Component::Var(var.clone()));
                                current
                            })?;
                            current_pre_release_var = None;

                            // Then add the duplicate as a string
                            schema.set_extra_core({
                                let mut current = schema.extra_core().clone();
                                current.push(Component::Str(s.clone()));
                                current
                            })?;
                            continue;
                        } else {
                            // Check if this is a duplicate of a different already-set var
                            let already_set = match current_var {
                                Var::PreRelease => vars.pre_release.is_some(),
                                Var::Epoch => vars.epoch.is_some(),
                                Var::Post => vars.post.is_some(),
                                Var::Dev => vars.dev.is_some(),
                                _ => false,
                            };

                            if already_set {
                                // Add pending var to schema first
                                schema.set_extra_core({
                                    let mut current = schema.extra_core().clone();
                                    current.push(Component::Var(var.clone()));
                                    current
                                })?;
                                current_pre_release_var = None;

                                // Then add the duplicate as a string
                                schema.set_extra_core({
                                    let mut current = schema.extra_core().clone();
                                    current.push(Component::Str(s.clone()));
                                    current
                                })?;
                                continue;
                            }
                        }
                    }

                    let value = match identifier {
                        PreReleaseIdentifier::UInt(n) => Some(*n),
                        _ => None,
                    };

                    // Update vars according to current_var
                    match var {
                        Var::Epoch => vars.epoch = value,
                        Var::Post => vars.post = value,
                        Var::Dev => vars.dev = value,
                        Var::PreRelease => {
                            if let Some(ref mut pr) = vars.pre_release {
                                pr.number = value;
                            } else {
                                unreachable!(
                                    "pre_release should exist when current_pre_release_var is Var::PreRelease"
                                );
                            }
                        }
                        _ => {}
                    }

                    // Add var to schema for round-trip conversion
                    schema.set_extra_core({
                        let mut current = schema.extra_core().clone();
                        current.push(Component::Var(var.clone()));
                        current
                    })?;
                    current_pre_release_var = None;
                    continue;
                }

                match identifier {
                    PreReleaseIdentifier::String(s) => {
                        if let Some(var) = Var::try_from_secondary_label(s) {
                            // Check if this var type is already set (first wins logic)
                            let already_set = match var {
                                Var::PreRelease => {
                                    vars.pre_release.is_some()
                                        || current_pre_release_var == Some(Var::PreRelease)
                                }
                                Var::Epoch => {
                                    vars.epoch.is_some()
                                        || current_pre_release_var == Some(Var::Epoch)
                                }
                                Var::Post => {
                                    vars.post.is_some()
                                        || current_pre_release_var == Some(Var::Post)
                                }
                                Var::Dev => {
                                    vars.dev.is_some() || current_pre_release_var == Some(Var::Dev)
                                }
                                _ => false,
                            };

                            // Debug for case 33
                            if s == "epoch" {
                                println!(
                                    "Processing epoch: var={:?}, already_set={}, vars.epoch={:?}, current_pre_release_var={:?}",
                                    var, already_set, vars.epoch, current_pre_release_var
                                );
                            }

                            // Debug for case 33
                            if s == "epoch" {
                                println!(
                                    "Branch decision: already_set={}, taking {} branch",
                                    already_set,
                                    if already_set { "duplicate" } else { "first" }
                                );
                            }

                            if !already_set {
                                if var == Var::PreRelease {
                                    // Set pre-release label only if not already set (first wins)
                                    if let Some(label) = PreReleaseLabel::try_from_str(s) {
                                        vars.pre_release = Some(PreReleaseVar {
                                            label,
                                            number: None,
                                        });
                                        // Set current_pre_release_var for first pre-release
                                        current_pre_release_var = Some(var);
                                    } else {
                                        // Not a recognized pre-release label, treat as string
                                        schema.set_extra_core({
                                            let mut current = schema.extra_core().clone();
                                            current.push(Component::Str(s.clone()));
                                            current
                                        })?;
                                    }
                                } else {
                                    // Set current_pre_release_var for other vars (Epoch, Post, Dev)
                                    current_pre_release_var = Some(var);
                                }
                            } else {
                                // Second occurrence of same var type: push as string to extra_core
                                schema.set_extra_core({
                                    let mut current = schema.extra_core().clone();
                                    current.push(Component::Str(s.clone()));
                                    current
                                })?;
                            }
                        } else {
                            schema.set_extra_core({
                                let mut current = schema.extra_core().clone();
                                current.push(Component::Str(s.clone()));
                                current
                            })?;
                        }
                    }
                    PreReleaseIdentifier::UInt(n) => {
                        schema.set_extra_core({
                            let mut current = schema.extra_core().clone();
                            current.push(Component::Int(*n));
                            current
                        })?;
                    }
                }
            }
        }

        // Handle build metadata - add to schema build
        if let Some(build_metadata) = &self.build_metadata {
            for metadata in build_metadata {
                let component = match metadata {
                    BuildMetadata::String(s) => Component::Str(s.clone()),
                    BuildMetadata::UInt(n) => Component::Int(*n),
                };
                schema.set_build({
                    let mut current = schema.build().clone();
                    current.push(component);
                    current
                })?;
            }
        }

        // Handle any remaining current_pre_release_var
        if let Some(var) = current_pre_release_var {
            // Add var to schema
            schema.set_extra_core({
                let mut current = schema.extra_core().clone();
                current.push(Component::Var(var));
                current
            })?;
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
