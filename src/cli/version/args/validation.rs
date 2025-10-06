use super::{
    BumpsConfig,
    MainConfig,
    OverridesConfig,
};
use crate::error::ZervError;

/// Validation methods for argument combinations
pub struct Validation;

impl Validation {
    /// Validate main configuration
    pub fn validate_main(_main: &MainConfig) -> Result<(), ZervError> {
        // Main config validation (currently no conflicts to check)
        Ok(())
    }

    /// Validate overrides configuration
    pub fn validate_overrides(overrides: &OverridesConfig) -> Result<(), ZervError> {
        // Check for conflicting dirty flags
        if overrides.dirty && overrides.no_dirty {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --dirty with --no-dirty (conflicting options)".to_string(),
            ));
        }

        // Check for --clean conflicts
        if overrides.clean {
            if overrides.distance.is_some() {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --distance (conflicting options)".to_string(),
                ));
            }
            if overrides.dirty {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --dirty (conflicting options)".to_string(),
                ));
            }
            if overrides.no_dirty {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --no-dirty (conflicting options)".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate bumps configuration
    pub fn validate_bumps(bumps: &BumpsConfig) -> Result<(), ZervError> {
        // Check for conflicting context control flags
        if bumps.bump_context && bumps.no_bump_context {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --bump-context with --no-bump-context (conflicting options)"
                    .to_string(),
            ));
        }

        // Validate schema-based bump arguments
        Self::validate_schema_bump_args(bumps)?;

        Ok(())
    }

    /// Validate cross-module conflicts
    pub fn validate_cross_module(
        overrides: &OverridesConfig,
        bumps: &BumpsConfig,
    ) -> Result<(), ZervError> {
        // Check for conflicting context control and dirty flags
        if bumps.no_bump_context && overrides.dirty {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --no-bump-context with --dirty (conflicting options)".to_string(),
            ));
        }

        // Validate pre-release flags
        Self::validate_pre_release_flags(overrides, bumps)?;

        Ok(())
    }

    /// Resolve default context control behavior
    /// If neither --bump-context nor --no-bump-context is provided, default to --bump-context
    pub fn resolve_context_control_defaults(bumps: &mut BumpsConfig) -> Result<(), ZervError> {
        // Mathematical approach: handle all possible states
        match (bumps.bump_context, bumps.no_bump_context) {
            // Invalid case: both flags provided
            (true, true) => {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --bump-context with --no-bump-context (conflicting options)"
                        .to_string(),
                ));
            }
            // Default case: neither flag provided
            (false, false) => {
                bumps.bump_context = true;
            }
            // Any other case: explicit flags provided (keep as is)
            _ => {
                // No change needed - already correct
            }
        }

        Ok(())
    }

    /// Resolve default bump values
    /// If a bump option is provided without a value, set it to 1 (the default)
    pub fn resolve_bump_defaults(bumps: &mut BumpsConfig) -> Result<(), ZervError> {
        // Resolve bump_major: Some(None) -> Some(Some(1))
        if let Some(None) = bumps.bump_major {
            bumps.bump_major = Some(Some(1));
        }

        // Resolve bump_minor: Some(None) -> Some(Some(1))
        if let Some(None) = bumps.bump_minor {
            bumps.bump_minor = Some(Some(1));
        }

        // Resolve bump_patch: Some(None) -> Some(Some(1))
        if let Some(None) = bumps.bump_patch {
            bumps.bump_patch = Some(Some(1));
        }

        // Resolve bump_post: Some(None) -> Some(Some(1))
        if let Some(None) = bumps.bump_post {
            bumps.bump_post = Some(Some(1));
        }

        // Resolve bump_dev: Some(None) -> Some(Some(1))
        if let Some(None) = bumps.bump_dev {
            bumps.bump_dev = Some(Some(1));
        }

        // Resolve bump_pre_release_num: Some(None) -> Some(Some(1))
        if let Some(None) = bumps.bump_pre_release_num {
            bumps.bump_pre_release_num = Some(Some(1));
        }

        // Resolve bump_epoch: Some(None) -> Some(Some(1))
        if let Some(None) = bumps.bump_epoch {
            bumps.bump_epoch = Some(Some(1));
        }

        Ok(())
    }

    /// Validate pre-release flags for conflicts
    fn validate_pre_release_flags(
        overrides: &OverridesConfig,
        bumps: &BumpsConfig,
    ) -> Result<(), ZervError> {
        if overrides.pre_release_label.is_some() && bumps.bump_pre_release_label.is_some() {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --pre-release-label with --bump-pre-release-label".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate schema-based bump arguments
    fn validate_schema_bump_args(bumps: &BumpsConfig) -> Result<(), ZervError> {
        // Validate bump_core arguments (must be pairs of index, value)
        if !bumps.bump_core.len().is_multiple_of(2) {
            return Err(ZervError::InvalidArgument(
                "--bump-core requires pairs of index and value arguments".to_string(),
            ));
        }

        // Validate bump_extra_core arguments (must be pairs of index, value)
        if !bumps.bump_extra_core.len().is_multiple_of(2) {
            return Err(ZervError::InvalidArgument(
                "--bump-extra-core requires pairs of index and value arguments".to_string(),
            ));
        }

        // Validate bump_build arguments (must be pairs of index, value)
        if !bumps.bump_build.len().is_multiple_of(2) {
            return Err(ZervError::InvalidArgument(
                "--bump-build requires pairs of index and value arguments".to_string(),
            ));
        }

        Ok(())
    }
}
