use crate::constants::bump_types;
use crate::error::ZervError;
use crate::version::zerv::components::Component;
use crate::version::zerv::core::Zerv;

impl Zerv {
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
    use rstest::*;

    use super::*;
    use crate::constants::ron_fields;
    use crate::test_utils::ZervFixture;
    use crate::version::zerv::components::Component;

    // Test schema processing functions (core, extra_core, build)
    #[rstest]
    #[case("core", vec![Component::VarField(ron_fields::MAJOR.to_string()), Component::String(".".to_string()), Component::VarField(ron_fields::MINOR.to_string())], vec!["0=5"], vec!["2=2"], |zerv: &mut Zerv| {
        assert_eq!(zerv.vars.major, Some(5)); // Override to 5
        assert_eq!(zerv.vars.minor, Some(2)); // 0 + 2 (bump)
    })] // core processing
    #[case("extra_core", vec![Component::VarField(bump_types::EPOCH.to_string())], vec!["0=5"], vec!["0=2"], |zerv: &mut Zerv| {
        assert_eq!(zerv.vars.epoch, Some(7)); // 5 + 2
    })] // extra_core processing
    #[case("build", vec![Component::VarField(bump_types::MINOR.to_string())], vec!["0=10"], vec!["0=3"], |zerv: &mut Zerv| {
        assert_eq!(zerv.vars.minor, Some(13)); // 10 + 3
    })] // build processing
    fn test_schema_processing_functions(
        #[case] section: &str,
        #[case] components: Vec<Component>,
        #[case] overrides: Vec<&str>,
        #[case] bumps: Vec<&str>,
        #[case] assertions: impl Fn(&mut Zerv),
    ) {
        let mut zerv_fixture = ZervFixture::new();

        // Set up components based on section
        match section {
            "core" => zerv_fixture = zerv_fixture.with_core_components(components),
            "extra_core" => zerv_fixture = zerv_fixture.with_extra_core(components[0].clone()),
            "build" => zerv_fixture = zerv_fixture.with_build(components[0].clone()),
            _ => panic!("Unknown section: {section}"),
        }
        let mut zerv = zerv_fixture.build();

        let override_strings: Vec<String> = overrides.iter().map(|s| s.to_string()).collect();
        let bump_strings: Vec<String> = bumps.iter().map(|s| s.to_string()).collect();

        // Process based on section
        match section {
            "core" => zerv
                .process_schema_core(&override_strings, &bump_strings)
                .unwrap(),
            "extra_core" => zerv
                .process_schema_extra_core(&override_strings, &bump_strings)
                .unwrap(),
            "build" => zerv
                .process_schema_build(&override_strings, &bump_strings)
                .unwrap(),
            _ => panic!("Unknown section: {section}"),
        }

        assertions(&mut zerv);
    }

    // Test process_schema_component with different field types
    #[rstest]
    #[case(bump_types::MAJOR, "5", None, |zerv: &mut Zerv| assert_eq!(zerv.vars.major, Some(5)))]
    #[case(bump_types::MINOR, "3", None, |zerv: &mut Zerv| assert_eq!(zerv.vars.minor, Some(3)))]
    #[case(bump_types::PATCH, "7", None, |zerv: &mut Zerv| assert_eq!(zerv.vars.patch, Some(7)))]
    #[case(bump_types::EPOCH, "2", None, |zerv: &mut Zerv| assert_eq!(zerv.vars.epoch, Some(2)))]
    fn test_process_schema_component_field_types(
        #[case] field_type: &str,
        #[case] override_value: &str,
        #[case] bump_value: Option<&str>,
        #[case] assertions: impl Fn(&mut Zerv),
    ) {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::VarField(field_type.to_string())])
            .build();

        zerv.process_schema_component(
            "core",
            0,
            Some(override_value.to_string()),
            bump_value.map(|s| s.to_string()),
        )
        .unwrap();

        assertions(&mut zerv);
    }

    // Test process_schema_component with Integer components
    #[rstest]
    #[case(Some("100"), None, Component::Integer(100))] // override only
    #[case(None, Some("5"), Component::Integer(47))] // bump only (42 + 5)
    #[case(Some("100"), Some("5"), Component::Integer(105))] // override + bump (100 + 5)
    #[case(None, None, Component::Integer(42))] // no changes
    fn test_process_schema_component_integer(
        #[case] override_value: Option<&str>,
        #[case] bump_value: Option<&str>,
        #[case] expected: Component,
    ) {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::Integer(42)])
            .build();

        zerv.process_schema_component(
            "core",
            0,
            override_value.map(|s| s.to_string()),
            bump_value.map(|s| s.to_string()),
        )
        .unwrap();

        assert_eq!(zerv.schema.core[0], expected);
    }

    // Test process_schema_component error cases
    #[rstest]
    #[case("dirty", "Unknown field")] // unknown field
    #[case("unknown_field", "Cannot process custom field")] // custom field not in precedence
    fn test_process_schema_component_errors(
        #[case] field_name: &str,
        #[case] expected_error: &str,
    ) {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::VarField(field_name.to_string())])
            .build();

        let result = zerv.process_schema_component("core", 0, Some("5".to_string()), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }
}
