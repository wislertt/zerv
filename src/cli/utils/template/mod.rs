// Re-export Handlebars implementation from subdirectory
pub mod handlebars;

// Re-export handlebars types for backward compatibility
pub use handlebars::{
    PreReleaseContext,
    Template,
    TemplateContext,
};
