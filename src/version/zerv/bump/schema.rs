use std::collections::HashSet;

use crate::constants::bump_types;
use crate::error::ZervError;
use crate::version::zerv::components::Component;
use crate::version::zerv::core::Zerv;

/// Type alias for process specifications: (index, override_value, bump_value)
type ProcessSpec = (usize, Option<String>, Option<String>);

impl Zerv {
    /// Parse a bump specification string into (index, value) pair
    /// Supports both "1=5" (explicit value) and "1" (default value) formats
    /// Supports negative indices for counting from end of schema
    /// OK:
    pub fn parse_bump_spec(spec: &str, schema_len: usize) -> Result<(usize, String), ZervError> {
        if let Some((index_str, value)) = spec.split_once('=') {
            // Explicit value: "1=5" -> (1, "5")
            let index = Self::parse_index(index_str, schema_len)?;
            let value = Self::parse_value(value)?;
            Ok((index, value))
        } else {
            // Default value: "1" -> (1, "1")
            let index = Self::parse_index(spec, schema_len)?;
            Ok((index, "1".to_string()))
        }
    }

    /// Parse an index string, supporting both positive and negative indices
    /// OK:
    fn parse_index(index_str: &str, schema_len: usize) -> Result<usize, ZervError> {
        let index: i32 = index_str
            .parse()
            .map_err(|_| ZervError::InvalidBumpTarget("Invalid index".to_string()))?;

        if index >= 0 {
            // Positive index: 0, 1, 2, ...
            let idx = index as usize;
            if idx >= schema_len {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Index {idx} out of bounds for schema of length {schema_len}"
                )));
            }
            Ok(idx)
        } else {
            // Negative index: -1, -2, -3, ... (count from end)
            let idx = schema_len as i32 + index;
            if idx < 0 || idx >= schema_len as i32 {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Negative index {index} out of bounds for schema of length {schema_len}"
                )));
            }
            Ok(idx as usize)
        }
    }

    /// Parse a value string, ensuring numeric values are positive
    /// Accepts both string values and positive numeric values
    /// OK:
    fn parse_value(value_str: &str) -> Result<String, ZervError> {
        // For numeric values, ensure they're positive
        if let Ok(num) = value_str.parse::<i32>()
            && num < 0
        {
            return Err(ZervError::InvalidBumpTarget(
                "Negative bump values not supported".to_string(),
            ));
        }

        Ok(value_str.to_string())
    }

    /// Parse an optional string value to Option<u32> with proper error handling
    /// Returns None if input is None, or Some(u32) if parsing succeeds
    /// Returns a descriptive error if parsing fails
    /// OK:
    fn parse_optional_u32(value: Option<&str>, field_name: &str) -> Result<Option<u32>, ZervError> {
        match value {
            Some(val) => {
                let parsed = val.parse::<u32>().map_err(|_| {
                    ZervError::InvalidBumpTarget(format!(
                        "Expected numeric value for {field_name} component, got: {val}"
                    ))
                })?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Process core schema components (both overrides and bumps)
    /// OK:
    pub fn process_schema_core(
        &mut self,
        core_overrides: &[String],
        core_bumps: &[String],
    ) -> Result<(), ZervError> {
        let specs = Self::parse_and_validate_process_specs(
            core_overrides,
            core_bumps,
            self.schema.core.len(),
        )?;

        // Process specs in order from lower index to higher index
        for (index, override_value, bump_value) in specs {
            self.process_schema_component("core", index, override_value, bump_value)?;
        }
        Ok(())
    }

    /// Process extra_core schema components (both overrides and bumps)
    /// Converts negative indices to positive, validates bounds, sorts by index, and checks for duplicates
    /// OK:
    pub fn process_schema_extra_core(
        &mut self,
        extra_core_overrides: &[String],
        extra_core_bumps: &[String],
    ) -> Result<(), ZervError> {
        let specs = Self::parse_and_validate_process_specs(
            extra_core_overrides,
            extra_core_bumps,
            self.schema.extra_core.len(),
        )?;

        // Process specs in order from lower index to higher index
        for (index, override_value, bump_value) in specs {
            self.process_schema_component("extra_core", index, override_value, bump_value)?;
        }
        Ok(())
    }

    /// Process build schema components (both overrides and bumps)
    /// OK:
    pub fn process_schema_build(
        &mut self,
        build_overrides: &[String],
        build_bumps: &[String],
    ) -> Result<(), ZervError> {
        let specs = Self::parse_and_validate_process_specs(
            build_overrides,
            build_bumps,
            self.schema.build.len(),
        )?;

        // Process specs in order from lower index to higher index
        for (index, override_value, bump_value) in specs {
            self.process_schema_component("build", index, override_value, bump_value)?;
        }
        Ok(())
    }

    /// Parse and validate process specifications with sorting and duplicate detection
    /// Returns sorted specs (lower index to higher index) with validation
    /// Each spec contains (index, override_value, bump_value)
    /// OK:
    pub fn parse_and_validate_process_specs(
        overrides: &[String],
        bumps: &[String],
        schema_len: usize,
    ) -> Result<Vec<ProcessSpec>, ZervError> {
        let mut parsed_specs = Vec::new();
        let mut seen_override_indices = HashSet::new();
        let mut seen_bump_indices = HashSet::new();

        // Parse override specs
        for spec in overrides {
            let (index, value) = Self::parse_override_spec(spec, schema_len)?;

            // Check for duplicate indices within overrides
            if !seen_override_indices.insert(index) {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Duplicate index {index} found in override specifications"
                )));
            }

            parsed_specs.push((index, Some(value), None));
        }

        // Parse bump specs
        for spec in bumps {
            let (index, value) = Self::parse_bump_spec(spec, schema_len)?;

            // Check for duplicate indices within bumps
            if !seen_bump_indices.insert(index) {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Duplicate index {index} found in bump specifications"
                )));
            }

            // Check if we already have an override for this index
            if let Some(existing) = parsed_specs.iter_mut().find(|(idx, _, _)| *idx == index) {
                // We already have an override for this index, add the bump value
                existing.2 = Some(value);
            } else {
                parsed_specs.push((index, None, Some(value)));
            }
        }

        // Sort specs by index (lower to higher)
        parsed_specs.sort_by_key(|(index, _, _)| *index);

        Ok(parsed_specs)
    }

    /// Parse an override specification string into (index, value) pair
    /// Override specs require explicit values: "1=5" (no default values)
    /// Supports negative indices for counting from end of schema
    /// OK:
    pub fn parse_override_spec(
        spec: &str,
        schema_len: usize,
    ) -> Result<(usize, String), ZervError> {
        if let Some((index_str, value)) = spec.split_once('=') {
            // Explicit value: "1=5" -> (1, "5")
            let index = Self::parse_index(index_str, schema_len)?;
            let value = Self::parse_value(value)?;
            Ok((index, value))
        } else {
            // Override specs require explicit values
            Err(ZervError::InvalidBumpTarget(format!(
                "Override specification '{spec}' requires explicit value (use index=value format)"
            )))
        }
    }

    /// Process a schema component by section, index, override value, and bump value
    /// Handles both overrides (absolute values) and bumps (relative values)
    pub fn process_schema_component(
        &mut self,
        section: &str,
        index: usize,
        override_value: Option<String>,
        bump_value: Option<String>,
    ) -> Result<(), ZervError> {
        // Get mutable reference to components at the beginning
        let components = match section {
            "core" => &mut self.schema.core,
            "extra_core" => &mut self.schema.extra_core,
            "build" => &mut self.schema.build,
            _ => {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Unknown schema section: {section}"
                )));
            }
        };

        let component = components.get(index).ok_or_else(|| {
            ZervError::InvalidBumpTarget(format!(
                "Index {index} out of bounds for {section} section"
            ))
        })?;

        match component {
            Component::VarField(field_name) => {
                let field_name_str = field_name.as_str().to_string();

                // Validate field can be processed
                if !self
                    .schema
                    .precedence_order
                    .field_precedence_names()
                    .contains(&field_name_str.as_str())
                {
                    return Err(ZervError::InvalidBumpTarget(format!(
                        "Cannot process custom field: {field_name_str}"
                    )));
                }

                // Parse override and bump values
                let override_val = Self::parse_optional_u32(override_value.as_deref(), "VarField")?;
                let bump_val = Self::parse_optional_u32(bump_value.as_deref(), "VarField")?;

                // Process field with both override and bump values
                match field_name_str.as_str() {
                    bump_types::MAJOR => self.process_major(override_val, bump_val)?,
                    bump_types::MINOR => self.process_minor(override_val, bump_val)?,
                    bump_types::PATCH => self.process_patch(override_val, bump_val)?,
                    bump_types::EPOCH => self.process_epoch(override_val, bump_val)?,
                    bump_types::POST => self.process_post(override_val, bump_val)?,
                    bump_types::DEV => self.process_dev(override_val, bump_val)?,
                    bump_types::PRE_RELEASE_NUM => {
                        self.process_pre_release_num(override_val, bump_val)?
                    }
                    bump_types::PRE_RELEASE_LABEL => {
                        return Err(ZervError::InvalidBumpTarget(
                            "Cannot process pre_release_label component - use process_pre_release_label method instead".to_string(),
                        ));
                    }
                    _ => {
                        return Err(ZervError::InvalidBumpTarget(format!(
                            "Unknown field: {field_name_str}"
                        )));
                    }
                }
            }
            Component::String(_) => {
                return Err(ZervError::NotImplemented(
                    "String component processing not yet implemented".to_string(),
                ));
            }
            Component::Integer(current_value) => {
                // Parse override and bump values for Integer components
                let override_val = Self::parse_optional_u32(override_value.as_deref(), "Integer")?;
                let bump_val = Self::parse_optional_u32(bump_value.as_deref(), "Integer")?;

                // Calculate new value: override first, then bump from that base
                let base_value = if let Some(override_val) = override_val {
                    // Override: set absolute value as base
                    override_val as u64
                } else {
                    // No override: use current value as base
                    *current_value
                };

                let new_value = if let Some(bump_val) = bump_val {
                    // Bump: add to base value (either override or current)
                    base_value + bump_val as u64
                } else {
                    // No bump: use base value as-is
                    base_value
                };

                // Update the component's stored value in the schema
                if let Some(Component::Integer(value)) = components.get_mut(index) {
                    *value = new_value;
                } else {
                    return Err(ZervError::InvalidBumpTarget(format!(
                        "Component at index {index} is not an Integer component"
                    )));
                }
            }
            Component::VarTimestamp(_) => {
                return Err(ZervError::InvalidBumpTarget(
                    "Cannot process timestamp component - timestamps are generated dynamically"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::ron_fields;
    use crate::test_utils::ZervFixture;
    use crate::version::zerv::components::Component;

    #[test]
    fn test_process_schema_core() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![
                Component::VarField(ron_fields::MAJOR.to_string()),
                Component::String(".".to_string()),
                Component::VarField(ron_fields::MINOR.to_string()),
            ])
            .build();

        let overrides = vec!["0=5".to_string()];
        let bumps = vec!["2=2".to_string()];

        // Override core[0] (major) to 5, bump core[2] (minor) by 2
        zerv.process_schema_core(&overrides, &bumps).unwrap();

        assert_eq!(zerv.vars.major, Some(5)); // Override to 5
        assert_eq!(zerv.vars.minor, Some(2)); // 0 + 2 (bump)
    }

    #[test]
    fn test_process_schema_extra_core() {
        let mut zerv = ZervFixture::new()
            .with_extra_core(Component::VarField(bump_types::EPOCH.to_string()))
            .build();

        let overrides = vec!["0=5".to_string()];
        let bumps = vec!["0=2".to_string()];

        // Override extra_core[0] to 5, then bump by 2
        zerv.process_schema_extra_core(&overrides, &bumps).unwrap();

        // Verify the field was processed (override first, then bump)
        assert_eq!(zerv.vars.epoch, Some(7)); // 5 + 2
    }

    #[test]
    fn test_process_schema_build() {
        let mut zerv = ZervFixture::new()
            .with_build(Component::VarField(bump_types::MINOR.to_string()))
            .build();

        let overrides = vec!["0=10".to_string()];
        let bumps = vec!["0=3".to_string()];

        // Override build[0] to 10, then bump by 3
        zerv.process_schema_build(&overrides, &bumps).unwrap();

        assert_eq!(zerv.vars.minor, Some(13)); // 10 + 3
    }

    #[test]
    fn test_parse_bump_spec_explicit_value() {
        let (index, value) = Zerv::parse_bump_spec("1=5", 3).unwrap();
        assert_eq!(index, 1);
        assert_eq!(value, "5");
    }

    #[test]
    fn test_parse_bump_spec_default_value() {
        let (index, value) = Zerv::parse_bump_spec("1", 3).unwrap();
        assert_eq!(index, 1);
        assert_eq!(value, "1");
    }

    #[test]
    fn test_parse_bump_spec_negative_index() {
        let (index, value) = Zerv::parse_bump_spec("-1=3", 3).unwrap();
        assert_eq!(index, 2); // -1 maps to last index (2)
        assert_eq!(value, "3");
    }

    #[test]
    fn test_parse_bump_spec_negative_index_default_value() {
        let (index, value) = Zerv::parse_bump_spec("-1", 3).unwrap();
        assert_eq!(index, 2); // -1 maps to last index (2)
        assert_eq!(value, "1");
    }

    #[test]
    fn test_parse_bump_spec_string_value() {
        let (index, value) = Zerv::parse_bump_spec("0=release", 3).unwrap();
        assert_eq!(index, 0);
        assert_eq!(value, "release");
    }

    #[test]
    fn test_parse_bump_spec_invalid_index() {
        let result = Zerv::parse_bump_spec("5", 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_bump_spec_negative_index_out_of_bounds() {
        let result = Zerv::parse_bump_spec("-5", 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_bump_spec_negative_value() {
        let result = Zerv::parse_bump_spec("0=-1", 3);
        assert!(result.is_err());
    }

    // Tests for parse_and_validate_process_specs function
    #[test]
    fn test_parse_and_validate_process_specs_empty() {
        let specs = Zerv::parse_and_validate_process_specs(&[], &[], 3).unwrap();
        assert!(specs.is_empty());
    }

    #[test]
    fn test_parse_and_validate_process_specs_single_override() {
        let specs = Zerv::parse_and_validate_process_specs(&["1=5".to_string()], &[], 3).unwrap();
        assert_eq!(specs, vec![(1, Some("5".to_string()), None)]);
    }

    #[test]
    fn test_parse_and_validate_process_specs_multiple_sorted() {
        let specs = Zerv::parse_and_validate_process_specs(
            &["2=3".to_string(), "0=1".to_string()],
            &["1=2".to_string()],
            3,
        )
        .unwrap();
        assert_eq!(
            specs,
            vec![
                (0, Some("1".to_string()), None), // Override
                (1, None, Some("2".to_string())), // Bump
                (2, Some("3".to_string()), None), // Override
            ]
        );
    }

    #[test]
    fn test_parse_and_validate_process_specs_negative_indices() {
        let specs = Zerv::parse_and_validate_process_specs(
            &["-1=3".to_string(), "0=1".to_string()],
            &["-2=2".to_string()],
            3,
        )
        .unwrap();
        assert_eq!(
            specs,
            vec![
                (0, Some("1".to_string()), None), // Override
                (1, None, Some("2".to_string())), // Bump (-2 maps to 1)
                (2, Some("3".to_string()), None), // Override (-1 maps to 2)
            ]
        );
    }

    #[test]
    fn test_parse_and_validate_process_specs_mixed_override_bump() {
        let specs = Zerv::parse_and_validate_process_specs(
            &["0=5".to_string()],
            &["1=3".to_string(), "2=1".to_string()],
            3,
        )
        .unwrap();
        assert_eq!(
            specs,
            vec![
                (0, Some("5".to_string()), None), // Override
                (1, None, Some("3".to_string())), // Bump
                (2, None, Some("1".to_string())), // Bump
            ]
        );
    }

    #[test]
    fn test_parse_and_validate_process_specs_duplicate_indices() {
        let result = Zerv::parse_and_validate_process_specs(
            &["1=5".to_string(), "1=1".to_string()], // Duplicate index 1 in overrides
            &["0=3".to_string()],
            3,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Duplicate index 1")
        );
    }

    #[test]
    fn test_parse_and_validate_process_specs_duplicate_negative_indices() {
        let result = Zerv::parse_and_validate_process_specs(
            &["-1=5".to_string(), "0=1".to_string()],
            &["-1=3".to_string(), "-1=2".to_string()], // Duplicate index -1 in bumps
            3,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Duplicate index 2")
        ); // -1 maps to 2
    }

    #[test]
    fn test_parse_and_validate_process_specs_out_of_bounds() {
        let result =
            Zerv::parse_and_validate_process_specs(&["1=5".to_string()], &["5=1".to_string()], 3);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_parse_and_validate_process_specs_negative_out_of_bounds() {
        let result =
            Zerv::parse_and_validate_process_specs(&["1=5".to_string()], &["-5=1".to_string()], 3);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_parse_and_validate_process_specs_string_values() {
        let specs = Zerv::parse_and_validate_process_specs(
            &["1=release".to_string()],
            &["0=alpha".to_string()],
            3,
        )
        .unwrap();
        assert_eq!(
            specs,
            vec![
                (0, None, Some("alpha".to_string())),   // Bump
                (1, Some("release".to_string()), None), // Override
            ]
        );
    }

    #[test]
    fn test_parse_and_validate_process_specs_negative_values() {
        let result = Zerv::parse_and_validate_process_specs(&[], &["1=-5".to_string()], 3);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Negative bump values not supported")
        );
    }

    // Tests for override functionality
    #[test]
    fn test_parse_override_spec_explicit_value() {
        let (index, value) = Zerv::parse_override_spec("1=5", 3).unwrap();
        assert_eq!(index, 1);
        assert_eq!(value, "5");
    }

    #[test]
    fn test_parse_override_spec_negative_index() {
        let (index, value) = Zerv::parse_override_spec("-1=3", 3).unwrap();
        assert_eq!(index, 2); // -1 maps to last index (2)
        assert_eq!(value, "3");
    }

    #[test]
    fn test_parse_override_spec_string_value() {
        let (index, value) = Zerv::parse_override_spec("0=release", 3).unwrap();
        assert_eq!(index, 0);
        assert_eq!(value, "release");
    }

    #[test]
    fn test_parse_override_spec_missing_value() {
        let result = Zerv::parse_override_spec("1", 3);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires explicit value")
        );
    }

    #[test]
    fn test_parse_override_spec_invalid_index() {
        let result = Zerv::parse_override_spec("5=1", 3);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_process_schema_component_major() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::VarField(bump_types::MAJOR.to_string())])
            .build();

        zerv.process_schema_component("core", 0, Some("5".to_string()), None)
            .unwrap();
        assert_eq!(zerv.vars.major, Some(5));
    }

    #[test]
    fn test_process_schema_component_minor() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::VarField(bump_types::MINOR.to_string())])
            .build();

        zerv.process_schema_component("core", 0, Some("3".to_string()), None)
            .unwrap();
        assert_eq!(zerv.vars.minor, Some(3));
    }

    #[test]
    fn test_process_schema_component_patch() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::VarField(bump_types::PATCH.to_string())])
            .build();

        zerv.process_schema_component("core", 0, Some("7".to_string()), None)
            .unwrap();
        assert_eq!(zerv.vars.patch, Some(7));
    }

    #[test]
    fn test_process_schema_component_epoch() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::VarField(bump_types::EPOCH.to_string())])
            .build();

        zerv.process_schema_component("core", 0, Some("2".to_string()), None)
            .unwrap();
        assert_eq!(zerv.vars.epoch, Some(2));
    }

    #[test]
    fn test_process_schema_component_dirty() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::VarField("dirty".to_string())])
            .build();

        // DIRTY field is not handled by the main processing methods
        let result = zerv.process_schema_component("core", 0, Some("1".to_string()), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown field"));
    }

    #[test]
    fn test_process_schema_component_integer_override() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::Integer(42)])
            .build();

        // Override integer component
        zerv.process_schema_component("core", 0, Some("100".to_string()), None)
            .unwrap();

        // Verify the integer component was updated
        assert_eq!(zerv.schema.core[0], Component::Integer(100));
    }

    #[test]
    fn test_process_schema_component_integer_bump() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::Integer(42)])
            .build();

        // Bump integer component
        zerv.process_schema_component("core", 0, None, Some("5".to_string()))
            .unwrap();

        // Verify the integer component was bumped
        assert_eq!(zerv.schema.core[0], Component::Integer(47));
    }

    #[test]
    fn test_process_schema_component_integer_override_and_bump() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::Integer(42)])
            .build();

        // Override first, then bump from that base
        zerv.process_schema_component("core", 0, Some("100".to_string()), Some("5".to_string()))
            .unwrap();

        // Verify override was applied as base, then bump was added
        assert_eq!(zerv.schema.core[0], Component::Integer(105)); // 100 + 5
    }

    #[test]
    fn test_process_schema_component_integer_no_changes() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::Integer(42)])
            .build();

        // No override, no bump
        zerv.process_schema_component("core", 0, None, None)
            .unwrap();

        // Verify no changes were made
        assert_eq!(zerv.schema.core[0], Component::Integer(42));
    }

    #[test]
    fn test_process_schema_component_unknown_field() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::VarField("unknown_field".to_string())])
            .build();

        let result = zerv.process_schema_component("core", 0, Some("5".to_string()), None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot process custom field")
        );
    }
}
