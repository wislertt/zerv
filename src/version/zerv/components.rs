use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

/// Variable field enum for type-safe field references
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    EnumString,
    Display,
    EnumIter,
    AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum Var {
    // Core version fields
    Major,
    Minor,
    Patch,
    Epoch,

    // Pre-release fields
    PreRelease,

    // Post-release fields
    Post,
    Dev,

    // VCS state fields
    Distance,
    Dirty,

    // VCS context fields (bumped)
    BumpedBranch,
    BumpedCommitHash,
    BumpedCommitHashShort,
    BumpedTimestamp,

    // VCS context fields (last)
    LastBranch,
    LastCommitHash,
    LastTimestamp,

    // Legacy fields for backward compatibility
    Branch,
    CommitHashShort,

    // Custom fields
    #[serde(rename = "custom")]
    #[strum(disabled)]
    Custom(String),

    // Timestamp patterns
    #[serde(rename = "ts")]
    #[strum(disabled)]
    Timestamp(String),
}

/// Component enum for internal use with compact serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "str")]
    Str(String),
    #[serde(rename = "int")]
    Int(u64),
    #[serde(rename = "var")]
    Var(Var),
}

/// ComponentConfig enum for human-readable RON serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentConfig {
    Str { value: String },
    Int { value: u64 },
    Var { field: Var },
}

// Conversion implementations
impl From<ComponentConfig> for Component {
    fn from(config: ComponentConfig) -> Self {
        match config {
            ComponentConfig::Str { value } => Component::Str(value),
            ComponentConfig::Int { value } => Component::Int(value),
            ComponentConfig::Var { field } => Component::Var(field),
        }
    }
}

impl From<&Component> for ComponentConfig {
    fn from(component: &Component) -> Self {
        match component {
            Component::Str(value) => ComponentConfig::Str {
                value: value.clone(),
            },
            Component::Int(value) => ComponentConfig::Int { value: *value },
            Component::Var(field) => ComponentConfig::Var {
                field: field.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_config_to_component_conversion() {
        let string_config = ComponentConfig::Str {
            value: "test".to_string(),
        };
        let component: Component = string_config.into();
        assert!(matches!(component, Component::Str(s) if s == "test"));

        let integer_config = ComponentConfig::Int { value: 42 };
        let component: Component = integer_config.into();
        assert!(matches!(component, Component::Int(42)));

        let var_config = ComponentConfig::Var { field: Var::Major };
        let component: Component = var_config.into();
        assert!(matches!(component, Component::Var(Var::Major)));
    }

    #[test]
    fn test_component_to_component_config_conversion() {
        let component = Component::Str("test".to_string());
        let config: ComponentConfig = (&component).into();
        assert!(matches!(config, ComponentConfig::Str { value } if value == "test"));

        let component = Component::Int(42);
        let config: ComponentConfig = (&component).into();
        assert!(matches!(config, ComponentConfig::Int { value: 42 }));

        let component = Component::Var(Var::Major);
        let config: ComponentConfig = (&component).into();
        assert!(matches!(config, ComponentConfig::Var { field: Var::Major }));
    }
}
