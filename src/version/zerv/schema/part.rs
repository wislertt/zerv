use std::fmt::{
    Display,
    Formatter,
};
use std::str::FromStr;

use crate::error::ZervError;
use crate::version::zerv::components::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaPartName {
    Core,
    ExtraCore,
    Build,
}

impl Display for SchemaPartName {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SchemaPartName::Core => write!(f, "core"),
            SchemaPartName::ExtraCore => write!(f, "extra_core"),
            SchemaPartName::Build => write!(f, "build"),
        }
    }
}

impl FromStr for SchemaPartName {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "core" => Ok(SchemaPartName::Core),
            "extra_core" => Ok(SchemaPartName::ExtraCore),
            "build" => Ok(SchemaPartName::Build),
            unknown => Err(ZervError::InvalidArgument(format!(
                "Invalid schema part name: '{}'. Valid names are: core, extra_core, build",
                unknown
            ))),
        }
    }
}

/// Simple representation of a schema part for error context
#[derive(Debug, Clone)]
pub struct ZervSchemaPart<'a> {
    pub name: SchemaPartName,
    pub components: &'a Vec<Component>,
}

impl<'a> ZervSchemaPart<'a> {
    pub fn new(name: SchemaPartName, components: &'a Vec<Component>) -> Self {
        Self { name, components }
    }

    /// Convenience method to create from string with validation
    pub fn from_str(name: &str, components: &'a Vec<Component>) -> Result<Self, ZervError> {
        let name = SchemaPartName::from_str(name)?;
        Ok(Self { name, components })
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn suggest_valid_index_range(&self, invalid_index: isize) -> Option<String> {
        if self.components.is_empty() {
            return Some("The section is empty".to_string());
        }

        let len = self.components.len();
        let max_positive = len - 1;
        let min_negative = -(len as isize);

        // Show the valid range
        let range_suggestion = if len == 1 {
            "Valid indices: 0 or -1".to_string()
        } else {
            format!(
                "Valid indices: 0 to {} or -1 to {}",
                max_positive, min_negative
            )
        };

        if invalid_index >= 0 {
            // Positive index out of bounds
            if invalid_index as usize >= len {
                Some(format!(
                    "{}. Did you mean index {}?",
                    range_suggestion, max_positive
                ))
            } else {
                Some(range_suggestion)
            }
        } else {
            // Negative index out of bounds
            if invalid_index < min_negative {
                Some(format!(
                    "{}. Did you mean index {}?",
                    range_suggestion, min_negative
                ))
            } else {
                Some(range_suggestion)
            }
        }
    }
}

impl Display for ZervSchemaPart<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.components.is_empty() {
            return write!(f, "{}: No fields available", self.name);
        }

        // Simple implementation, exactly like ZervSchema::Display
        let ron_string = ron::to_string(self.components).map_err(|_| std::fmt::Error)?;
        write!(f, "{}: {}", self.name, ron_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::components::{
        Component,
        Var,
    };

    mod schema_part_name_tests {
        use super::*;

        #[test]
        fn test_schema_part_name_display() {
            assert_eq!(format!("{}", SchemaPartName::Core), "core");
            assert_eq!(format!("{}", SchemaPartName::ExtraCore), "extra_core");
            assert_eq!(format!("{}", SchemaPartName::Build), "build");
        }

        #[test]
        fn test_schema_part_name_from_str() {
            assert_eq!(
                "core".parse::<SchemaPartName>().unwrap(),
                SchemaPartName::Core
            );
            assert_eq!(
                "extra_core".parse::<SchemaPartName>().unwrap(),
                SchemaPartName::ExtraCore
            );
            assert_eq!(
                "build".parse::<SchemaPartName>().unwrap(),
                SchemaPartName::Build
            );
        }

        #[test]
        fn test_schema_part_name_from_str_invalid() {
            let result: Result<SchemaPartName, _> = "invalid".parse();
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.to_string().contains("Invalid schema part name"));
            assert!(error.to_string().contains("invalid"));
        }

        #[test]
        fn test_schema_part_name_equality() {
            assert_eq!(SchemaPartName::Core, SchemaPartName::Core);
            assert_ne!(SchemaPartName::Core, SchemaPartName::ExtraCore);
            assert_ne!(SchemaPartName::ExtraCore, SchemaPartName::Build);
        }
    }

    mod zerv_schema_part_tests {
        use super::*;

        #[test]
        fn test_schema_part_core_section() {
            let components = vec![
                Component::Var(Var::Major),
                Component::Var(Var::Minor),
                Component::Var(Var::Patch),
            ];
            let part = ZervSchemaPart::new(SchemaPartName::Core, &components);

            // Test Display implementation - assert exact expected output
            let display = format!("{}", part);
            assert_eq!(display, "core: [var(Major),var(Minor),var(Patch)]");

            // Test suggestion
            let suggestion = part.suggest_valid_index_range(5);
            assert_eq!(
                suggestion.unwrap(),
                "Valid indices: 0 to 2 or -1 to -3. Did you mean index 2?"
            );
        }

        #[test]
        fn test_schema_part_negative_index_suggestion() {
            let components = vec![
                Component::Var(Var::Major),
                Component::Var(Var::Minor),
                Component::Var(Var::Patch),
            ];
            let part = ZervSchemaPart::new(SchemaPartName::Core, &components);

            let suggestion = part.suggest_valid_index_range(-5);
            assert_eq!(
                suggestion.unwrap(),
                "Valid indices: 0 to 2 or -1 to -3. Did you mean index -3?"
            );
        }

        #[test]
        fn test_schema_part_empty_section() {
            let components = vec![];
            let part = ZervSchemaPart::new(SchemaPartName::Build, &components);

            let display = format!("{}", part);
            assert_eq!(display, "build: No fields available");

            let suggestion = part.suggest_valid_index_range(0);
            assert_eq!(suggestion, Some("The section is empty".to_string()));
        }

        #[test]
        fn test_schema_part_mixed_components() {
            let components = vec![
                Component::Var(Var::Major),
                Component::Str("test".to_string()),
                Component::UInt(42),
            ];
            let part = ZervSchemaPart::new(SchemaPartName::ExtraCore, &components);

            let display = format!("{}", part);
            assert_eq!(display, "extra_core: [var(Major),str(\"test\"),uint(42)]");
        }

        #[test]
        fn test_schema_part_len_and_empty() {
            let empty_components = vec![];
            let part = ZervSchemaPart::new(SchemaPartName::Core, &empty_components);
            assert_eq!(part.len(), 0);
            assert!(part.is_empty());

            let single_component = vec![Component::Var(Var::Major)];
            let part = ZervSchemaPart::new(SchemaPartName::Core, &single_component);
            assert_eq!(part.len(), 1);
            assert!(!part.is_empty());
        }

        #[test]
        fn test_schema_part_single_element_suggestion() {
            let components = vec![Component::Var(Var::Major)];
            let part = ZervSchemaPart::new(SchemaPartName::Core, &components);

            let suggestion = part.suggest_valid_index_range(5);
            assert_eq!(
                suggestion.unwrap(),
                "Valid indices: 0 or -1. Did you mean index 0?"
            );
        }

        #[test]
        fn test_schema_part_valid_indices_no_suggestion() {
            let components = vec![Component::Var(Var::Major), Component::Var(Var::Minor)];
            let part = ZervSchemaPart::new(SchemaPartName::Core, &components);

            // Valid index should return range suggestion but no specific index suggestion
            let suggestion = part.suggest_valid_index_range(1);
            assert_eq!(suggestion.unwrap(), "Valid indices: 0 to 1 or -1 to -2");
        }

        #[test]
        fn test_zerv_schema_part_from_str() {
            let components = vec![Component::Var(Var::Major)];

            let part = ZervSchemaPart::from_str("core", &components).unwrap();
            assert_eq!(part.name, SchemaPartName::Core);

            let part = ZervSchemaPart::from_str("extra_core", &components).unwrap();
            assert_eq!(part.name, SchemaPartName::ExtraCore);

            let part = ZervSchemaPart::from_str("build", &components).unwrap();
            assert_eq!(part.name, SchemaPartName::Build);
        }

        #[test]
        fn test_zerv_schema_part_from_str_invalid() {
            let components = vec![Component::Var(Var::Major)];

            let result = ZervSchemaPart::from_str("invalid", &components);
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Invalid schema part name")
            );
        }
    }
}
