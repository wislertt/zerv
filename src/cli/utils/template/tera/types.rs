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

    // === PHASE 3: Core Tera Functionality Tests ===
    // These tests demonstrate Tera's built-in advantages over Handlebars

    #[test]
    fn test_tera_builtin_math_operations() {
        // Test all basic math operations
        let template = TeraTemplate::new("{{ major + minor + patch }}".to_string()).unwrap();
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "6");

        // Test multiplication and division
        let template = TeraTemplate::new("{{ (major * 10) + patch }}".to_string()).unwrap();
        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "13");
    }

    #[test]
    fn test_tera_builtin_string_operations() {
        // Test string concatenation with Tera's ~ operator
        let template =
            TeraTemplate::new("v{{ major ~ '.' ~ minor ~ '.' ~ patch }}".to_string()).unwrap();
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "v1.2.3");
    }

    #[test]
    fn test_tera_builtin_string_filters() {
        // Test uppercase filter
        let template = TeraTemplate::new("{{ bumped_branch | upper }}".to_string()).unwrap();
        let zerv_fixture = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_branch("main".to_string());
        let zerv = zerv_fixture.zerv();

        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "MAIN");

        // Test lowercase filter
        let template = TeraTemplate::new("{{ bumped_branch | lower }}".to_string()).unwrap();
        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "main");

        // Test capitalize filter
        let template = TeraTemplate::new("{{ bumped_branch | capitalize }}".to_string()).unwrap();
        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Main");
    }

    #[test]
    fn test_era_builtin_default_values_comprehensive() {
        // Test default values for different field types
        let template = TeraTemplate::new(
            "post={{ post | default(value=0) }}, dev={{ dev | default(value=0) }}, epoch={{ epoch | default(value=0) }}"
                .to_string(),
        ).unwrap();
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0);
        // All these fields are None by default
        let zerv = zerv_fixture.zerv();

        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "post=0, dev=0, epoch=0");
    }

    #[test]
    fn test_era_advanced_conditional_logic() {
        // Test complex conditional with multiple conditions
        // Handle None distance case properly
        let template = TeraTemplate::new(
            "{% if dirty %}{{ major }}.{{ minor }}.{{ patch }}-dirty{% elif distance and distance > 0 %}{{ major }}.{{ minor }}.{{ patch }}-{{ distance }}{% else %}{{ major }}.{{ minor }}.{{ patch }}{% endif %}"
                .to_string(),
        ).unwrap();

        // Test dirty condition
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
        assert!(result.is_ok(), "Dirty condition failed: {:?}", result);
        assert_eq!(result.unwrap(), "1.2.3-dirty");

        // Test distance condition
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            Some(5),
            Some(false),
            None,
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(result.is_ok(), "Distance condition failed: {:?}", result);
        assert_eq!(result.unwrap(), "1.2.3-5");

        // Test else condition
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(result.is_ok(), "Else condition failed: {:?}", result);
        assert_eq!(result.unwrap(), "1.2.3");
    }

    #[test]
    fn test_era_logical_operations() {
        // Test logical AND/OR conditions - handle None values properly
        let template = TeraTemplate::new(
            "{% if dirty and distance and distance > 0 %}dirty-with-distance{% elif dirty or (distance and distance > 0) %}dirty-or-distance{% else %}clean{% endif %}"
                .to_string(),
        ).unwrap();

        // Test both dirty and distance
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            Some(5),
            Some(true),
            None,
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(
            result.is_ok(),
            "Both dirty and distance failed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), "dirty-with-distance");

        // Test only dirty
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
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
        assert!(result.is_ok(), "Only dirty failed: {:?}", result);
        assert_eq!(result.unwrap(), "dirty-or-distance");

        // Test only distance
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            Some(3),
            Some(false),
            None,
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(result.is_ok(), "Only distance failed: {:?}", result);
        assert_eq!(result.unwrap(), "dirty-or-distance");

        // Test neither
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0);
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(result.is_ok(), "Neither failed: {:?}", result);
        assert_eq!(result.unwrap(), "clean");
    }

    #[test]
    fn test_era_pre_release_context_access() {
        // Test accessing nested pre_release context
        let template = TeraTemplate::new(
            "{% if pre_release %}{{ major }}.{{ minor }}.{{ patch }}-{{ pre_release.label }}{% if pre_release.number %}.{{ pre_release.number }}{% endif %}{% else %}{{ major }}.{{ minor }}.{{ patch }}{% endif %}"
                .to_string(),
        ).unwrap();

        // Test with pre-release
        let zerv_fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(crate::version::zerv::PreReleaseLabel::Beta, Some(2));
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.2.3-beta.2");

        // Test without pre-release
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.2.3");
    }

    #[test]
    fn test_era_comparison_operations() {
        // Test comparison operators - handle None distance properly
        let template = TeraTemplate::new(
            "{% if major >= 1 %}stable{% else %}unstable{% endif %}-{% if distance and distance > 10 %}many-commits{% elif distance and distance > 0 %}some-commits{% else %}no-commits{% endif %}"
                .to_string(),
        ).unwrap();

        // Test with major >= 1 and distance > 10
        let zerv_fixture = ZervFixture::new().with_version(2, 0, 0).with_vcs_data(
            Some(15),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(
            result.is_ok(),
            "Major >=1 and distance >10 failed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), "stable-many-commits");

        // Test with major >= 1 and small distance
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            Some(3),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(
            result.is_ok(),
            "Major >=1 and small distance failed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), "stable-some-commits");

        // Test with major < 1 and no distance
        let zerv_fixture = ZervFixture::new().with_version(0, 1, 0);
        let zerv = zerv_fixture.zerv();
        let result = template.render(zerv);
        assert!(
            result.is_ok(),
            "Major <1 and no distance failed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), "unstable-no-commits");
    }
}
