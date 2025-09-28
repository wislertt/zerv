use super::component::component_validation;
use crate::constants::ron_fields;
use crate::error::ZervError;
use crate::version::zerv::{Component, Zerv};

/// Schema structure validation utilities
pub mod structure_validation {
    use super::*;

    /// Validate the structure of a parsed Zerv object
    pub fn validate_zerv_structure(zerv: &Zerv) -> Result<(), ZervError> {
        // Check that schema has at least one component in core
        if zerv.schema.core.is_empty()
            && zerv.schema.extra_core.is_empty()
            && zerv.schema.build.is_empty()
        {
            return Err(ZervError::StdinError(
                "Invalid Zerv RON: schema must contain at least one component in core, extra_core, or build sections".to_string()
            ));
        }

        // Validate all schema components
        component_validation::validate_schema_components(&zerv.schema.core)?;
        component_validation::validate_schema_components(&zerv.schema.extra_core)?;
        component_validation::validate_schema_components(&zerv.schema.build)?;

        // Validate that VarField references in schema have corresponding values in vars (where applicable)
        let mut missing_vars = Vec::new();
        let mut check_var_field = |component: &Component| {
            if let Component::VarField(field_name) = component {
                match field_name.as_str() {
                    ron_fields::MAJOR if zerv.vars.major.is_none() => {
                        missing_vars.push(ron_fields::MAJOR)
                    }
                    ron_fields::MINOR if zerv.vars.minor.is_none() => {
                        missing_vars.push(ron_fields::MINOR)
                    }
                    ron_fields::PATCH if zerv.vars.patch.is_none() => {
                        missing_vars.push(ron_fields::PATCH)
                    }
                    ron_fields::EPOCH if zerv.vars.epoch.is_none() => {
                        missing_vars.push(ron_fields::EPOCH)
                    }
                    ron_fields::PRE_RELEASE if zerv.vars.pre_release.is_none() => {
                        missing_vars.push(ron_fields::PRE_RELEASE)
                    }
                    ron_fields::POST if zerv.vars.post.is_none() => {
                        missing_vars.push(ron_fields::POST)
                    }
                    ron_fields::DEV if zerv.vars.dev.is_none() => {
                        missing_vars.push(ron_fields::DEV)
                    }
                    ron_fields::DISTANCE if zerv.vars.distance.is_none() => {
                        missing_vars.push(ron_fields::DISTANCE)
                    }
                    ron_fields::DIRTY if zerv.vars.dirty.is_none() => {
                        missing_vars.push(ron_fields::DIRTY)
                    }
                    ron_fields::BRANCH if zerv.vars.bumped_branch.is_none() => {
                        missing_vars.push(ron_fields::BRANCH)
                    }
                    ron_fields::COMMIT_HASH_SHORT if zerv.vars.bumped_commit_hash.is_none() => {
                        missing_vars.push(ron_fields::COMMIT_HASH_SHORT)
                    }
                    ron_fields::LAST_COMMIT_HASH if zerv.vars.last_commit_hash.is_none() => {
                        missing_vars.push(ron_fields::LAST_COMMIT_HASH)
                    }
                    ron_fields::LAST_TIMESTAMP if zerv.vars.last_timestamp.is_none() => {
                        missing_vars.push(ron_fields::LAST_TIMESTAMP)
                    }
                    ron_fields::LAST_BRANCH if zerv.vars.last_branch.is_none() => {
                        missing_vars.push(ron_fields::LAST_BRANCH)
                    }
                    _ => {
                        // Custom fields and other valid fields don't need to be present in vars
                        // This is handled by the component validation above
                    }
                }
            }
        };

        // Check all schema components for missing variables
        for component in &zerv.schema.core {
            check_var_field(component);
        }
        for component in &zerv.schema.extra_core {
            check_var_field(component);
        }
        for component in &zerv.schema.build {
            check_var_field(component);
        }

        // Only warn about missing core version components (major, minor, patch)
        let core_missing: Vec<&str> = missing_vars
            .iter()
            .filter(|&&var| {
                matches!(
                    var,
                    ron_fields::MAJOR | ron_fields::MINOR | ron_fields::PATCH
                )
            })
            .copied()
            .collect();

        if !core_missing.is_empty() {
            return Err(ZervError::StdinError(format!(
                "Invalid Zerv RON: schema references missing core variables: {}. Ensure these fields are present in the vars section.",
                core_missing.join(", ")
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::{Zerv, ZervSchema, ZervVars};

    #[test]
    fn test_validate_zerv_structure_valid() {
        let zerv = Zerv {
            schema: ZervSchema {
                core: vec![Component::VarField("major".to_string())],
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars {
                major: Some(1),
                ..Default::default()
            },
        };
        assert!(structure_validation::validate_zerv_structure(&zerv).is_ok());
    }

    #[test]
    fn test_validate_zerv_structure_empty_schema() {
        let zerv = Zerv {
            schema: ZervSchema {
                core: vec![],
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars::default(),
        };
        let result = structure_validation::validate_zerv_structure(&zerv);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("schema must contain at least one component")
        );
    }

    #[test]
    fn test_validate_zerv_structure_invalid_field() {
        let zerv = Zerv {
            schema: ZervSchema {
                core: vec![Component::VarField("invalid_field".to_string())],
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars::default(),
        };
        let result = structure_validation::validate_zerv_structure(&zerv);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }

    #[test]
    fn test_validate_zerv_structure_invalid_timestamp() {
        let zerv = Zerv {
            schema: ZervSchema {
                core: vec![Component::VarTimestamp("INVALID".to_string())],
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars::default(),
        };
        let result = structure_validation::validate_zerv_structure(&zerv);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unknown timestamp pattern")
        );
    }
}
