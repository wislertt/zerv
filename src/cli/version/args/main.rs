use clap::Parser;

use crate::cli::utils::template::Template;
use crate::utils::constants::{
    SUPPORTED_FORMATS_ARRAY,
    formats,
    sources,
};

/// Main configuration for input, schema, and output
#[derive(Parser, Debug, Clone)]
pub struct MainConfig {
    // ============================================================================
    // 1. INPUT CONTROL
    // ============================================================================
    /// Input source for version data
    #[arg(long, default_value = sources::GIT, value_parser = [sources::GIT, sources::STDIN],
          help = "Input source: 'git' (extract from repository) or 'stdin' (read Zerv RON format)")]
    pub source: String,

    /// Input format for version string parsing
    #[arg(long, default_value = formats::AUTO, value_parser = [formats::AUTO, formats::SEMVER, formats::PEP440],
          help = "Input format: 'auto' (detect), 'semver', or 'pep440'")]
    pub input_format: String,

    /// Change to directory before running command
    #[arg(short = 'C', help = "Change to directory before running command")]
    pub directory: Option<String>,

    // ============================================================================
    // 2. SCHEMA
    // ============================================================================
    /// Schema preset name
    #[arg(long, help = "Schema preset name (standard, calver, etc.)")]
    pub schema: Option<String>,

    /// Custom RON schema definition
    #[arg(long, help = "Custom schema in RON format")]
    pub schema_ron: Option<String>,

    // ============================================================================
    // 3. OUTPUT CONTROL
    // ============================================================================
    /// Output format for generated version
    #[arg(long, default_value = formats::SEMVER, value_parser = SUPPORTED_FORMATS_ARRAY,
          help = format!("Output format: '{}' (default), '{}', or '{}' (RON format for piping)", formats::SEMVER, formats::PEP440, formats::ZERV))]
    pub output_format: String,

    /// Output template for custom formatting (Handlebars syntax)
    #[arg(
        long,
        help = "Output template for custom formatting (Handlebars syntax)"
    )]
    pub output_template: Option<Template<String>>,

    /// Prefix to add to output
    #[arg(
        long,
        help = "Prefix to add to version output (e.g., 'v' for 'v1.0.0')"
    )]
    pub output_prefix: Option<String>,
}

impl Default for MainConfig {
    fn default() -> Self {
        Self {
            source: sources::GIT.to_string(),
            input_format: formats::AUTO.to_string(),
            directory: None,
            schema: None,
            schema_ron: None,
            output_format: formats::SEMVER.to_string(),
            output_template: None,
            output_prefix: None,
        }
    }
}

impl MainConfig {
    /// Resolve schema selection with default fallback
    /// Returns (schema_name, schema_ron) with default applied if neither is provided
    pub fn resolve_schema(&self) -> (Option<&str>, Option<&str>) {
        match (self.schema.as_deref(), self.schema_ron.as_deref()) {
            (Some(name), None) => (Some(name), None),
            (None, Some(ron)) => (None, Some(ron)),
            (Some(_), Some(_)) => (self.schema.as_deref(), self.schema_ron.as_deref()), // Both provided - let validation handle conflict
            (None, None) => (Some("zerv-standard"), None), // Default fallback
        }
    }
}
