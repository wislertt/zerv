use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use super::context::TemplateContext;
use super::helpers::register_helpers;
use crate::error::ZervError;
use crate::version::Zerv;

/// Template-aware type that always stores a string template
#[derive(Debug, Clone, PartialEq)]
pub struct Template<T> {
    content: String,
    _phantom: PhantomData<T>,
}

impl<T> Template<T>
where
    T: FromStr + Clone + Display,
    T::Err: Display,
{
    /// Create a new template with content
    pub fn new(content: String) -> Self {
        Template {
            content,
            _phantom: PhantomData,
        }
    }

    /// Get the template content
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn resolve(&self, zerv: Option<&Zerv>) -> Result<Option<T>, ZervError> {
        let rendered = Self::render_template(&self.content, zerv)?;

        let trimmed = rendered.trim().to_lowercase();
        if trimmed.is_empty() || matches!(trimmed.as_str(), "none" | "null" | "nil") {
            return Ok(None);
        }

        let parsed = rendered
            .parse::<T>()
            .map_err(|e| ZervError::TemplateError(format!("Failed to parse '{rendered}': {e}")))?;
        Ok(Some(parsed))
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

        let rendered = handlebars
            .render_template(template, &context)
            .map_err(|e| ZervError::TemplateError(format!("Template render error: {e}")))?;

        // Strip leading/trailing whitespace and normalize internal whitespace
        Ok(rendered.trim().to_string())
    }
}

impl Template<String> {
    /// Parse a template string and render it to final string result.
    /// Uses empty context (no Zerv object) and handles errors internally.
    pub fn render(template_str: &str) -> String {
        let template = Self::new(template_str.to_string());
        template
            .resolve(None)
            .map(|opt| opt.unwrap_or_default())
            .unwrap_or_else(|e| format!("template_error: {}", e))
    }
}

impl From<String> for Template<String> {
    fn from(value: String) -> Self {
        Template::new(value)
    }
}

impl From<&str> for Template<String> {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<u32> for Template<u32> {
    fn from(value: u32) -> Self {
        Template::new(value.to_string())
    }
}

impl From<&str> for Template<u32> {
    fn from(value: &str) -> Self {
        Template::new(value.to_string())
    }
}

impl<T> FromStr for Template<T>
where
    T: FromStr + Clone + Display,
    T::Err: Display,
{
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Template::new(input.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::test_utils::zerv::ZervFixture;

    #[rstest]
    #[case("{{template}}", Template::new("{{template}}".to_string()))]
    #[case("no_braces", Template::new("no_braces".to_string()))]
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
        assert_eq!(result.unwrap(), expected);
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
            .map(|opt| opt.unwrap_or_default())
            .unwrap_or_else(|e| format!("template_error: {}", e));

        assert_eq!(render_result, verbose_result);
    }

    #[rstest]
    #[case("none", None)] // lowercase
    #[case("NONE", None)] // uppercase
    #[case("None", None)] // mixed case
    #[case(" null ", None)] // with whitespace
    #[case("null", None)] // null keyword
    #[case("NULL", None)] // uppercase null
    #[case("Nil", None)] // nil keyword
    #[case("NIL", None)] // uppercase nil
    #[case("", None)] // empty string
    #[case("   ", None)] // whitespace only
    #[case("empty", Some("empty"))] // should NOT be None - literal string
    #[case("nothing", Some("nothing"))] // should NOT be None - literal string
    #[case("~", Some("~"))] // should NOT be None - literal string
    #[case("nonempty", Some("nonempty"))] // regular string
    #[case("alpha", Some("alpha"))] // valid version component
    fn test_template_resolve_none_keywords(#[case] input: &str, #[case] expected: Option<&str>) {
        let template: Template<String> = Template::from_str(input).unwrap();
        let result = template.resolve(None).unwrap();
        assert_eq!(result, expected.map(|s| s.to_string()));
    }

    #[rstest]
    #[case("{{major}}.{{minor}}.{{patch}}", Some("1.2.3"))] // normal template
    #[case("1.2.3", Some("1.2.3"))] // static version string
    #[case("none", None)] // none keyword without context
    #[case("null", None)] // null keyword without context
    #[case("empty", Some("empty"))] // literal string should not be None
    fn test_template_resolve_none_keywords_with_context(
        #[case] input: &str,
        #[case] expected: Option<&str>,
    ) {
        let template: Template<String> = Template::from_str(input).unwrap();
        let zerv = ZervFixture::new().with_version(1, 2, 3).build();
        let result = template.resolve(Some(&zerv)).unwrap();
        assert_eq!(result, expected.map(|s| s.to_string()));
    }

    #[test]
    fn test_template_resolve_numeric_none_keywords() {
        // Test with numeric templates
        let template: Template<u32> = Template::from_str("none").unwrap();
        let result = template.resolve(None).unwrap();
        assert_eq!(result, None);

        let template: Template<u32> = Template::from_str("null").unwrap();
        let result = template.resolve(None).unwrap();
        assert_eq!(result, None);

        // Regular number should resolve
        let template: Template<u32> = Template::from_str("5").unwrap();
        let result = template.resolve(None).unwrap();
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_template_resolve_preserves_literal_strings() {
        // Test that "empty", "nothing", "~" remain as literal strings
        let test_cases = vec![
            ("empty", "empty"),
            ("nothing", "nothing"),
            ("~", "~"),
            ("  empty  ", "empty"), // trimmed but preserved
            ("EMPTY", "EMPTY"),     // case preserved
        ];

        for (input, expected) in test_cases {
            let template: Template<String> = Template::from_str(input).unwrap();
            let result = template.resolve(None).unwrap();
            assert_eq!(
                result,
                Some(expected.trim().to_string()),
                "Input '{}' should resolve to '{}', not None",
                input,
                expected
            );
        }
    }
}
