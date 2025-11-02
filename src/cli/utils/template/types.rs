use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use once_cell::sync::OnceCell;

use super::context::ZervTemplateContext;
use super::functions::register_functions;
use crate::error::ZervError;
use crate::version::Zerv;

/// Template type using Tera engine with efficient caching
#[derive(Debug, Clone)]
pub struct Template<T> {
    template: String,
    _cached_tera: OnceCell<tera::Tera>,
    _phantom: PhantomData<T>,
}

impl<T: PartialEq> PartialEq for Template<T> {
    fn eq(&self, other: &Self) -> bool {
        self.template == other.template
    }
}

impl<T> Template<T>
where
    T: FromStr + Clone + Display,
    T::Err: Display,
{
    /// Create a new template from string (pure storage, no validation yet)
    pub fn new(template: String) -> Self {
        Self {
            template,
            _cached_tera: OnceCell::new(),
            _phantom: PhantomData,
        }
    }

    /// Get template content
    pub fn as_str(&self) -> &str {
        &self.template
    }

    /// Render template and parse to typed result
    pub fn render(&self, zerv: Option<&Zerv>) -> Result<Option<T>, ZervError> {
        let rendered = self.render_string(zerv)?;

        // Handle empty/null results
        let trimmed = rendered.trim().to_lowercase();
        if trimmed.is_empty() || matches!(trimmed.as_str(), "none" | "null" | "nil") {
            return Ok(None);
        }

        // Parse to target type
        let parsed = rendered
            .parse::<T>()
            .map_err(|e| ZervError::TemplateError(format!("Failed to parse '{rendered}': {e}")))?;
        Ok(Some(parsed))
    }

    /// Internal method: get or create cached Tera instance
    fn get_tera(&self) -> Result<&tera::Tera, ZervError> {
        self._cached_tera.get_or_try_init(|| {
            let mut tera = tera::Tera::default();
            register_functions(&mut tera)?; // Register only once!
            tera.add_raw_template("template", &self.template)
                .map_err(|e| {
                    ZervError::TemplateError(format!(
                        "Failed to parse template '{}': {}",
                        self.template, e
                    ))
                })?;
            Ok(tera)
        })
    }

    /// Internal method: render to string
    fn render_string(&self, zerv: Option<&Zerv>) -> Result<String, ZervError> {
        let tera = self.get_tera()?;
        let context = self.create_context(zerv)?;

        tera.render("template", &context)
            .map(|s| s.trim().to_string())
            .map_err(|e| {
                ZervError::TemplateError(format!(
                    "Template render error '{}': {}",
                    self.template, e
                ))
            })
    }

    /// Create template context from Zerv object
    fn create_context(&self, zerv: Option<&Zerv>) -> Result<tera::Context, ZervError> {
        if let Some(z) = zerv {
            let template_context = ZervTemplateContext::from_zerv(z);
            tera::Context::from_serialize(template_context)
                .map_err(|e| ZervError::TemplateError(format!("Serialization error: {e}")))
        } else {
            Ok(tera::Context::new())
        }
    }
}

// Additional trait implementations for clap compatibility
impl FromStr for Template<u32> {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.to_string()))
    }
}

impl From<String> for Template<u32> {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Template<u32> {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl From<u32> for Template<u32> {
    fn from(value: u32) -> Self {
        Self::new(value.to_string())
    }
}

impl From<String> for Template<String> {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Template<String> {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl FromStr for Template<String> {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.to_string()))
    }
}

// Tests updated for Tera implementation
// Comprehensive test coverage is provided by both unit and integration tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::zerv::ZervFixture;

    #[test]
    fn test_template_new() {
        let template = Template::<String>::new("{{ major }}.{{ minor }}.{{ patch }}".to_string());
        assert_eq!(template.as_str(), "{{ major }}.{{ minor }}.{{ patch }}");
    }

    #[test]
    fn test_template_from_str() {
        let template = Template::<String>::from_str("v{{ major }}.{{ minor }}");
        assert!(template.is_ok());

        let template = template.unwrap();
        assert_eq!(template.as_str(), "v{{ major }}.{{ minor }}");
    }

    #[test]
    fn test_template_try_from() {
        let template = Template::<String>::from("{{ major }}.{{ minor }}.{{ patch }}");
        assert_eq!(template.as_str(), "{{ major }}.{{ minor }}.{{ patch }}");
    }

    #[test]
    fn test_template_render_basic() {
        let template = Template::<String>::new("{{ major }}.{{ minor }}.{{ patch }}".to_string());
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("1.2.3".to_string()));
    }

    #[test]
    fn test_template_render_with_expression() {
        let template =
            Template::<String>::new("{{ major + 1 }}.{{ minor }}.{{ patch }}".to_string());
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("2.2.3".to_string()));
    }

    #[test]
    fn test_template_render_with_default() {
        let template = Template::<String>::new("{{ post | default(value=0) }}".to_string());
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0);
        // post is None by default
        let zerv = zerv_fixture.zerv();

        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("0".to_string())); // Uses default value
    }

    #[test]
    fn test_template_render_with_condition() {
        let template: Template<String> = Template::new("{% if dirty %}{{ major }}.{{ minor }}.{{ patch }}-dirty{% else %}{{ major }}.{{ minor }}.{{ patch }}{% endif %}".to_string());
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

        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("1.2.3-dirty".to_string()));
    }

    #[test]
    fn test_template_invalid_syntax() {
        let template = Template::<String>::new("{{ major }".to_string()); // Missing closing brace
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0);
        let zerv = zerv_fixture.zerv();

        let result = template.render(Some(zerv));
        assert!(result.is_err());
    }

    #[test]
    fn test_template_render_compatibility() {
        let template: Template<String> = Template::new("v{{ major }}.{{ minor }}".to_string());
        let zerv_fixture = ZervFixture::new().with_version(2, 5, 0);
        let zerv = zerv_fixture.zerv();

        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("v2.5".to_string()));
    }

    // === Core Tera Functionality Tests ===
    // These tests demonstrate Tera's built-in advantages

    #[test]
    fn test_builtin_math_operations() {
        // Test all basic math operations
        let template: Template<String> = Template::new("{{ major + minor + patch }}".to_string());
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("6".to_string()));

        // Test multiplication and division
        let template: Template<String> = Template::new("{{ (major * 10) + patch }}".to_string());
        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("13".to_string()));
    }

    #[test]
    fn test_builtin_string_operations() {
        // Test string concatenation with Tera's ~ operator
        let template: Template<String> =
            Template::new("v{{ major ~ '.' ~ minor ~ '.' ~ patch }}".to_string());
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("v1.2.3".to_string()));
    }

    #[test]
    fn test_builtin_string_filters() {
        // Test uppercase filter
        let template: Template<String> = Template::new("{{ bumped_branch | upper }}".to_string());
        let zerv_fixture = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_branch("main".to_string());
        let zerv = zerv_fixture.zerv();

        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("MAIN".to_string()));

        // Test lowercase filter
        let template: Template<String> = Template::new("{{ bumped_branch | lower }}".to_string());
        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("main".to_string()));

        // Test capitalize filter
        let template: Template<String> =
            Template::new("{{ bumped_branch | capitalize }}".to_string());
        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("Main".to_string()));
    }

    #[test]
    fn test_builtin_default_values_comprehensive() {
        // Test default values for different field types
        let template: Template<String> = Template::new(
            "post={{ post | default(value=0) }}, dev={{ dev | default(value=0) }}, epoch={{ epoch | default(value=0) }}"
                .to_string(),
        );
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0);
        // All these fields are None by default
        let zerv = zerv_fixture.zerv();

        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("post=0, dev=0, epoch=0".to_string()));
    }

    #[test]
    fn test_advanced_conditional_logic() {
        // Test complex conditional with multiple conditions
        // Handle None distance case properly
        let template: Template<String> = Template::new(
            "{% if dirty %}{{ major }}.{{ minor }}.{{ patch }}-dirty{% elif distance and distance > 0 %}{{ major }}.{{ minor }}.{{ patch }}-{{ distance }}{% else %}{{ major }}.{{ minor }}.{{ patch }}{% endif %}"
                .to_string(),
        );

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
        let result = template.render(Some(zerv));
        assert!(result.is_ok(), "Dirty condition failed: {:?}", result);
        assert_eq!(result.unwrap(), Some("1.2.3-dirty".to_string()));

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
        let result = template.render(Some(zerv));
        assert!(result.is_ok(), "Distance condition failed: {:?}", result);
        assert_eq!(result.unwrap(), Some("1.2.3-5".to_string()));

        // Test else condition
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();
        let result = template.render(Some(zerv));
        assert!(result.is_ok(), "Else condition failed: {:?}", result);
        assert_eq!(result.unwrap(), Some("1.2.3".to_string()));
    }

    #[test]
    fn test_logical_operations() {
        // Test logical AND/OR conditions - handle None values properly
        let template: Template<String> = Template::new(
            "{% if dirty and distance and distance > 0 %}dirty-with-distance{% elif dirty or (distance and distance > 0) %}dirty-or-distance{% else %}clean{% endif %}"
                .to_string(),
        );

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
        let result = template.render(Some(zerv));
        assert!(
            result.is_ok(),
            "Both dirty and distance failed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), Some("dirty-with-distance".to_string()));

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
        let result = template.render(Some(zerv));
        assert!(result.is_ok(), "Only dirty failed: {:?}", result);
        assert_eq!(result.unwrap(), Some("dirty-or-distance".to_string()));

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
        let result = template.render(Some(zerv));
        assert!(result.is_ok(), "Only distance failed: {:?}", result);
        assert_eq!(result.unwrap(), Some("dirty-or-distance".to_string()));

        // Test neither
        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0);
        let zerv = zerv_fixture.zerv();
        let result = template.render(Some(zerv));
        assert!(result.is_ok(), "Neither failed: {:?}", result);
        assert_eq!(result.unwrap(), Some("clean".to_string()));
    }

    #[test]
    fn test_pre_release_context_access() {
        // Test accessing nested pre_release context
        let template: Template<String> = Template::new(
            "{% if pre_release %}{{ major }}.{{ minor }}.{{ patch }}-{{ pre_release.label }}{% if pre_release.number %}.{{ pre_release.number }}{% endif %}{% else %}{{ major }}.{{ minor }}.{{ patch }}{% endif %}"
                .to_string(),
        );

        // Test with pre-release
        let zerv_fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(crate::version::zerv::PreReleaseLabel::Beta, Some(2));
        let zerv = zerv_fixture.zerv();
        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("1.2.3-beta.2".to_string()));

        // Test without pre-release
        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();
        let result = template.render(Some(zerv));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("1.2.3".to_string()));
    }

    #[test]
    fn test_comparison_operations() {
        // Test comparison operators - handle None distance properly
        let template: Template<String> = Template::new(
            "{% if major >= 1 %}stable{% else %}unstable{% endif %}-{% if distance and distance > 10 %}many-commits{% elif distance and distance > 0 %}some-commits{% else %}no-commits{% endif %}"
                .to_string(),
        );

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
        let result = template.render(Some(zerv));
        assert!(
            result.is_ok(),
            "Major >=1 and distance >10 failed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), Some("stable-many-commits".to_string()));

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
        let result = template.render(Some(zerv));
        assert!(
            result.is_ok(),
            "Major >=1 and small distance failed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), Some("stable-some-commits".to_string()));

        // Test with major < 1 and no distance
        let zerv_fixture = ZervFixture::new().with_version(0, 1, 0);
        let zerv = zerv_fixture.zerv();
        let result = template.render(Some(zerv));
        assert!(
            result.is_ok(),
            "Major <1 and no distance failed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), Some("unstable-no-commits".to_string()));
    }
}
