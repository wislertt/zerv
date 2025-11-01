// Re-export Handlebars implementation from subdirectory
pub mod handlebars;

// Re-export Tera implementation from subdirectory
pub mod tera;

// Re-export handlebars types for backward compatibility
pub use handlebars::{
    PreReleaseContext,
    Template,
    TemplateContext,
};
// Export Tera types for future use
pub use tera::{
    TeraTemplate,
    TeraTemplateContext,
};
