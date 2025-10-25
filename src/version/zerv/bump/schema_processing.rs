use crate::error::ZervError;
use crate::version::zerv::components::{
    Component,
    Var,
};
use crate::version::zerv::core::Zerv;
use crate::version::zerv::schema::{
    SchemaPartName,
    ZervSchemaPart,
};

impl Zerv {
    pub fn process_schema_section(
        &mut self,
        section_name: SchemaPartName,
        overrides: &[String],
        bumps: &[String],
    ) -> Result<(), ZervError> {
        let schema_part = ZervSchemaPart::new(section_name, &self.schema);

        let specs = Self::parse_and_validate_process_specs(overrides, bumps, schema_part.clone())?;

        // Process specs in order from lower index to higher index
        for (index, override_value, bump_value) in specs {
            self.process_schema_component(section_name, index, override_value, bump_value)?;
        }
        Ok(())
    }

    fn parse_var_field_values(
        override_value: Option<&str>,
        bump_value: Option<&str>,
        schema_part: ZervSchemaPart,
    ) -> Result<(Option<u32>, Option<u32>), ZervError> {
        let override_val =
            Self::parse_optional_u32(override_value, "VarField", schema_part.clone())?;
        let bump_val = Self::parse_optional_u32(bump_value, "VarField", schema_part.clone())?;
        Ok((override_val, bump_val))
    }

    fn process_var_field(
        &mut self,
        var: &Var,
        override_value: Option<String>,
        bump_value: Option<String>,
        schema_part: ZervSchemaPart,
    ) -> Result<(), ZervError> {
        // Parse override and bump values
        let (override_val, bump_val) = Self::parse_var_field_values(
            override_value.as_deref(),
            bump_value.as_deref(),
            schema_part.clone(),
        )?;

        // Process field with both override and bump values
        match var {
            Var::Major => self.process_major(override_val, bump_val)?,
            Var::Minor => self.process_minor(override_val, bump_val)?,
            Var::Patch => self.process_patch(override_val, bump_val)?,
            Var::Epoch => self.process_epoch(override_val, bump_val)?,
            Var::Post => self.process_post(override_val, bump_val)?,
            Var::Dev => self.process_dev(override_val, bump_val)?,
            Var::PreRelease => self.process_pre_release_num(override_val, bump_val)?,
            Var::Custom(field_name) => {
                return Err(ZervError::InvalidBumpTarget {
                    message: format!("Unknown field: {field_name}"),
                    schema_part,
                    suggestion: None,
                });
            }
            Var::Distance
            | Var::Dirty
            | Var::BumpedBranch
            | Var::BumpedCommitHash
            | Var::BumpedCommitHashShort
            | Var::BumpedTimestamp
            | Var::LastBranch
            | Var::LastCommitHash
            | Var::LastCommitHashShort
            | Var::LastTimestamp => {
                return Err(ZervError::InvalidBumpTarget {
                    message: format!("Cannot process VCS-derived field: {var:?}"),
                    schema_part: schema_part.clone(),
                    suggestion: Some(
                        "VCS fields are automatically managed and cannot be manually overridden"
                            .to_string(),
                    ),
                });
            }
            _ => {
                return Err(ZervError::InvalidBumpTarget {
                    message: format!("Cannot process field: {var:?}"),
                    schema_part: schema_part.clone(),
                    suggestion: Some(
                        "This field type is not supported for schema processing".to_string(),
                    ),
                });
            }
        }

        Ok(())
    }

    fn process_integer_component(
        component: &mut Component,
        override_value: Option<String>,
        bump_value: Option<String>,
        schema_part: ZervSchemaPart,
    ) -> Result<(), ZervError> {
        if let Component::UInt(current_value) = component {
            // Parse override and bump values for UInt components
            let override_val =
                Self::parse_optional_u32(override_value.as_deref(), "UInt", schema_part.clone())?;
            let bump_val = Self::parse_optional_u32(bump_value.as_deref(), "UInt", schema_part)?;

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
            Err(ZervError::InvalidBumpTarget {
                message: "Expected UInt component".to_string(),
                schema_part,
                suggestion: None,
            })
        }
    }

    fn process_string_component(
        component: &mut Component,
        override_value: Option<String>,
        bump_value: Option<String>,
        schema_part: ZervSchemaPart,
    ) -> Result<(), ZervError> {
        if let Component::Str(current_value) = component {
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
            Err(ZervError::InvalidBumpTarget {
                message: "Expected String component".to_string(),
                schema_part,
                suggestion: None,
            })
        }
    }

    pub fn process_schema_component(
        &mut self,
        section: SchemaPartName,
        index: usize,
        override_value: Option<String>,
        bump_value: Option<String>,
    ) -> Result<(), ZervError> {
        // Get components for reading first
        let components = self.schema.get_part(&section);
        let components_len = components.len();

        if index >= components_len {
            let schema_part = ZervSchemaPart::new(section, &self.schema);
            return Err(ZervError::InvalidBumpTarget {
                message: format!("Index {index} out of bounds for {section} section"),
                schema_part,
                suggestion: None,
            });
        }

        let component = &components[index];

        match component {
            Component::Var(var) => match var {
                Var::Timestamp(_) => {
                    let schema_part = ZervSchemaPart::new(section, &self.schema);
                    return Err(ZervError::InvalidBumpTarget {
                        message: "Cannot process timestamp component - timestamps are generated dynamically"
                            .to_string(),
                        schema_part,
                        suggestion: None,
                    });
                }
                _ => {
                    let var_clone = var.clone();
                    let schema_part = ZervSchemaPart::new(section, &self.schema);
                    self.process_var_field(&var_clone, override_value, bump_value, schema_part)?;
                }
            },
            Component::Str(_) => {
                // Process String component directly (mutates the component)
                // For string components, we need to update through setters
                let mut components_vec = self.schema.get_part(&section).clone();
                let schema_part = ZervSchemaPart::new(section, &self.schema);
                Self::process_string_component(
                    &mut components_vec[index],
                    override_value,
                    bump_value,
                    schema_part,
                )?;
                self.schema.set_part(section, components_vec)?;
            }
            Component::UInt(_) => {
                // Process UInt component directly (mutates the component)
                // For integer components, we need to update through setters
                let mut components_vec = self.schema.get_part(&section).clone();
                let schema_part = ZervSchemaPart::new(section, &self.schema);
                Self::process_integer_component(
                    &mut components_vec[index],
                    override_value,
                    bump_value,
                    schema_part,
                )?;
                self.schema.set_part(section, components_vec)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::test_utils::ZervFixture;
    use crate::utils::constants::bump_types;
    use crate::version::zerv::components::Component;

    // Test schema processing functions (core, extra_core, build)
    #[rstest]
    #[case(SchemaPartName::Core, vec![Component::Var(Var::Major), Component::Var(Var::Minor)], vec!["0=5"], vec!["1=2"], |zerv: &mut Zerv| {
        assert_eq!(zerv.vars.major, Some(5)); // Override to 5
        assert_eq!(zerv.vars.minor, Some(2)); // 0 + 2 (bump)
    })] // core processing
    #[case(SchemaPartName::ExtraCore, vec![Component::Var(Var::Dev)], vec!["0=5"], vec!["0=2"], |zerv: &mut Zerv| {
        assert_eq!(zerv.vars.dev, Some(7)); // 5 + 2
    })] // extra_core processing
    #[case(SchemaPartName::Build, vec![Component::Str("initial".to_string())], vec!["0=override"], vec!["0=final"], |zerv: &mut Zerv| {
        assert_eq!(zerv.schema.build()[0], Component::Str("final".to_string())); // override then bump
    })] // build processing
    fn test_schema_processing_functions(
        #[case] section_name: SchemaPartName,
        #[case] components: Vec<Component>,
        #[case] overrides: Vec<&str>,
        #[case] bumps: Vec<&str>,
        #[case] assertions: impl Fn(&mut Zerv),
    ) {
        let mut zerv_fixture = ZervFixture::new();

        // Set up components based on section
        match section_name {
            SchemaPartName::Core => zerv_fixture = zerv_fixture.with_core_components(components),
            SchemaPartName::ExtraCore => {
                // For extra_core test, create a clean schema with only the test component
                zerv_fixture = ZervFixture::new()
                    .with_core_components(vec![Component::Var(Var::Major)])
                    .with_extra_core_components(components)
            }
            SchemaPartName::Build => zerv_fixture = zerv_fixture.with_build(components[0].clone()),
        }
        let mut zerv = zerv_fixture.build();

        let override_strings: Vec<String> = overrides.iter().map(|s| s.to_string()).collect();
        let bump_strings: Vec<String> = bumps.iter().map(|s| s.to_string()).collect();

        // Process based on section using the generic function
        zerv.process_schema_section(section_name, &override_strings, &bump_strings)
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
        let (var, section_name) = match field_type {
            "major" => (Var::Major, SchemaPartName::Core),
            "minor" => (Var::Minor, SchemaPartName::Core),
            "patch" => (Var::Patch, SchemaPartName::Core),
            "epoch" => (Var::Epoch, SchemaPartName::ExtraCore),
            _ => panic!("Unsupported field type in test: {field_type}"),
        };
        let mut zerv = match section_name {
            SchemaPartName::Core => ZervFixture::new()
                .with_core_components(vec![Component::Var(var)])
                .build(),
            SchemaPartName::ExtraCore => ZervFixture::new()
                .with_extra_core_components(vec![Component::Var(var)])
                .build(),
            SchemaPartName::Build => panic!("Build section not supported in this test"),
        };

        zerv.process_schema_component(
            section_name,
            0,
            override_value.map(|s| s.to_string()),
            bump_value.map(|s| s.to_string()),
        )
        .unwrap();

        assertions(&mut zerv);
    }

    // Test process_schema_component with UInt components
    #[rstest]
    #[case(Some("100"), None, Component::UInt(100))] // override only
    #[case(None, Some("5"), Component::UInt(47))] // bump only (42 + 5)
    #[case(Some("100"), Some("5"), Component::UInt(105))] // override + bump (100 + 5)
    #[case(None, None, Component::UInt(42))] // no changes
    #[case(Some("50"), Some("10"), Component::UInt(60))] // override 50 + bump 10
    #[case(Some("0"), Some("25"), Component::UInt(25))] // override 0 + bump 25
    #[case(Some("200"), Some("1"), Component::UInt(201))] // override 200 + bump 1
    fn test_process_schema_component_integer(
        #[case] override_value: Option<&str>,
        #[case] bump_value: Option<&str>,
        #[case] expected: Component,
    ) {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::UInt(42)])
            .build();

        zerv.process_schema_component(
            SchemaPartName::Core,
            0,
            override_value.map(|s| s.to_string()),
            bump_value.map(|s| s.to_string()),
        )
        .unwrap();

        assert_eq!(zerv.schema.core()[0], expected);
    }

    // Test process_schema_component with String components
    #[rstest]
    #[case(Some("release"), None, Component::Str("release".to_string()))] // override only
    #[case(None, Some("beta"), Component::Str("beta".to_string()))] // bump only
    #[case(Some("release"), Some("beta"), Component::Str("beta".to_string()))] // override first, then bump
    #[case(None, None, Component::Str("alpha".to_string()))] // no changes
    #[case(Some("stable"), Some("rc"), Component::Str("rc".to_string()))] // override first, then bump
    fn test_process_schema_component_string(
        #[case] override_value: Option<&str>,
        #[case] bump_value: Option<&str>,
        #[case] expected: Component,
    ) {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::Str("alpha".to_string())])
            .build();

        zerv.process_schema_component(
            SchemaPartName::Core,
            0,
            override_value.map(|s| s.to_string()),
            bump_value.map(|s| s.to_string()),
        )
        .unwrap();

        assert_eq!(zerv.schema.core()[0], expected);
    }

    // Test process_schema_component error cases
    #[rstest]
    #[case(Var::Dirty, "Cannot process VCS-derived field")] // VCS-derived field
    #[case(Var::Custom("unknown_field".to_string()), "Unknown field")] // custom field processing
    fn test_process_schema_component_errors(#[case] var: Var, #[case] expected_error: &str) {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::Var(var)])
            .build();

        let result =
            zerv.process_schema_component(SchemaPartName::Core, 0, Some("5".to_string()), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }

    #[test]
    fn test_process_schema_section_unknown_section() {
        let _zerv = ZervFixture::new().build();
        // Test parsing invalid section name - this should fail at parse level
        let section_str = "unknown_section";
        let parse_result: Result<SchemaPartName, _> = section_str.parse();
        assert!(parse_result.is_err());

        let error_msg = parse_result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid schema part name"));
    }

    #[test]
    fn test_process_var_field_pre_release_and_post() {
        let mut zerv = ZervFixture::new()
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
            .build();

        // Test PreRelease field (line 62)
        zerv.process_schema_component(SchemaPartName::ExtraCore, 0, Some("5".to_string()), None)
            .unwrap();

        // Test Post field (line 60)
        let mut zerv2 = ZervFixture::new()
            .with_extra_core_components(vec![Component::Var(Var::Post)])
            .build();
        zerv2
            .process_schema_component(SchemaPartName::ExtraCore, 0, Some("3".to_string()), None)
            .unwrap();
    }

    #[test]
    fn test_process_var_field_other_var_types() {
        // Test other Var types that go to the catch-all case (line 83-84)
        let mut zerv = ZervFixture::new()
            .with_extra_core_components(vec![Component::Var(Var::Distance)])
            .build();

        let result = zerv.process_schema_component(
            SchemaPartName::ExtraCore,
            0,
            Some("5".to_string()),
            None,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot process VCS-derived field")
        );
    }

    #[test]
    fn test_process_integer_component_error() {
        let mut component = Component::Str("not_an_int".to_string());
        let schema = crate::version::zerv::schema::ZervSchema::new(
            vec![Component::Var(Var::Major)], // Core must have at least one component
            vec![],
            vec![],
        )
        .unwrap();
        let schema_part = ZervSchemaPart::new(SchemaPartName::Core, &schema);
        let result = Zerv::process_integer_component(
            &mut component,
            Some("5".to_string()),
            None,
            schema_part,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Expected UInt component")
        );
    }

    #[test]
    fn test_process_string_component_error() {
        let mut component = Component::UInt(42);
        let schema = crate::version::zerv::schema::ZervSchema::new(
            vec![Component::Var(Var::Major)], // Core must have at least one component
            vec![],
            vec![],
        )
        .unwrap();
        let schema_part = ZervSchemaPart::new(SchemaPartName::Core, &schema);
        let result = Zerv::process_string_component(
            &mut component,
            Some("override".to_string()),
            None,
            schema_part,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Expected String component")
        );
    }

    #[test]
    fn test_process_schema_component_invalid_index() {
        let mut zerv = ZervFixture::new().build();
        // Build section is empty by default, so index 0 should be out of bounds
        let result = zerv.process_schema_component(SchemaPartName::Build, 0, None, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Index 0 out of bounds for build section")
        );
    }

    #[test]
    fn test_process_schema_component_index_out_of_bounds() {
        let mut zerv = ZervFixture::new()
            .with_core_components(vec![Component::Var(Var::Major)])
            .build();

        let result = zerv.process_schema_component(SchemaPartName::Core, 5, None, None); // Index 5 doesn't exist
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Index 5 out of bounds")
        );
    }

    #[test]
    fn test_process_schema_component_timestamp_error() {
        let mut zerv = ZervFixture::new()
            .with_extra_core_components(vec![Component::Var(Var::Timestamp(
                "compact_date".to_string(),
            ))])
            .build();

        let result = zerv.process_schema_component(SchemaPartName::ExtraCore, 0, None, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot process timestamp component")
        );
    }

    #[test]
    fn test_process_schema_component_with_build_section_integer() {
        let mut zerv = ZervFixture::new().with_build(Component::UInt(123)).build();

        // This should execute the build section integer processing (lines 224-225, 235-236)
        zerv.process_schema_component(
            SchemaPartName::Build,
            0,
            Some("200".to_string()),
            Some("10".to_string()),
        )
        .unwrap();

        assert_eq!(zerv.schema.build()[0], Component::UInt(210)); // 200 + 10
    }

    #[test]
    fn test_process_schema_component_with_extra_core_section_integer() {
        let mut zerv = ZervFixture::new()
            .with_extra_core_components(vec![Component::UInt(50)])
            .build();

        // This should execute the extra_core section integer processing
        zerv.process_schema_component(
            SchemaPartName::ExtraCore,
            0,
            Some("100".to_string()),
            Some("25".to_string()),
        )
        .unwrap();

        assert_eq!(zerv.schema.extra_core()[0], Component::UInt(125)); // 100 + 25
    }
}
