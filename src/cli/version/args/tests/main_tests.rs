use clap::Parser;

use super::super::*;
use crate::utils::constants::{
    formats,
    sources,
};

#[test]
fn test_main_config_defaults() {
    let config = MainConfig::try_parse_from(["version"]).unwrap();
    assert_eq!(config.source, sources::GIT);
    assert!(config.schema.is_none());
    assert!(config.schema_ron.is_none());
    assert_eq!(config.input_format, formats::AUTO);
    assert_eq!(config.output_format, formats::SEMVER);
    assert!(config.directory.is_none());
    assert!(config.output_template.is_none());
    assert!(config.output_prefix.is_none());
}

#[test]
fn test_main_config_with_overrides() {
    let config = MainConfig::try_parse_from([
        "zerv",
        "--source",
        "stdin",
        "--input-format",
        "semver",
        "--output-format",
        "pep440",
        "--schema",
        "calver",
        "--output-prefix",
        "version:",
        "-C",
        "/path/to/repo",
    ])
    .unwrap();

    assert_eq!(config.source, "stdin");
    assert_eq!(config.input_format, formats::SEMVER);
    assert_eq!(config.output_format, formats::PEP440);
    assert_eq!(config.schema, Some("calver".to_string()));
    assert_eq!(config.output_prefix, Some("version:".to_string()));
    assert_eq!(config.directory, Some("/path/to/repo".to_string()));
}
