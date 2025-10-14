use crate::error::ZervError;
use crate::utils::constants::{
    SUPPORTED_FORMATS,
    formats,
};
use crate::version::Zerv;
use crate::version::pep440::PEP440;
use crate::version::semver::SemVer;

/// Output formatter for version strings with support for prefixes and templates
pub struct OutputFormatter;

impl OutputFormatter {
    /// Format the Zerv object according to the specified output format and options
    pub fn format_output(
        zerv_object: &Zerv,
        output_format: &str,
        output_prefix: Option<&str>,
        output_template: Option<&str>,
    ) -> Result<String, ZervError> {
        // 1. Generate base output according to format
        let mut output = Self::format_base_output(zerv_object, output_format)?;

        // 2. Apply template if specified (future extension)
        if let Some(template) = output_template {
            output = Self::apply_template(&output, template, zerv_object)?;
        }

        // 3. Apply prefix if specified
        if let Some(prefix) = output_prefix {
            output = format!("{prefix}{output}");
        }

        Ok(output)
    }

    /// Generate base output according to the specified format
    fn format_base_output(zerv_object: &Zerv, output_format: &str) -> Result<String, ZervError> {
        match output_format {
            formats::PEP440 => Ok(PEP440::from(zerv_object.clone()).to_string()),
            formats::SEMVER => Ok(SemVer::from(zerv_object.clone()).to_string()),
            formats::ZERV => Ok(zerv_object.to_string()),
            format => Err(ZervError::UnknownFormat(format!(
                "Unknown output format: '{}'. Supported formats: {}",
                format,
                SUPPORTED_FORMATS.join(", ")
            ))),
        }
    }

    /// Apply template to the output (basic infrastructure for future extension)
    fn apply_template(
        base_output: &str,
        template: &str,
        _zerv_object: &Zerv,
    ) -> Result<String, ZervError> {
        // Basic template support - for now just replace {version} placeholder
        if template.contains("{version}") {
            Ok(template.replace("{version}", base_output))
        } else {
            // If no {version} placeholder, just return the template as-is
            // This allows for simple prefix/suffix templates
            Ok(template.to_string())
        }
    }

    /// Get list of supported output formats
    pub fn supported_formats() -> &'static [&'static str] {
        SUPPORTED_FORMATS
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::version::zerv::bump::precedence::PrecedenceOrder;
    use crate::version::zerv::{
        Component,
        Var,
    };
    use crate::version::{
        ZervSchema,
        ZervVars,
    };

    fn create_test_zerv() -> Zerv {
        Zerv {
            schema: ZervSchema::new_with_precedence(
                vec![
                    Component::Var(Var::Major),
                    Component::Var(Var::Minor),
                    Component::Var(Var::Patch),
                ],
                vec![],
                vec![],
                PrecedenceOrder::default(),
            )
            .unwrap(),
            vars: ZervVars {
                major: Some(1),
                minor: Some(2),
                patch: Some(3),
                distance: Some(0),
                dirty: Some(false),
                bumped_branch: Some("main".to_string()),
                bumped_commit_hash: Some("abc123".to_string()),
                dev: None,
                last_timestamp: Some(1234567890),
                ..Default::default()
            },
        }
    }

    #[rstest]
    #[case(formats::SEMVER, "1.2.3")]
    #[case(formats::PEP440, "1.2.3")]
    fn test_format_output_basic_formats(#[case] format: &str, #[case] expected: &str) {
        let zerv = create_test_zerv();
        let result = OutputFormatter::format_output(&zerv, format, None, None);
        assert!(result.is_ok(), "Formatting should succeed");

        let output = result.unwrap();
        assert_eq!(output, expected, "Output should match expected format");
        assert!(!output.contains('\n'), "Output should be single line");
    }

    #[test]
    fn test_format_output_zerv() {
        let zerv = create_test_zerv();
        let result = OutputFormatter::format_output(&zerv, formats::ZERV, None, None);
        assert!(result.is_ok(), "Zerv formatting should succeed");

        let output = result.unwrap();
        // Zerv format is complex RON, so we check for key components
        assert!(
            output.contains("schema"),
            "Zerv output should contain schema"
        );
        assert!(output.contains("vars"), "Zerv output should contain vars");
        assert!(
            output.contains("major: Some(1)"),
            "Should contain major version"
        );
        assert!(
            output.contains("minor: Some(2)"),
            "Should contain minor version"
        );
        assert!(
            output.contains("patch: Some(3)"),
            "Should contain patch version"
        );
    }

    #[rstest]
    #[case(Some("v"), None, "v1.2.3")]
    #[case(None, Some("Version: {version}"), "Version: 1.2.3")]
    #[case(Some("Release "), Some("{version}-final"), "Release 1.2.3-final")]
    fn test_format_output_with_options(
        #[case] prefix: Option<&str>,
        #[case] template: Option<&str>,
        #[case] expected: &str,
    ) {
        let zerv = create_test_zerv();
        let result = OutputFormatter::format_output(&zerv, formats::SEMVER, prefix, template);
        assert!(result.is_ok(), "Formatting should succeed");

        let output = result.unwrap();
        assert_eq!(output, expected, "Output should match expected format");
    }

    #[test]
    fn test_format_output_unknown_format() {
        let zerv = create_test_zerv();
        let result = OutputFormatter::format_output(&zerv, "unknown", None, None);
        assert!(result.is_err(), "Unknown format should fail");
        assert!(matches!(result, Err(ZervError::UnknownFormat(_))));
    }

    #[test]
    fn test_supported_formats() {
        let formats = OutputFormatter::supported_formats();
        assert!(formats.contains(&formats::SEMVER));
        assert!(formats.contains(&formats::PEP440));
        assert!(formats.contains(&formats::ZERV));
        assert_eq!(formats.len(), 3);
    }

    #[rstest]
    #[case("1.2.3", "custom-output", "custom-output")]
    #[case("1.2.3", "Version {version} ready", "Version 1.2.3 ready")]
    fn test_apply_template(
        #[case] base_output: &str,
        #[case] template: &str,
        #[case] expected: &str,
    ) {
        let zerv = create_test_zerv();
        let result = OutputFormatter::apply_template(base_output, template, &zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}
