use std::str::FromStr;

use super::context::TeraTemplateContext;
use super::functions::register_functions;
use crate::error::ZervError;
use crate::version::Zerv;

/// Tera-based template type
#[derive(Debug, Clone)]
pub struct TeraTemplate {
    template: String,
    tera: tera::Tera,
}

impl TeraTemplate {
    /// Create a new TeraTemplate from a string template
    pub fn new(template: String) -> Result<Self, ZervError> {
        let mut tera = tera::Tera::default();

        // Register custom functions
        register_functions(&mut tera)?;

        // Add the template to Tera instance
        let template_name = "template";
        tera.add_raw_template(template_name, &template)
            .map_err(|e| ZervError::TemplateError(format!("Failed to parse template: {}", e)))?;

        Ok(Self { template, tera })
    }

    /// Render the template with Zerv context
    pub fn render(&self, zerv: &Zerv) -> Result<String, ZervError> {
        let context = TeraTemplateContext::from_zerv(zerv);
        self.render_with_context(&context)
    }

    /// Render the template with custom context
    pub fn render_with_context(&self, context: &TeraTemplateContext) -> Result<String, ZervError> {
        let template_name = "template";

        self.tera
            .render(
                template_name,
                &tera::Context::from_serialize(context).map_err(|e| {
                    ZervError::TemplateError(format!("Failed to serialize context: {}", e))
                })?,
            )
            .map_err(|e| ZervError::TemplateError(format!("Failed to render template: {}", e)))
    }

    /// Get the raw template string
    pub fn as_str(&self) -> &str {
        &self.template
    }

    /// Resolve template to string (for compatibility with existing API)
    pub fn resolve(&self, zerv: &Zerv) -> Result<String, ZervError> {
        self.render(zerv)
    }
}

impl FromStr for TeraTemplate {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl TryFrom<&str> for TeraTemplate {
    type Error = ZervError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_string())
    }
}

impl TryFrom<String> for TeraTemplate {
    type Error = ZervError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::zerv::ZervFixture;

    #[test]
    fn test_tera_template_new() {
        let template = TeraTemplate::new("{{ major }}.{{ minor }}.{{ patch }}".to_string());
        assert!(template.is_ok());

        let template = template.unwrap();
        assert_eq!(template.as_str(), "{{ major }}.{{ minor }}.{{ patch }}");
    }

    #[test]
    fn test_tera_template_from_str() {
        let template = TeraTemplate::from_str("v{{ major }}.{{ minor }}");
        assert!(template.is_ok());

        let template = template.unwrap();
        assert_eq!(template.as_str(), "v{{ major }}.{{ minor }}");
    }

    #[test]
    fn test_tera_template_try_from() {
        let template = TeraTemplate::try_from("{{ major }}.{{ minor }}.{{ patch }}");
        assert!(template.is_ok());

        let template = template.unwrap();
        assert_eq!(template.as_str(), "{{ major }}.{{ minor }}.{{ patch }}");
    }

    #[test]
    fn test_tera_template_render_basic() {
        let template =
            TeraTemplate::new("{{ major }}.{{ minor }}.{{ patch }}".to_string()).unwrap();
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.2.3");
    }

    #[test]
    fn test_tera_template_render_with_expression() {
        let template =
            TeraTemplate::new("{{ major + 1 }}.{{ minor }}.{{ patch }}".to_string()).unwrap();
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2.2.3");
    }

    #[test]
    fn test_tera_template_render_with_default() {
        let template = TeraTemplate::new("{{ post | default(value=0) }}".to_string()).unwrap();
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0);
        // post is None by default
        let zerv = zerv_fixture.zerv();

        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0"); // Uses default value
    }

    #[test]
    fn test_tera_template_render_with_condition() {
        let template = TeraTemplate::new("{% if dirty %}{{ major }}.{{ minor }}.{{ patch }}-dirty{% else %}{{ major }}.{{ minor }}.{{ patch }}{% endif %}".to_string()).unwrap();
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            Some(0),
            Some(true),
            None,
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.2.3-dirty");
    }

    #[test]
    fn test_tera_template_invalid_syntax() {
        let template = TeraTemplate::new("{{ major }".to_string()); // Missing closing brace
        assert!(template.is_err());
    }

    #[test]
    fn test_tera_template_resolve_compatibility() {
        let template = TeraTemplate::new("v{{ major }}.{{ minor }}".to_string()).unwrap();
        let zerv_fixture = ZervFixture::new().with_version(2, 5, 0);
        let zerv = zerv_fixture.zerv();

        let result = template.resolve(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "v2.5");
    }
}
