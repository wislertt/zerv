use std::fmt::Display;
use std::str::FromStr;

use super::context::TemplateContext;
use super::helpers::register_helpers;
use crate::error::ZervError;
use crate::version::Zerv;

/// Template-aware type that can hold a direct value or template string
#[derive(Debug, Clone, PartialEq)]
pub enum Template<T> {
    Value(T),
    Template(String), // Handlebars template string
}

impl<T> Template<T>
where
    T: FromStr + Clone,
    T::Err: Display,
{
    /// Resolve template using optional Zerv object context, return final value
    pub fn resolve(&self, zerv: Option<&Zerv>) -> Result<T, ZervError> {
        match self {
            Template::Value(v) => Ok(v.clone()),
            Template::Template(template) => {
                let rendered = Self::render_template(template, zerv)?;
                let parsed = rendered.parse::<T>().map_err(|e| {
                    ZervError::TemplateError(format!("Failed to parse '{rendered}': {e}"))
                })?;
                Ok(parsed)
            }
        }
    }

    /// Render Handlebars template using optional Zerv object as context
    fn render_template(template: &str, zerv: Option<&Zerv>) -> Result<String, ZervError> {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars.set_strict_mode(false); // Allow missing variables

        // Register custom Zerv helpers
        register_helpers(&mut handlebars)?;

        // Create template context from Zerv object or empty context
        let context = if let Some(z) = zerv {
            let template_context = TemplateContext::from_zerv(z);
            serde_json::to_value(template_context)
                .map_err(|e| ZervError::TemplateError(format!("Serialization error: {e}")))?
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        handlebars
            .render_template(template, &context)
            .map_err(|e| ZervError::TemplateError(format!("Template render error: {e}")))
    }
}

impl Template<String> {
    /// Parse a template string and render it to final string result.
    /// Uses empty context (no Zerv object) and handles errors internally.
    pub fn render(template_str: &str) -> String {
        let template = Self::from(template_str.to_string());
        template
            .resolve(None)
            .unwrap_or_else(|e| format!("template_error: {}", e))
    }
}

impl From<String> for Template<String> {
    fn from(value: String) -> Self {
        if value.contains("{{") && value.contains("}}") {
            Template::Template(value)
        } else {
            Template::Value(value)
        }
    }
}

impl From<&str> for Template<String> {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<u32> for Template<u32> {
    fn from(value: u32) -> Self {
        Template::Value(value)
    }
}

impl From<&str> for Template<u32> {
    fn from(value: &str) -> Self {
        if value.contains("{{") && value.contains("}}") {
            Template::Template(value.to_string())
        } else {
            match value.parse::<u32>() {
                Ok(parsed) => Template::Value(parsed),
                Err(_) => Template::Template(value.to_string()), // Fallback to template
            }
        }
    }
}

impl<T> FromStr for Template<T>
where
    T: FromStr,
    T::Err: Display,
{
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.contains("{{") && input.contains("}}") {
            Ok(Template::Template(input.to_string()))
        } else {
            match input.parse::<T>() {
                Ok(value) => Ok(Template::Value(value)),
                Err(_) => Ok(Template::Template(input.to_string())), // Fallback to template
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::test_utils::zerv::ZervFixture;

    #[rstest]
    #[case("{{template}}", Template::Template("{{template}}".to_string()))]
    #[case("no_braces", Template::Value("no_braces".to_string()))]
    fn test_template_from_str(#[case] input: &str, #[case] expected: Template<String>) {
        let result = Template::from_str(input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("test", "test")]
    #[case("{{major}}.{{minor}}", "1.2")]
    fn test_template_resolve(#[case] input: &str, #[case] expected: &str) {
        let template: Template<String> = Template::from_str(input).unwrap();
        let zerv = ZervFixture::new().with_version(1, 2, 0).build();
        let result = template.resolve(Some(&zerv)).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    // Test basic string without template syntax
    #[case("static_text", "static_text")]
    // Test template with helpers (without Zerv context)
    #[case("{{hash_int 'test' 5}}", "16668")]
    #[case("{{hash 'test'}}", "c7dedb4")]
    #[case("{{sanitize 'Feature-Branch'}}", "feature.branch")]
    #[case("{{prefix 'abcdefghij' 5}}", "abcde")]
    // Test math helpers
    #[case("{{add 5 3}}", "8")]
    #[case("{{multiply 7 6}}", "42")]
    // Test complex template with multiple helpers
    #[case("hash_{{hash_int 'branch' 3}}_{{prefix 'commit' 2}}", "hash_380_co")]
    fn test_template_render(#[case] input: &str, #[case] expected: &str) {
        let result = Template::render(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_template_render_error_handling() {
        // Test with invalid helper that should produce error
        let result = Template::render("{{unknown_helper 'test'}}");

        // Should not panic and should return error message
        assert!(result.starts_with("template_error:"));
        assert!(result.contains("Template render error"));
    }

    #[test]
    fn test_template_render_empty_string() {
        let result = Template::render("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_template_render_with_context_variables() {
        // When using Template::render(), context variables should not be available
        // and should be treated as empty/missing
        let result = Template::render("{{missing_var}}");
        assert_eq!(result, ""); // Missing variables become empty strings
    }

    #[rstest]
    #[case("{{hash_int 'feature-1' 5}}")]
    #[case("{{sanitize 'Test-Branch-Name'}}")]
    #[case("{{add 10 20}}")]
    fn test_template_render_consistency(#[case] template_str: &str) {
        // Template::render() should give same result as the verbose chain
        let render_result = Template::render(template_str);

        let verbose_result = Template::from(template_str.to_string())
            .resolve(None)
            .unwrap_or_else(|e| format!("template_error: {}", e));

        assert_eq!(render_result, verbose_result);
    }
}
