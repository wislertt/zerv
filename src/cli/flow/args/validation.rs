use std::str::FromStr;

use super::FlowArgs;
use crate::cli::common::args::Validation as CommonValidation;
use crate::error::ZervError;
use crate::schema::ZervSchemaPreset;
use crate::utils::constants::pre_release_labels::ALPHA;
use crate::version::zerv::core::Zerv;

impl FlowArgs {
    pub fn validate(&mut self, current_zerv: &Zerv) -> Result<(), ZervError> {
        // Use shared validation for input/output
        CommonValidation::validate_io(&self.input, &self.output)?;

        // Apply branch rules first to set proper defaults based on branch patterns
        self.branch_config.apply_branch_rules(current_zerv)?;

        // Validate and set defaults only for values not already set by branch rules
        self.validate_pre_release_label()?;
        self.validate_pre_release_num()?;
        self.validate_hash_branch_len()?;
        self.validate_post_mode()?;
        self.validate_schema()?;

        Ok(())
    }

    fn validate_pre_release_label(&mut self) -> Result<(), ZervError> {
        if self.branch_config.pre_release_label.is_none() {
            self.branch_config.pre_release_label = Some(ALPHA.to_string());
        }
        Ok(())
    }

    fn validate_pre_release_num(&self) -> Result<(), ZervError> {
        // No longer need to check for no_pre_release conflicts
        Ok(())
    }

    fn validate_hash_branch_len(&self) -> Result<(), ZervError> {
        if self.hash_branch_len == 0 || self.hash_branch_len > 10 {
            return Err(ZervError::InvalidArgument(format!(
                "hash-branch-len must be between 1 and 10, got {}",
                self.hash_branch_len
            )));
        }
        Ok(())
    }

    fn validate_post_mode(&self) -> Result<(), ZervError> {
        use crate::utils::constants::post_modes;

        if let Some(post_mode) = &self.branch_config.post_mode
            && !post_modes::VALID_MODES.contains(&post_mode.as_str())
        {
            return Err(ZervError::InvalidArgument(format!(
                "post-mode must be one of: {}, got {}",
                post_modes::VALID_MODES.join(", "),
                post_mode
            )));
        }
        Ok(())
    }

    fn validate_schema(&self) -> Result<(), ZervError> {
        if let Some(schema_name) = &self.schema {
            // First, validate it's a known schema
            let _parsed = ZervSchemaPreset::from_str(schema_name).map_err(|_| {
                ZervError::InvalidArgument(format!("Unknown schema variant: '{}'", schema_name))
            })?;

            // Restrict to standard schemas only (check prefix)
            if schema_name.starts_with("standard") {
                Ok(())
            } else {
                Err(ZervError::InvalidArgument(format!(
                    "zerv flow only supports standard schema variants, got: '{}'",
                    schema_name
                )))
            }
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::cli::flow::args::branch_rules::BranchRulesConfig;
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::zerv::core::Zerv;

    /// Helper function to create a mock zerv object for tests
    fn mock_zerv() -> Zerv {
        let mut zerv = ZervFixture::new().build();
        // Set a mock branch name for tests that need branch detection
        zerv.vars.last_branch = Some("main".to_string());
        zerv
    }

    mod validation {
        use super::*;

        #[rstest]
        #[case("alpha")]
        #[case("beta")]
        #[case("rc")]
        fn test_valid_pre_release_labels(#[case] label: &str) {
            let mut args = FlowArgs::default();
            args.branch_config.pre_release_label = Some(label.to_string());
            assert!(args.validate(&mock_zerv()).is_ok());
        }

        #[rstest]
        #[case(1)]
        #[case(5)]
        #[case(10)]
        fn test_valid_hash_branch_lengths(#[case] length: u32) {
            let mut args = FlowArgs {
                hash_branch_len: length,
                ..FlowArgs::default()
            };
            assert!(args.validate(&mock_zerv()).is_ok());
        }

        #[rstest]
        #[case("commit")]
        #[case("tag")]
        fn test_valid_post_modes(#[case] mode: &str) {
            let mut args = FlowArgs::default();
            args.branch_config.post_mode = Some(mode.to_string());
            assert!(args.validate(&mock_zerv()).is_ok());
        }

        #[rstest]
        #[case(0)]
        #[case(11)]
        #[case(20)]
        fn test_invalid_hash_branch_lengths(#[case] length: u32) {
            let mut args = FlowArgs {
                hash_branch_len: length,
                ..FlowArgs::default()
            };
            let result = args.validate(&mock_zerv());
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("hash-branch-len must be between 1 and 10")
            );
        }

        #[rstest]
        #[case("invalid")]
        #[case("commit-invalid")]
        #[case("tag-invalid")]
        #[case("")]
        fn test_invalid_post_modes(#[case] mode: &str) {
            let mut args = FlowArgs::default();
            args.branch_config.post_mode = Some(mode.to_string());
            let result = args.validate(&mock_zerv());
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("post-mode must be one of:")
            );
        }

        #[rstest]
        #[case(Some("beta".to_string()), Some(5))]
        #[case(None, None)]
        #[case(Some("rc".to_string()), None)]
        #[case(None, Some(10))]
        fn test_valid_combinations(#[case] label: Option<String>, #[case] num: Option<u32>) {
            let mut args = FlowArgs {
                branch_config: BranchRulesConfig {
                    pre_release_label: label,
                    pre_release_num: num,
                    ..Default::default()
                },
                ..FlowArgs::default()
            };
            assert!(args.validate(&mock_zerv()).is_ok());
        }
    }

    mod schema_validation {
        use super::*;

        #[rstest]
        #[case("standard")]
        #[case("standard-no-context")]
        #[case("standard-context")]
        #[case("standard-base")]
        #[case("standard-base-prerelease")]
        #[case("standard-base-prerelease-post")]
        #[case("standard-base-prerelease-post-dev")]
        #[case("standard-base-context")]
        #[case("standard-base-prerelease-context")]
        #[case("standard-base-prerelease-post-context")]
        #[case("standard-base-prerelease-post-dev-context")]
        fn test_valid_standard_schemas(#[case] schema: &str) {
            let mut args = FlowArgs {
                schema: Some(schema.to_string()),
                ..FlowArgs::default()
            };
            assert!(args.validate(&mock_zerv()).is_ok());
        }

        #[rstest]
        #[case("calver")]
        #[case("calver-base")]
        #[case("calver-context")]
        #[case("calver-no-context")]
        #[case("calver-base-prerelease")]
        #[case("invalid-schema")]
        #[case("unknown")]
        #[case("")]
        fn test_invalid_schemas(#[case] schema: &str) {
            let mut args = FlowArgs {
                schema: Some(schema.to_string()),
                ..FlowArgs::default()
            };
            let result = args.validate(&mock_zerv());
            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();

            // Both error types are valid - unknown schemas or non-standard schemas
            assert!(
                error_msg.contains("Unknown schema variant")
                    || error_msg.contains("zerv flow only supports standard schema variants")
            );
        }

        #[test]
        fn test_no_schema_defaults_to_none() {
            let args = FlowArgs::default();
            assert!(args.schema.is_none());

            let mut args = args;
            assert!(args.validate(&mock_zerv()).is_ok());
            assert!(args.schema.is_none()); // Should remain None
        }

        #[test]
        fn test_schema_validation_with_pre_release_overrides() {
            let mut args = FlowArgs {
                schema: Some("standard-base".to_string()),
                branch_config: BranchRulesConfig {
                    pre_release_label: Some("beta".to_string()),
                    pre_release_num: Some(42),
                    ..Default::default()
                },
                ..FlowArgs::default()
            };

            // Should validate successfully - schema and manual overrides can coexist
            assert!(args.validate(&mock_zerv()).is_ok());
        }
    }
}
