use super::generators::{self, ZervParams};
use crate::version::zerv::PreReleaseLabel;
use serde_json;

/// Basic Zerv RON with core version fields only
pub fn basic() -> String {
    generators::basic_zerv_ron(1, 2, 3)
}

/// Zerv RON with VCS context
pub fn vcs() -> String {
    let params = ZervParams::new(1, 2, 3, "main", "abc1234", 5, false);
    generators::vcs_zerv_ron(&params, 5)
}

/// Zerv RON with pre-release information
pub fn prerelease() -> String {
    let params = ZervParams::new(1, 2, 3, "main", "abc1234", 0, false);
    generators::prerelease_zerv_ron(&params, PreReleaseLabel::Alpha, None)
}

/// Zerv RON with custom fields
pub fn custom_fields() -> String {
    let params = ZervParams::new(1, 2, 3, "main", "abc1234", 0, false);
    generators::custom_fields_zerv_ron(
        &params,
        serde_json::json!({
            "environment": "prod",
            "build_number": "123"
        }),
    )
}

/// Zerv RON with PEP440 epoch
pub fn epoch() -> String {
    let params = ZervParams::new(1, 2, 3, "main", "abc1234", 0, false);
    generators::epoch_zerv_ron(&params, 2, PreReleaseLabel::Alpha, None, 1)
}

/// Zerv RON with development version
pub fn dev() -> String {
    let params = ZervParams::new(1, 2, 3, "main", "abc1234", 5, true);
    generators::dev_zerv_ron(&params, 20241201123045)
}

/// Zerv RON with last version information
pub fn last_version() -> String {
    let params = ZervParams::new(1, 2, 3, "main", "abc1234", 5, false);
    generators::last_version_zerv_ron(&params, 5, "main", "def5678", 1703123456)
}

/// Invalid RON that fails to parse
pub fn malformed() -> String {
    generators::malformed_ron()
}

/// Zerv RON with missing core variables (for error testing)
pub fn missing_core_vars() -> String {
    generators::missing_core_vars_zerv_ron(1)
}

/// Get all valid Zerv RON fixtures (using programmatic generation)
pub fn get_valid_fixtures() -> Vec<(&'static str, String)> {
    vec![
        ("basic", basic()),
        ("vcs", vcs()),
        ("prerelease", prerelease()),
        ("custom_fields", custom_fields()),
        ("epoch", epoch()),
        ("dev", dev()),
        ("last_version", last_version()),
    ]
}

/// Get all invalid Zerv RON fixtures for error testing (using programmatic generation)
pub fn get_invalid_fixtures() -> Vec<(&'static str, String)> {
    vec![
        ("malformed", malformed()),
        ("invalid_syntax", generators::invalid_syntax_ron()),
    ]
}

/// Get all Zerv RON fixtures (valid and invalid) - programmatic version
pub fn get_all_fixtures() -> Vec<(&'static str, String)> {
    let mut all = get_valid_fixtures();
    all.extend(get_invalid_fixtures());
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::Zerv;
    use ron::from_str;

    #[test]
    fn test_valid_fixtures_parse_correctly() {
        for (name, ron_str) in get_valid_fixtures() {
            let result: Result<Zerv, _> = from_str(&ron_str);
            assert!(
                result.is_ok(),
                "Fixture '{}' should parse correctly, but got error: {:?}",
                name,
                result.err()
            );
        }
    }

    #[test]
    fn test_invalid_fixtures_fail_parsing() {
        for (name, ron_str) in get_invalid_fixtures() {
            let result: Result<Zerv, _> = from_str(&ron_str);
            assert!(
                result.is_err(),
                "Fixture '{}' should fail parsing, but succeeded: {:?}",
                name,
                result.ok()
            );
        }
    }

    #[test]
    fn test_fixtures_use_correct_field_names() {
        // Test that valid fixtures use the correct field names in var() components
        for (name, ron_str) in get_valid_fixtures() {
            // The VCS fixture should use branch in the schema (this is correct)
            if name == "vcs" {
                assert!(
                    ron_str.contains("var(\"branch\")"),
                    "Fixture '{name}' should use branch in var() components"
                );
            }

            // Other fixtures should not use bumped_commit_hash or bumped_timestamp in schema
            if name != "vcs" {
                assert!(
                    !ron_str.contains("var(\"bumped_commit_hash\")"),
                    "Fixture '{name}' should not use bumped_commit_hash in var() components"
                );
                assert!(
                    !ron_str.contains("var(\"bumped_timestamp\")"),
                    "Fixture '{name}' should not use bumped_timestamp in var() components"
                );
            }
        }
    }
}
