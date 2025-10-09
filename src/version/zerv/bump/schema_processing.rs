use crate::constants::bump_types;
use crate::error::ZervError;
use crate::version::zerv::components::Component;
use crate::version::zerv::core::Zerv;

impl Zerv {
    pub fn process_schema_section(
        &mut self,
        section: &str,
        overrides: &[String],
        bumps: &[String],
    ) -> Result<(), ZervError> {
        let component_count = match section {
            "core" => self.schema.core.len(),
            "extra_core" => self.schema.extra_core.len(),
            "build" => self.schema.build.len(),
            _ => {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Unknown schema section: {section}"
                )));
            }
        };

        let specs = Self::parse_and_validate_process_specs(overrides, bumps, component_count)?;

        // Process specs in order from lower index to higher index
        for (index, override_value, bump_value) in specs {
            self.process_schema_component(section, index, override_value, bump_value)?;
        }
        Ok(())
    }

    fn validate_field_can_be_processed(
        field_name: &str,
        precedence_names: &[String],
    ) -> Result<(), ZervError> {
        if !precedence_names.contains(&field_name.to_string()) {
            return Err(ZervError::InvalidBumpTarget(format!(
                "Cannot process custom field: {field_name}"
            )));
        }
        Ok(())
    }

    fn parse_var_field_values(
        override_value: Option<&str>,
        bump_value: Option<&str>,
    ) -> Result<(Option<u32>, Option<u32>), ZervError> {
        let override_val = Self::parse_optional_u32(override_value, "VarField")?;
        let bump_val = Self::parse_optional_u32(bump_value, "VarField")?;
        Ok((override_val, bump_val))
    }

    fn process_var_field(
        &mut self,
        field_name: String,
        override_value: Option<String>,
        bump_value: Option<String>,
        precedence_names: &[String],
    ) -> Result<(), ZervError> {
        let field_name_str = field_name.as_str();

        // Validate field can be processed
        Self::validate_field_can_be_processed(field_name_str, precedence_names)?;

        // Parse override and bump values
        let (override_val, bump_val) =
            Self::parse_var_field_values(override_value.as_deref(), bump_value.as_deref())?;

        // Process field with both override and bump values
        match field_name_str {
            bump_types::MAJOR => self.process_major(override_val, bump_val)?,
            bump_types::MINOR => self.process_minor(override_val, bump_val)?,
            bump_types::PATCH => self.process_patch(override_val, bump_val)?,
            bump_types::EPOCH => self.process_epoch(override_val, bump_val)?,
            bump_types::POST => self.process_post(override_val, bump_val)?,
            bump_types::DEV => self.process_dev(override_val, bump_val)?,
            bump_types::PRE_RELEASE_NUM => self.process_pre_release_num(override_val, bump_val)?,
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

        Ok(())
    }

    fn process_integer_component(
        component: &mut Component,
        override_value: Option<String>,
        bump_value: Option<String>,
    ) -> Result<(), ZervError> {
        if let Component::Integer(current_value) = component {
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

            // Update the component's value
            *current_value = new_value;
            Ok(())
        } else {
            Err(ZervError::InvalidBumpTarget(
                "Expected Integer component".to_string(),
            ))
        }
    }

    fn process_string_component(
        component: &mut Component,
        override_value: Option<String>,
        bump_value: Option<String>,
    ) -> Result<(), ZervError> {
        if let Component::String(current_value) = component {
            // 1. Override step - set absolute value if specified
            if let Some(override_val) = override_value {
                *current_value = override_val;
            }

            // 2. Bump step - replace with bump value if specified
            if let Some(bump_val) = bump_value {
                *current_value = bump_val;
            }

            Ok(())
        } else {
            Err(ZervError::InvalidBumpTarget(
                "Expected String component".to_string(),
            ))
        }
    }

    pub fn process_schema_component(
        &mut self,
        section: &str,
        index: usize,
        override_value: Option<String>,
        bump_value: Option<String>,
    ) -> Result<(), ZervError> {
        // Extract precedence names first to avoid borrowing conflicts
        let precedence_names: Vec<String> = self
            .schema
            .precedence_order
            .field_precedence_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

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

        let component: &Component = components.get(index).ok_or_else(|| {
            ZervError::InvalidBumpTarget(format!(
                "Index {index} out of bounds for {section} section"
            ))
        })?;

        match component {
            Component::VarField(field_name) => {
                let field_name = field_name.clone();

                self.process_var_field(field_name, override_value, bump_value, &precedence_names)?;
            }
            Component::String(_) => {
                // Process String component directly (mutates the component)
                if let Some(component) = components.get_mut(index) {
                    Self::process_string_component(component, override_value, bump_value)?;
                } else {
                    return Err(ZervError::InvalidBumpTarget(format!(
                        "Component at index {index} not found"
                    )));
                }
            }
            Component::Integer(_) => {
                // Process Integer component directly (mutates the component)
                if let Some(component) = components.get_mut(index) {
                    Self::process_integer_component(component, override_value, bump_value)?;
                } else {
                    return Err(ZervError::InvalidBumpTarget(format!(
                        "Component at index {index} not found"
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
    #[case("core", vec![Component::VarField(ron_fields::MAJOR.to_string()), Component::VarField(ron_fields::MINOR.to_string())], vec!["0=5"], vec!["1=2"], |zerv: &mut Zerv| {
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

        // Process based on section using the generic function
        zerv.process_schema_section(section, &override_strings, &bump_strings)
            .unwrap();

        assertions(&mut zerv);
    }

    // Test process_schema_component with different field types
    #[rstest]
    #[case(bump_types::MAJOR, Some("5"), None, |zerv: &mut Zerv| assert_eq!(zerv.vars.major, Some(5)))]
    #[case(bump_types::MINOR, Some("3"), None, |zerv: &mut Zerv| assert_eq!(zerv.vars.minor, Some(3)))]
    #[case(bump_types::PATCH, Some("7"), None, |zerv: &mut Zerv| assert_eq!(zerv.vars.patch, Some(7)))]
    #[case(bump_types::EPOCH, Some("2"), None, |zerv: &mut Zerv| assert_eq!(zerv.vars.epoch, Some(2)))]
    #[case(bump_types::MAJOR, None, Some("3"), |zerv: &mut Zerv| assert_eq!(zerv.vars.major, Some(4)))]
    #[case(bump_types::MINOR, None, Some("2"), |zerv: &mut Zerv| assert_eq!(zerv.vars.minor, Some(2)))]
    #[case(bump_types::PATCH, None, Some("4"), |zerv: &mut Zerv| assert_eq!(zerv.vars.patch, Some(4)))]
    #[case(bump_types::EPOCH, None, Some("1"), |zerv: &mut Zerv| assert_eq!(zerv.vars.epoch, Some(1)))]
    #[case(bump_types::MAJOR, Some("10"), Some("2"), |zerv: &mut Zerv| assert_eq!(zerv.vars.major, Some(12)))]
    #[case(bump_types::MINOR, Some("5"), Some("3"), |zerv: &mut Zerv| assert_eq!(zerv.vars.minor, Some(8)))]
    #[case(bump_types::PATCH, Some("20"), Some("1"), |zerv: &mut Zerv| assert_eq!(zerv.vars.patch, Some(21)))]
    #[case(bump_types::EPOCH, Some("3"), Some("2"), |zerv: &mut Zerv| assert_eq!(zerv.vars.epoch, Some(5)))]
    fn test_process_schema_component_field_types(
        #[case] field_type: &str,
        #[case] override_value: Option<&str>,
        #[case] bump_value: Option<&str>,
        #[case] assertions: impl Fn(&mut Zerv),
    ) {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::VarField(field_type.to_string())])
            .build();

        zerv.process_schema_component(
            "core",
            0,
            override_value.map(|s| s.to_string()),
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
    #[case(Some("50"), Some("10"), Component::Integer(60))] // override 50 + bump 10
    #[case(Some("0"), Some("25"), Component::Integer(25))] // override 0 + bump 25
    #[case(Some("200"), Some("1"), Component::Integer(201))] // override 200 + bump 1
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

    // Test process_schema_component with String components
    #[rstest]
    #[case(Some("release"), None, Component::String("release".to_string()))] // override only
    #[case(None, Some("beta"), Component::String("beta".to_string()))] // bump only
    #[case(Some("release"), Some("beta"), Component::String("beta".to_string()))] // override first, then bump
    #[case(None, None, Component::String("alpha".to_string()))] // no changes
    #[case(Some("stable"), Some("rc"), Component::String("rc".to_string()))] // override first, then bump
    fn test_process_schema_component_string(
        #[case] override_value: Option<&str>,
        #[case] bump_value: Option<&str>,
        #[case] expected: Component,
    ) {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::String("alpha".to_string())])
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
