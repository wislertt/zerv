use super::{
    BumpsConfig,
    OverridesConfig,
};
use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
};
use crate::cli::utils::template::Template;
use crate::error::ZervError;

/// Validation methods for argument combinations
pub struct Validation;

impl Validation {
    /// Validate main configuration (using shared validation)
    pub fn validate_main(_input: &InputConfig, _output: &OutputConfig) -> Result<(), ZervError> {
        // Validation is now handled by the shared Validation::validate_io function
        Ok(())
    }

    /// Validate overrides configuration
    pub fn validate_overrides(overrides: &OverridesConfig) -> Result<(), ZervError> {
        // Check for conflicting dirty flags
        if overrides.common.dirty && overrides.common.no_dirty {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --dirty with --no-dirty (conflicting options)".to_string(),
            ));
        }

        // Check for --clean conflicts
        if overrides.common.clean {
            if overrides.common.distance.is_some() {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --distance (conflicting options)".to_string(),
                ));
            }
            if overrides.common.dirty {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --dirty (conflicting options)".to_string(),
                ));
            }
            if overrides.common.no_dirty {
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
        if bumps.no_bump_context && overrides.common.dirty {
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
        // Resolve bump_major: Some(None) -> Some(Some(Template::new("1".to_string())))
        if let Some(None) = bumps.bump_major {
            bumps.bump_major = Some(Some(Template::new("1".to_string())));
        }

        // Resolve bump_minor: Some(None) -> Some(Some(Template::new("1".to_string())))
        if let Some(None) = bumps.bump_minor {
            bumps.bump_minor = Some(Some(Template::new("1".to_string())));
        }

        // Resolve bump_patch: Some(None) -> Some(Some(Template::new("1".to_string())))
        if let Some(None) = bumps.bump_patch {
            bumps.bump_patch = Some(Some(Template::new("1".to_string())));
        }

        // Resolve bump_post: Some(None) -> Some(Some(Template::new("1".to_string())))
        if let Some(None) = bumps.bump_post {
            bumps.bump_post = Some(Some(Template::new("1".to_string())));
        }

        // Resolve bump_dev: Some(None) -> Some(Some(Template::new("1".to_string())))
        if let Some(None) = bumps.bump_dev {
            bumps.bump_dev = Some(Some(Template::new("1".to_string())));
        }

        // Resolve bump_pre_release_num: Some(None) -> Some(Some(Template::new("1".to_string())))
        if let Some(None) = bumps.bump_pre_release_num {
            bumps.bump_pre_release_num = Some(Some(Template::new("1".to_string())));
        }

        // Resolve bump_epoch: Some(None) -> Some(Some(Template::new("1".to_string())))
        if let Some(None) = bumps.bump_epoch {
            bumps.bump_epoch = Some(Some(Template::new("1".to_string())));
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
        // Validate each section's arguments
        Self::validate_bump_section(&bumps.bump_core, "--bump-core")?;
        Self::validate_bump_section(&bumps.bump_extra_core, "--bump-extra-core")?;
        Self::validate_bump_section(&bumps.bump_build, "--bump-build")?;
        Ok(())
    }

    /// Validate a single bump section's arguments
    fn validate_bump_section(specs: &[Template<String>], arg_name: &str) -> Result<(), ZervError> {
        for template in specs {
            // For validation, we only check the string format, not template resolution
            let spec = template.as_str();
            if !Self::is_valid_bump_spec(spec) {
                return Err(ZervError::InvalidArgument(format!(
                    "{arg_name} argument '{spec}' must be in format 'index[=value]'"
                )));
            }
        }
        Ok(())
    }

    /// Check if a bump specification is valid
    fn is_valid_bump_spec(spec: &str) -> bool {
        if spec.is_empty() {
            return false;
        }

        if spec.contains('=') {
            // Format: index=value
            let parts: Vec<&str> = spec.split('=').collect();
            if parts.len() != 2 {
                return false; // Multiple equals signs
            }
            let index_part = parts[0];
            let value_part = parts[1];

            // Index must be valid (digits or negative number)
            if !Self::is_valid_index(index_part) {
                return false;
            }

            // Value can be anything (string or number)
            !value_part.is_empty()
        } else {
            // Format: index (defaults to value 1)
            Self::is_valid_index(spec)
        }
    }

    /// Check if an index string is valid (digits or negative number)
    fn is_valid_index(index: &str) -> bool {
        if index.is_empty() {
            return false;
        }

        // Must be all digits, or start with - followed by digits
        if let Some(digits) = index.strip_prefix('-') {
            // Negative index: -1, -2, etc.
            !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
        } else {
            // Positive index: 0, 1, 2, etc.
            index.chars().all(|c| c.is_ascii_digit())
        }
    }
}
