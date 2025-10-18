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
    /// Resolve template using Zerv object context, return final value
    pub fn resolve(&self, zerv: &Zerv) -> Result<T, ZervError> {
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

    /// Render Handlebars template using Zerv object as context
    fn render_template(template: &str, zerv: &Zerv) -> Result<String, ZervError> {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars.set_strict_mode(false); // Allow missing variables

        // Register custom Zerv helpers
        register_helpers(&mut handlebars)?;

        // Create template context from Zerv object
        let template_context = TemplateContext::from_zerv(zerv);
        let context = serde_json::to_value(template_context)
            .map_err(|e| ZervError::TemplateError(format!("Serialization error: {e}")))?;

        handlebars
            .render_template(template, &context)
            .map_err(|e| ZervError::TemplateError(format!("Template render error: {e}")))
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
        let result = template.resolve(&zerv).unwrap();
        assert_eq!(result, expected);
    }
}
