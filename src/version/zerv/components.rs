use serde::{
    Deserialize,
    Serialize,
};

/// Component enum for internal use with compact serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "str")]
    String(String),
    #[serde(rename = "int")]
    Integer(u64),
    #[serde(rename = "var")]
    VarField(String),
    #[serde(rename = "ts")]
    VarTimestamp(String),
}

/// ComponentConfig enum for human-readable RON serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentConfig {
    String { value: String },
    Integer { value: u64 },
    VarField { field: String },
    VarTimestamp { pattern: String },
}

// Conversion implementations
impl From<ComponentConfig> for Component {
    fn from(config: ComponentConfig) -> Self {
        match config {
            ComponentConfig::String { value } => Component::String(value),
            ComponentConfig::Integer { value } => Component::Integer(value),
            ComponentConfig::VarField { field } => Component::VarField(field),
            ComponentConfig::VarTimestamp { pattern } => Component::VarTimestamp(pattern),
        }
    }
}

impl From<&Component> for ComponentConfig {
    fn from(component: &Component) -> Self {
        match component {
            Component::String(value) => ComponentConfig::String {
                value: value.clone(),
            },
            Component::Integer(value) => ComponentConfig::Integer { value: *value },
            Component::VarField(field) => ComponentConfig::VarField {
                field: field.clone(),
            },
            Component::VarTimestamp(pattern) => ComponentConfig::VarTimestamp {
                pattern: pattern.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_config_to_component_conversion() {
        let string_config = ComponentConfig::String {
            value: "test".to_string(),
        };
        let component: Component = string_config.into();
        assert!(matches!(component, Component::String(s) if s == "test"));

        let integer_config = ComponentConfig::Integer { value: 42 };
        let component: Component = integer_config.into();
        assert!(matches!(component, Component::Integer(42)));

        let var_field_config = ComponentConfig::VarField {
            field: "major".to_string(),
        };
        let component: Component = var_field_config.into();
        assert!(matches!(component, Component::VarField(f) if f == "major"));

        let var_timestamp_config = ComponentConfig::VarTimestamp {
            pattern: "YYYY".to_string(),
        };
        let component: Component = var_timestamp_config.into();
        assert!(matches!(component, Component::VarTimestamp(p) if p == "YYYY"));
    }

    #[test]
    fn test_component_to_component_config_conversion() {
        let component = Component::String("test".to_string());
        let config: ComponentConfig = (&component).into();
        assert!(matches!(config, ComponentConfig::String { value } if value == "test"));

        let component = Component::Integer(42);
        let config: ComponentConfig = (&component).into();
        assert!(matches!(config, ComponentConfig::Integer { value: 42 }));

        let component = Component::VarField("major".to_string());
        let config: ComponentConfig = (&component).into();
        assert!(matches!(config, ComponentConfig::VarField { field } if field == "major"));

        let component = Component::VarTimestamp("YYYY".to_string());
        let config: ComponentConfig = (&component).into();
        assert!(matches!(config, ComponentConfig::VarTimestamp { pattern } if pattern == "YYYY"));
    }
}
