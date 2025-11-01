// Re-export Handlebars implementation from subdirectory
pub mod handlebars;

// Re-export Tera implementation from subdirectory
pub mod tera;

// Integration testing for template engine migration
#[cfg(any(test, feature = "test-utils"))]
pub mod integration;

use std::fmt::Display;
use std::str::FromStr;

// Re-export Tera types for the new template system
pub use tera::TeraTemplateContext;

use crate::error::ZervError;
use crate::version::Zerv;

// Template interface using Tera only
#[derive(Debug, Clone, PartialEq)]
pub struct Template<T> {
    content: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Template<T>
where
    T: std::str::FromStr + Clone + std::fmt::Display,
    T::Err: std::fmt::Display,
{
    /// Create a new template with content
    pub fn new(content: String) -> Self {
        Template {
            content,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the template content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Resolve template using Tera
    pub fn resolve(&self, zerv: Option<&Zerv>) -> Result<Option<T>, ZervError> {
        let tera_template = tera::TeraTemplate::new(self.content.clone())?;
        tera_template.resolve(zerv)
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

// Type aliases for backward compatibility
pub type TemplateContext = tera::TeraTemplateContext;
pub type PreReleaseContext = tera::TeraTemplateContext;

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
