use super::{
    FlowSpecificConfig,
    OverridesConfig,
};
use crate::error::ZervError;
use crate::utils::bool_resolution::BoolResolution;

/// Validation methods for flow argument combinations
pub struct Validation;

impl Validation {
    /// Validate flow-specific configuration
    pub fn validate_flow_specific(flow_specific: &FlowSpecificConfig) -> Result<(), ZervError> {
        // Check for conflicting output mode flags
        if flow_specific.with_pre_release && flow_specific.base_only {
            return Err(ZervError::ConflictingOptions(
                "Cannot specify both --with-pre-release and --base-only".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate overrides configuration
    pub fn validate_overrides(overrides: &OverridesConfig) -> Result<(), ZervError> {
        // Check for conflicting dirty flags using the standard utility
        BoolResolution::validate_opposing_flags(overrides.dirty, overrides.no_dirty, "dirty")?;

        // Check for --clean conflicts
        if overrides.clean {
            if overrides.distance.is_some() {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --distance".to_string(),
                ));
            }
            if overrides.dirty {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --dirty".to_string(),
                ));
            }
            if overrides.no_dirty {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --no-dirty".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate RON format for branch rules
    pub fn validate_branch_rules(branch_rules: &Option<String>) -> Result<(), ZervError> {
        if let Some(rules) = branch_rules
            && let Err(e) = ron::from_str::<Vec<()>>(&format!("[{}]", rules))
        {
            return Err(ZervError::InvalidArgument(format!(
                "Invalid RON format for branch rules: {}",
                e
            )));
        }

        Ok(())
    }
}
