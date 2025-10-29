use clap::Parser;

/// Version-specific configuration with schema support
#[derive(Parser, Debug, Clone, Default)]
pub struct MainConfig {
    // ============================================================================
    // SCHEMA OPTIONS
    // ============================================================================
    /// Schema preset name
    #[arg(long, help = "Schema preset name (standard, calver, etc.)")]
    pub schema: Option<String>,

    /// Custom RON schema definition
    #[arg(long, help = "Custom schema in RON format")]
    pub schema_ron: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_config_defaults() {
        let config = MainConfig::default();
        assert!(config.schema.is_none());
        assert!(config.schema_ron.is_none());
    }

    #[test]
    fn test_main_config_construction() {
        // Test direct construction with schema
        let config = MainConfig {
            schema: Some("calver".to_string()),
            schema_ron: None,
        };
        assert_eq!(config.schema, Some("calver".to_string()));
        assert!(config.schema_ron.is_none());
    }

    #[test]
    fn test_main_config_with_schema_ron() {
        let ron_schema = "core: [{var: \"major\"}]";
        let config = MainConfig {
            schema: None,
            schema_ron: Some(ron_schema.to_string()),
        };
        assert!(config.schema.is_none());
        assert_eq!(config.schema_ron, Some(ron_schema.to_string()));
    }

    #[test]
    fn test_main_config_with_both_schema_options() {
        let ron_schema = "core: [{var: \"major\"}]";
        let config = MainConfig {
            schema: Some("calver".to_string()),
            schema_ron: Some(ron_schema.to_string()),
        };
        assert_eq!(config.schema, Some("calver".to_string()));
        assert_eq!(config.schema_ron, Some(ron_schema.to_string()));
    }

    #[test]
    fn test_main_config_empty_args() {
        // Should parse successfully with no arguments
        let config = MainConfig::try_parse_from(&[] as &[&str]).unwrap();
        assert!(config.schema.is_none());
        assert!(config.schema_ron.is_none());
    }

    #[test]
    fn test_main_config_debug_format() {
        let config = MainConfig {
            schema: Some("test".to_string()),
            schema_ron: Some("custom schema".to_string()),
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("custom schema"));
    }

    #[test]
    fn test_main_config_clone() {
        let config = MainConfig {
            schema: Some("test".to_string()),
            schema_ron: Some("custom schema".to_string()),
        };
        let cloned = config.clone();
        assert_eq!(config.schema, cloned.schema);
        assert_eq!(config.schema_ron, cloned.schema_ron);
    }

    #[test]
    fn test_main_config_integration_with_version_args() {
        // Test that MainConfig works correctly within VersionArgs
        use super::super::VersionArgs;

        let args = VersionArgs::try_parse_from([
            "version",
            "--schema",
            "calver",
            "--schema-ron",
            "core: [{var: \"major\"}]",
        ])
        .unwrap();

        assert_eq!(args.main.schema, Some("calver".to_string()));
        assert_eq!(
            args.main.schema_ron,
            Some("core: [{var: \"major\"}]".to_string())
        );
    }
}
