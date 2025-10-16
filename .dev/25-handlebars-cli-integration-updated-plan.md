# Plan 25: Handlebars CLI Integration - Updated Implementation

## Overview

Implement Handlebars templating support for CLI arguments based on the completed foundation from Plans 19-24 and aligned with the ideal specification from Plan 11.

## Prerequisites - ✅ COMPLETED

- ✅ **Plan 19**: String Sanitization Utils (implemented in `src/utils/sanitize.rs`)
- ✅ **Plan 20**: Component Resolution Centralization (implemented in `src/version/zerv/components.rs`)
- ✅ **Plan 22**: Schema-First Zerv Conversion (completed)
- ✅ **Plan 23**: Schema-First Zerv Conversion Implementation (completed)
- ✅ **Plan 24**: SemVer to Zerv Cleanup (completed)

## Current State Analysis

### ✅ What's Already Implemented

1. **String Sanitization** (`src/utils/sanitize.rs`):
    - `Sanitizer` struct with format-specific methods
    - `pep440_local_str()`, `semver_str()`, `uint()`, `key()` sanitizers
    - Comprehensive sanitization rules and testing

2. **Component Resolution** (`src/version/zerv/components.rs`):
    - `Var` enum with categorization (primary/secondary/context components)
    - `resolve_value()` and `resolve_expanded_values()` methods
    - Integration with sanitizers for format-specific cleaning
    - Support for custom fields via `Var::Custom(String)`

3. **CLI Structure** (`src/cli/version/args/`):
    - `MainConfig` with `output_template: Option<String>`
    - `OverridesConfig` with all override fields as `Option<u32>` or `Option<String>`
    - `BumpsConfig` with all bump fields as `Option<Option<u32>>`
    - Schema component overrides: `core`, `extra_core`, `build` as `Vec<String>`

4. **Schema System** (`src/version/zerv/schema/`):
    - Validated schema API with private fields and getters
    - Component placement validation (primary→core, secondary→extra_core)
    - Schema-first conversion system

### 🔄 What Needs Implementation

1. **Template Types**: Replace primitive types with template-aware types
2. **Handlebars Integration**: Add handlebars dependency and processing
3. **Template Context**: Create template context from ZervVars
4. **Custom Helpers**: Implement Zerv-specific Handlebars helpers
5. **CLI Integration**: Update argument types to support templating
6. **Render Timing**: Implement early vs late rendering logic

## Implementation Plan

### Step 1: Add Handlebars Dependency

**File**: `Cargo.toml`

```toml
[dependencies]
# ... existing dependencies ...
handlebars = "^6.3"
```

### Step 2: Template Module Implementation

**File**: `src/cli/utils/template/mod.rs` (new)

```rust
mod types;
mod context;
mod helpers;
pub use types::{Template, IndexValue};
pub use context::{TemplateContext, PreReleaseContext};
```

**File**: `src/cli/utils/template/types.rs` (new)

```rust
use std::str::FromStr;
use std::fmt::Display;
use crate::version::zerv::vars::ZervVars;
use crate::error::ZervError;
use super::context::TemplateContext;
use super::helpers::register_helpers;

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
    /// Resolve template using ZervVars context, return final value
    pub fn resolve(&self, vars: &ZervVars) -> Result<T, ZervError> {
        match self {
            Template::Value(v) => Ok(v.clone()),
            Template::Template(template) => {
                let rendered = Self::render_template(template, vars)?;
                let parsed = rendered.parse::<T>()
                    .map_err(|e| ZervError::TemplateError(format!("Failed to parse '{}': {}", rendered, e)))?;
                Ok(parsed)
            }
        }
    }

    /// Render Handlebars template using ZervVars as context
    fn render_template(template: &str, vars: &ZervVars) -> Result<String, ZervError> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false); // Allow missing variables

        // Register custom Zerv helpers
        Self::register_custom_helpers(&mut handlebars)?;

        // Create template context from ZervVars
        let template_context = TemplateContext::from_zerv_vars(vars);
        let context = serde_json::to_value(template_context)
            .map_err(|e| ZervError::TemplateError(format!("Serialization error: {}", e)))?;

        handlebars.render_template(template, &context)
            .map_err(|e| ZervError::TemplateError(format!("Template render error: {}", e)))
    }

    /// Render Handlebars template using ZervVars as context
    fn render_template(template: &str, vars: &ZervVars) -> Result<String, ZervError> {
        let mut handlebars = handlebars::Handlebars::new();
        handlebars.set_strict_mode(false); // Allow missing variables

        // Register custom Zerv helpers
        register_helpers(&mut handlebars)?;

        // Create template context from ZervVars
        let template_context = TemplateContext::from_zerv_vars(vars);
        let context = serde_json::to_value(template_context)
            .map_err(|e| ZervError::TemplateError(format!("Serialization error: {}", e)))?;

        handlebars.render_template(template, &context)
            .map_err(|e| ZervError::TemplateError(format!("Template render error: {}", e)))
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

/// INDEX=VALUE pair for schema component arguments with template support
#[derive(Debug, Clone, PartialEq)]
pub struct IndexValue {
    pub index: usize,
    pub value: Template<String>,
}

impl FromStr for IndexValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (index_str, value_str) = s.split_once('=')
            .ok_or_else(|| format!("Invalid INDEX=VALUE format: {}", s))?;

        let index = index_str.parse::<usize>()
            .map_err(|_| format!("Invalid index: {}", index_str))?;

        let value = Template::from_str(value_str)?;

        Ok(IndexValue { index, value })
    }
}

/// Template context for Handlebars rendering
#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateContext {
    // Core version fields
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub epoch: Option<u64>,

    // Metadata fields
    pub post: Option<u64>,
    pub dev: Option<u64>,

    // Pre-release fields
    pub pre_release: Option<PreReleaseContext>,

    // VCS fields
    pub distance: Option<u64>,
    pub dirty: Option<bool>,
    pub bumped_branch: Option<String>,
    pub bumped_commit_hash: Option<String>,
    pub bumped_commit_hash_short: Option<String>,
    pub bumped_timestamp: Option<u64>,

    // Last version fields
    pub last_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_commit_hash_short: Option<String>,
    pub last_timestamp: Option<u64>,

    // Custom variables
    pub custom: serde_json::Value,

    // Formatted versions
    pub pep440: String,
    pub semver: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PreReleaseContext {
    pub label: String,
    pub number: Option<u64>,
}

impl TemplateContext {
    pub fn from_zerv_vars(vars: &ZervVars) -> Self {
        Self {
            major: vars.major,
            minor: vars.minor,
            patch: vars.patch,
            epoch: vars.epoch,
            post: vars.post,
            dev: vars.dev,
            pre_release: vars.pre_release.as_ref().map(|pr| PreReleaseContext {
                label: pr.label.label_str().to_string(),
                number: pr.number,
            }),
            distance: vars.distance,
            dirty: vars.dirty,
            bumped_branch: vars.bumped_branch.clone(),
            bumped_commit_hash: vars.bumped_commit_hash.clone(),
            bumped_commit_hash_short: vars.get_bumped_commit_hash_short(),
            bumped_timestamp: vars.bumped_timestamp,
            last_branch: vars.last_branch.clone(),
            last_commit_hash: vars.last_commit_hash.clone(),
            last_commit_hash_short: vars.get_last_commit_hash_short(),
            last_timestamp: vars.last_timestamp,
            custom: vars.custom.clone(),
            // TODO: Generate formatted versions
            pep440: "".to_string(), // Will be populated by conversion
            semver: "".to_string(),  // Will be populated by conversion
        }
    }
}

```

**File**: `src/cli/utils/template/helpers.rs` (new)

```rust
use handlebars::{Handlebars, Helper, Context, RenderContext, Output, HelperResult};
use crate::error::ZervError;

/// Register all custom Zerv helpers
pub fn register_helpers(handlebars: &mut Handlebars) -> Result<(), ZervError> {
    // Math helpers
    handlebars.register_helper("add", Box::new(add_helper));
    handlebars.register_helper("subtract", Box::new(subtract_helper));
    handlebars.register_helper("multiply", Box::new(multiply_helper));

    // String helpers
    handlebars.register_helper("hash", Box::new(hash_helper));
    handlebars.register_helper("hash_int", Box::new(hash_int_helper));
    handlebars.register_helper("prefix", Box::new(prefix_helper));

    // Timestamp helpers
    handlebars.register_helper("format_timestamp", Box::new(format_timestamp_helper));

    Ok(())
}

// Helper implementations would go here...
// (Math, string, and timestamp helper functions)
```

**File**: `src/cli/utils/template/context.rs` (new)

```rust
use crate::version::zerv::vars::ZervVars;
use serde::Serialize;

/// Template context for Handlebars rendering
#[derive(Debug, Clone, Serialize)]
pub struct TemplateContext {
    // Core version fields
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub epoch: Option<u64>,

    // Metadata fields
    pub post: Option<u64>,
    pub dev: Option<u64>,

    // Pre-release fields
    pub pre_release: Option<PreReleaseContext>,

    // VCS fields
    pub distance: Option<u64>,
    pub dirty: Option<bool>,
    pub bumped_branch: Option<String>,
    pub bumped_commit_hash: Option<String>,
    pub bumped_commit_hash_short: Option<String>,
    pub bumped_timestamp: Option<u64>,

    // Last version fields
    pub last_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_commit_hash_short: Option<String>,
    pub last_timestamp: Option<u64>,

    // Custom variables
    pub custom: serde_json::Value,

    // Formatted versions (only available in late rendering)
    pub pep440: Option<String>,
    pub semver: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreReleaseContext {
    pub label: String,
    pub number: Option<u64>,
}

impl TemplateContext {
    pub fn from_zerv_vars(vars: &ZervVars) -> Self {
        Self {
            major: vars.major,
            minor: vars.minor,
            patch: vars.patch,
            epoch: vars.epoch,
            post: vars.post,
            dev: vars.dev,
            pre_release: vars.pre_release.as_ref().map(|pr| PreReleaseContext {
                label: pr.label.label_str().to_string(),
                number: pr.number,
            }),
            distance: vars.distance,
            dirty: vars.dirty,
            bumped_branch: vars.bumped_branch.clone(),
            bumped_commit_hash: vars.bumped_commit_hash.clone(),
            bumped_commit_hash_short: vars.get_bumped_commit_hash_short(),
            bumped_timestamp: vars.bumped_timestamp,
            last_branch: vars.last_branch.clone(),
            last_commit_hash: vars.last_commit_hash.clone(),
            last_commit_hash_short: vars.get_last_commit_hash_short(),
            last_timestamp: vars.last_timestamp,
            custom: vars.custom.clone(),
            // Formatted versions populated separately based on timing
            pep440: None,
            semver: None,
        }
    }
}andlebars = handlebars::Handlebars::new();
    handlebars.set_strict_mode(false);
    register_helpers(&mut handlebars)?;

    let context = TemplateContext::from_zerv_vars(vars);
    let json_context = serde_json::to_value(context)
        .map_err(|e| ZervError::TemplateError(format!("Serialization error: {}", e)))?;

    handlebars.render_template(template, &json_context)
        .map_err(|e| ZervError::TemplateError(format!("Template render error: {}", e)))
}
```

**File**: `src/cli/utils/template/context.rs` (new)

```rust
use crate::version::zerv::vars::ZervVars;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateContext {
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub epoch: Option<u64>,
    pub post: Option<u64>,
    pub dev: Option<u64>,
    pub pre_release: Option<PreReleaseContext>,
    pub distance: Option<u64>,
    pub dirty: Option<bool>,
    pub bumped_branch: Option<String>,
    pub bumped_commit_hash: Option<String>,
    pub bumped_commit_hash_short: Option<String>,
    pub bumped_timestamp: Option<u64>,
    pub last_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_commit_hash_short: Option<String>,
    pub last_timestamp: Option<u64>,
    pub custom: serde_json::Value,
    pub pep440: String,
    pub semver: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PreReleaseContext {
    pub label: String,
    pub number: Option<u64>,
}

impl TemplateContext {
    pub fn from_zerv_vars(vars: &ZervVars) -> Self {
        Self {
            major: vars.major,
            minor: vars.minor,
            patch: vars.patch,
            epoch: vars.epoch,
            post: vars.post,
            dev: vars.dev,
            pre_release: vars.pre_release.as_ref().map(|pr| PreReleaseContext {
                label: pr.label.label_str().to_string(),
                number: pr.number,
            }),
            distance: vars.distance,
            dirty: vars.dirty,
            bumped_branch: vars.bumped_branch.clone(),
            bumped_commit_hash: vars.bumped_commit_hash.clone(),
            bumped_commit_hash_short: vars.get_bumped_commit_hash_short(),
            bumped_timestamp: vars.bumped_timestamp,
            last_branch: vars.last_branch.clone(),
            last_commit_hash: vars.last_commit_hash.clone(),
            last_commit_hash_short: vars.get_last_commit_hash_short(),
            last_timestamp: vars.last_timestamp,
            custom: vars.custom.clone(),
            pep440: "".to_string(),
            semver: "".to_string(),
        }
    }
}
```

**File**: `src/cli/utils/template/helpers.rs` (new)

```rust
use handlebars::{Handlebars, Helper, Context, RenderContext, Output, HelperResult, RenderError};
use crate::error::ZervError;

pub fn register_helpers(handlebars: &mut Handlebars) -> Result<(), ZervError> {
    handlebars.register_helper("add", Box::new(add_helper));
    handlebars.register_helper("subtract", Box::new(subtract_helper));
    handlebars.register_helper("multiply", Box::new(multiply_helper));
    handlebars.register_helper("hash", Box::new(hash_helper));
    handlebars.register_helper("hash_int", Box::new(hash_int_helper));
    handlebars.register_helper("prefix", Box::new(prefix_helper));
    handlebars.register_helper("format_timestamp", Box::new(format_timestamp_helper));
    Ok(())
}

fn add_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let a = h.param(0).and_then(|v| v.value().as_u64()).ok_or_else(|| RenderError::new("First parameter must be a number"))?;
    let b = h.param(1).and_then(|v| v.value().as_u64()).ok_or_else(|| RenderError::new("Second parameter must be a number"))?;
    out.write(&(a + b).to_string())?;
    Ok(())
}

fn subtract_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let a = h.param(0).and_then(|v| v.value().as_u64()).ok_or_else(|| RenderError::new("First parameter must be a number"))?;
    let b = h.param(1).and_then(|v| v.value().as_u64()).ok_or_else(|| RenderError::new("Second parameter must be a number"))?;
    out.write(&a.saturating_sub(b).to_string())?;
    Ok(())
}

fn multiply_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let a = h.param(0).and_then(|v| v.value().as_u64()).ok_or_else(|| RenderError::new("First parameter must be a number"))?;
    let b = h.param(1).and_then(|v| v.value().as_u64()).ok_or_else(|| RenderError::new("Second parameter must be a number"))?;
    out.write(&(a * b).to_string())?;
    Ok(())
}

fn hash_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let input = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| RenderError::new("First parameter must be a string"))?;
    let length = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(7) as usize;

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();
    let hex = format!("{:x}", hash);
    let result = if hex.len() > length { &hex[..length] } else { &hex };
    out.write(result)?;
    Ok(())
}

fn hash_int_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let input = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| RenderError::new("First parameter must be a string"))?;
    let length = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(7) as usize;

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();
    let result = hash % (10_u64.pow(length as u32));
    out.write(&result.to_string())?;
    Ok(())
}

fn prefix_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let input = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| RenderError::new("First parameter must be a string"))?;
    let length = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(input.len() as u64) as usize;

    let result = if input.len() > length { &input[..length] } else { input };
    out.write(result)?;
    Ok(())
}

fn format_timestamp_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let timestamp = h.param(0).and_then(|v| v.value().as_u64()).ok_or_else(|| RenderError::new("First parameter must be a timestamp"))?;
    let format = h.hash_get("format").and_then(|v| v.value().as_str()).unwrap_or("iso_date");

    use chrono::{DateTime, Utc, TimeZone};
    let dt = Utc.timestamp_opt(timestamp as i64, 0).single().ok_or_else(|| RenderError::new("Invalid timestamp"))?;

    let formatted = match format {
        "iso_date" => dt.format("%Y-%m-%d").to_string(),
        "iso_datetime" => dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
        "compact_date" => dt.format("%Y%m%d").to_string(),
        "compact_datetime" => dt.format("%Y%m%d%H%M%S").to_string(),
        custom => dt.format(custom).to_string(),
    };

    out.write(&formatted)?;
    Ok(())
}
```

// No utils.rs needed - use template.resolve(vars) directly
// Option handling can be done inline where needed

### Step 3: Update CLI Arguments with Template Types

**File**: `src/cli/version/args/main.rs` (update existing)

```rust
use clap::Parser;
use crate::cli::utils::template::Template;
use crate::utils::constants::{
    SUPPORTED_FORMATS_ARRAY,
    formats,
    sources,
};

/// Main configuration for input, schema, and output
#[derive(Parser)]
pub struct MainConfig {
    // ... existing fields unchanged ...

    /// Output template for custom formatting (Handlebars syntax)
    /// Examples:
    ///   "{{major}}.{{minor}}.{{patch}}"
    ///   "v{{major}}.{{minor}}.{{patch}}-{{bumped_branch}}"
    ///   "{{major}}.{{minor}}.{{patch}}+{{custom.build_id}}"
    #[arg(
        long,
        help = "Output template using Handlebars syntax for custom formatting"
    )]
    pub output_template: Option<Template<String>>, // UPDATED: was Option<String>

    // ... rest unchanged ...
}
```

**File**: `src/cli/version/args/overrides.rs` (update existing)

```rust
use clap::Parser;
use crate::cli::utils::template::{Template, IndexValue};
use crate::utils::constants::pre_release_labels;

/// Override configuration for VCS and version components
#[derive(Parser, Default)]
pub struct OverridesConfig {
    // ... VCS override options unchanged ...

    // ============================================================================
    // VERSION COMPONENT OVERRIDE OPTIONS (UPDATED WITH TEMPLATE SUPPORT)
    // ============================================================================
    /// Override major version number (supports Handlebars templating)
    /// Examples: --major 2, --major "{{add major 1}}", --major "{{custom.version}}"
    #[arg(long, help = "Override major version number (supports Handlebars templating)")]
    pub major: Option<Template<u32>>, // UPDATED: was Option<u32>

    /// Override minor version number (supports Handlebars templating)
    #[arg(long, help = "Override minor version number (supports Handlebars templating)")]
    pub minor: Option<Template<u32>>, // UPDATED: was Option<u32>

    /// Override patch version number (supports Handlebars templating)
    #[arg(long, help = "Override patch version number (supports Handlebars templating)")]
    pub patch: Option<Template<u32>>, // UPDATED: was Option<u32>

    /// Override epoch number (supports Handlebars templating)
    #[arg(long, help = "Override epoch number (supports Handlebars templating)")]
    pub epoch: Option<Template<u32>>, // UPDATED: was Option<u32>

    /// Override post number (supports Handlebars templating)
    #[arg(long, help = "Override post number (supports Handlebars templating)")]
    pub post: Option<Template<u32>>, // UPDATED: was Option<u32>

    /// Override dev number (supports Handlebars templating)
    #[arg(long, help = "Override dev number (supports Handlebars templating)")]
    pub dev: Option<Template<u32>>, // UPDATED: was Option<u32>

    /// Override pre-release number (supports Handlebars templating)
    #[arg(long, help = "Override pre-release number (supports Handlebars templating)")]
    pub pre_release_num: Option<Template<u32>>, // UPDATED: was Option<u32>

    // ... other fields unchanged ...

    // ============================================================================
    // SCHEMA COMPONENT OVERRIDE OPTIONS (UPDATED WITH TEMPLATE SUPPORT)
    // ============================================================================
    /// Override core schema component by index=value (VALUE supports Handlebars)
    /// Examples:
    ///   --core "0={{major}}"  (use current major)
    ///   --core "1={{bumped_branch}}" (use current branch)
    ///   --core "2=v{{major}}.{{minor}}" (template in value)
    #[arg(
        long,
        value_name = "INDEX=VALUE",
        num_args = 1..,
        help = "Override core schema component by index=value (VALUE supports Handlebars templating)"
    )]
    pub core: Vec<IndexValue>, // UPDATED: was Vec<String>

    /// Override extra-core schema component by index=value (VALUE supports Handlebars)
    #[arg(
        long,
        value_name = "INDEX=VALUE",
        num_args = 1..,
        help = "Override extra-core schema component by index=value (VALUE supports Handlebars templating)"
    )]
    pub extra_core: Vec<IndexValue>, // UPDATED: was Vec<String>

    /// Override build schema component by index=value (VALUE supports Handlebars)
    #[arg(
        long,
        value_name = "INDEX=VALUE",
        num_args = 1..,
        help = "Override build schema component by index=value (VALUE supports Handlebars templating)"
    )]
    pub build: Vec<IndexValue>, // UPDATED: was Vec<String>
}
```

### Step 4: Update Bump Arguments with Template Types

**File**: `src/cli/version/args/bumps.rs` (update existing)

```rust
use clap::Parser;
use crate::cli::utils::template::Template;
use crate::utils::constants::pre_release_labels;

/// Bump configuration for field-based and schema-based version bumping
#[derive(Parser, Default)]
pub struct BumpsConfig {
    // ============================================================================
    // FIELD-BASED BUMP OPTIONS (UPDATED WITH TEMPLATE SUPPORT)
    // ============================================================================
    /// Add to major version (supports Handlebars templating, default: 1)
    /// Examples: --bump-major, --bump-major 2, --bump-major "{{distance}}"
    #[arg(long, help = "Add to major version (supports Handlebars templating, default: 1)")]
    pub bump_major: Option<Option<Template<u32>>>, // UPDATED: was Option<Option<u32>>

    /// Add to minor version (supports Handlebars templating, default: 1)
    #[arg(long, help = "Add to minor version (supports Handlebars templating, default: 1)")]
    pub bump_minor: Option<Option<Template<u32>>>, // UPDATED: was Option<Option<u32>>

    /// Add to patch version (supports Handlebars templating, default: 1)
    #[arg(long, help = "Add to patch version (supports Handlebars templating, default: 1)")]
    pub bump_patch: Option<Option<Template<u32>>>, // UPDATED: was Option<Option<u32>>

    /// Add to post number (supports Handlebars templating, default: 1)
    #[arg(long, help = "Add to post number (supports Handlebars templating, default: 1)")]
    pub bump_post: Option<Option<Template<u32>>>, // UPDATED: was Option<Option<u32>>

    /// Add to dev number (supports Handlebars templating, default: 1)
    #[arg(long, help = "Add to dev number (supports Handlebars templating, default: 1)")]
    pub bump_dev: Option<Option<Template<u32>>>, // UPDATED: was Option<Option<u32>>

    /// Add to pre-release number (supports Handlebars templating, default: 1)
    #[arg(long, help = "Add to pre-release number (supports Handlebars templating, default: 1)")]
    pub bump_pre_release_num: Option<Option<Template<u32>>>, // UPDATED: was Option<Option<u32>>

    /// Add to epoch number (supports Handlebars templating, default: 1)
    #[arg(long, help = "Add to epoch number (supports Handlebars templating, default: 1)")]
    pub bump_epoch: Option<Option<Template<u32>>>, // UPDATED: was Option<Option<u32>>

    // ... rest unchanged ...
}
```

### Step 5: Update Module Exports

**File**: `src/cli/utils/mod.rs` (update existing)

```rust
pub mod format_handler;
pub mod output_formatter;
pub mod template; // NEW: Add template module
```

**File**: `src/error.rs` (update existing)

Add template error variant:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ZervError {
    // ... existing variants ...

    #[error("Template error: {0}")]
    TemplateError(String), // NEW: Add template error
}
```

### Step 6: Pipeline Integration with Render Timing

**File**: `src/cli/version/pipeline.rs` (update existing)

```rust
use crate::cli::utils::template::Template;

pub fn run_version_pipeline(mut args: VersionArgs) -> Result<String, ZervError> {
    // 0. Early validation - fail fast on conflicting options
    args.validate()?;

    // 1. Determine working directory
    let work_dir = match args.main.directory.as_deref() {
        Some(dir) => std::path::PathBuf::from(dir),
        None => current_dir()?,
    };

    // 2. Get ZervDraft from source (no schema applied yet)
    let zerv_draft = match args.main.source.as_str() {
        sources::GIT => super::git_pipeline::process_git_source(&work_dir, &args)?,
        sources::STDIN => super::stdin_pipeline::process_stdin_source(&args)?,
        source => return Err(ZervError::UnknownSource(source.to_string())),
    };

    // 3. Convert to Zerv (EARLY RENDERING happens inside to_zerv)
    let zerv_object = zerv_draft.to_zerv(&args)?;

    // 4. Apply output formatting (LATE RENDERING for output_template)
    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.main.output_format,
        args.main.output_prefix.as_deref(),
        &args.main.output_template, // Now Option<Template<String>>
    )?;

    Ok(output)
}
```

### Step 8: Template Resolution and ResolvedArgs

**File**: `src/cli/version/args/resolved.rs` (new)

```rust
use crate::cli::utils::template::Template;
use crate::cli::version::args::{VersionArgs, MainConfig};
use crate::version::zerv::vars::ZervVars;
use crate::error::ZervError;

/// Resolved version of VersionArgs with templates rendered
pub struct ResolvedArgs {
    pub overrides: ResolvedOverrides,
    pub bumps: ResolvedBumps,
    pub main: MainConfig, // Keep entire MainConfig for simplicity
}

/// Resolved overrides with all templates rendered to values
pub struct ResolvedOverrides {
    pub major: Option<u32>,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub epoch: Option<u32>,
    pub post: Option<u32>,
    pub dev: Option<u32>,
    pub pre_release_num: Option<u32>,
    pub core: Vec<String>, // Resolved INDEX=VALUE strings
    pub extra_core: Vec<String>,
    pub build: Vec<String>,
    // ... other non-template fields unchanged
}

/// Resolved bumps with all templates rendered to values
pub struct ResolvedBumps {
    pub bump_major: Option<Option<u32>>,
    pub bump_minor: Option<Option<u32>>,
    pub bump_patch: Option<Option<u32>>,
    pub bump_epoch: Option<Option<u32>>,
    pub bump_post: Option<Option<u32>>,
    pub bump_dev: Option<Option<u32>>,
    pub bump_pre_release_num: Option<Option<u32>>,
    // ... other non-template fields unchanged
}

impl ResolvedArgs {
    /// Resolve all templates in VersionArgs using ZervVars snapshot
    pub fn resolve(args: &VersionArgs, vars: &ZervVars) -> Result<Self, ZervError> {
        let overrides = ResolvedOverrides::resolve(&args.overrides, vars)?;
        let bumps = ResolvedBumps::resolve(&args.bumps, vars)?;

        Ok(ResolvedArgs {
            overrides,
            bumps,
            main: args.main.clone(), // Keep entire MainConfig
        })
    }

    /// Delegate to main config for method access
    pub fn dirty_override(&self) -> bool {
        self.main.dirty_override()
    }
}

impl ResolvedOverrides {
    fn resolve(overrides: &OverridesConfig, vars: &ZervVars) -> Result<Self, ZervError> {
        Ok(ResolvedOverrides {
            major: Self::resolve_template(&overrides.major, vars)?,
            minor: Self::resolve_template(&overrides.minor, vars)?,
            patch: Self::resolve_template(&overrides.patch, vars)?,
            epoch: Self::resolve_template(&overrides.epoch, vars)?,
            post: Self::resolve_template(&overrides.post, vars)?,
            dev: Self::resolve_template(&overrides.dev, vars)?,
            pre_release_num: Self::resolve_template(&overrides.pre_release_num, vars)?,
            core: Self::resolve_index_values(&overrides.core, vars)?,
            extra_core: Self::resolve_index_values(&overrides.extra_core, vars)?,
            build: Self::resolve_index_values(&overrides.build, vars)?,
        })
    }

    fn resolve_template<T>(template: &Option<Template<T>>, vars: &ZervVars) -> Result<Option<T>, ZervError>
    where
        T: FromStr + Clone,
        T::Err: Display,
    {
        match template {
            Some(t) => Ok(Some(t.resolve(vars)?)),
            None => Ok(None),
        }
    }

    fn resolve_index_values(index_values: &[IndexValue], vars: &ZervVars) -> Result<Vec<String>, ZervError> {
        index_values.iter()
            .map(|iv| {
                let resolved_value = iv.value.resolve(vars)?;
                Ok(format!("{}={}", iv.index, resolved_value))
            })
            .collect()
    }
}

impl ResolvedBumps {
    fn resolve(bumps: &BumpsConfig, vars: &ZervVars) -> Result<Self, ZervError> {
        Ok(ResolvedBumps {
            bump_major: Self::resolve_bump(&bumps.bump_major, vars)?,
            bump_minor: Self::resolve_bump(&bumps.bump_minor, vars)?,
            bump_patch: Self::resolve_bump(&bumps.bump_patch, vars)?,
            bump_epoch: Self::resolve_bump(&bumps.bump_epoch, vars)?,
            bump_post: Self::resolve_bump(&bumps.bump_post, vars)?,
            bump_dev: Self::resolve_bump(&bumps.bump_dev, vars)?,
            bump_pre_release_num: Self::resolve_bump(&bumps.bump_pre_release_num, vars)?,
        })
    }

    fn resolve_bump(bump: &Option<Option<Template<u32>>>, vars: &ZervVars) -> Result<Option<Option<u32>>, ZervError> {
        match bump {
            Some(Some(template)) => Ok(Some(Some(template.resolve(vars)?))),
            Some(None) => Ok(Some(None)),
            None => Ok(None),
        }
    }
}
```

**File**: `src/version/zerv_draft.rs` (update existing)

````rust
use crate::cli::version::args::{VersionArgs, resolved::ResolvedArgs};

impl ZervDraft {
    pub fn to_zerv(mut self, args: &VersionArgs) -> Result<Zerv, ZervError> {
        // Clone vars to create snapshot (prevent order-dependent template resolution)
        let vars_snapshot = self.vars.clone();

        // Resolve ALL templates at once using same snapshot
        let resolved_args = ResolvedArgs::resolve(args, &vars_snapshot)?;

        // Apply overrides first (now uses resolved values)
        self.vars.apply_context_overrides(&resolved_args)?;

        // Then create the Zerv object (preserve existing logic)
        let (schema_name, schema_ron) = args.resolve_schema();
        let mut zerv = self.create_zerv_version(schema_name, schema_ron)?;

        // Apply component processing (bumps with reset logic) (now uses resolved values)
        zerv.apply_component_processing(&resolved_args)?;
        zerv.normalize();

        Ok(zerv)
    }

    // Keep existing create_zerv_version method unchanged
    pub fn create_zerv_version(
        self,
        schema_name: Option<&str>,
        schema_ron: Option<&str>,
    ) -> Result<Zerv, ZervError> {
        // Existing implementation unchanged...
        let schema = match (schema_name, schema_ron) {
            (None, Some(ron_str)) => parse_ron_schema(ron_str)?,
            (Some(name), None) => {
                if let Some(schema) = get_preset_schema(name, &self.vars) {
                    schema
                } else {
                    return Err(ZervError::UnknownSchema(name.to_string()));
                }
            }
            (Some(_), Some(_)) => {
                return Err(ZervError::ConflictingSchemas(
                    "Cannot specify both schema_name and schema_ron".to_string(),
                ));
            }
            (None, None) => {
                if let Some(existing_schema) = self.schema {
                    existing_schema
                } else {
                    return Err(ZervError::MissingSchema(
                        "Either schema_name or schema_ron must be provided".to_string(),
                    ));
                }
            }
        };

        Zerv::new(schema, self.vars)
    }
}

### Step 7: Update Existing Methods to Use ResolvedArgs

**File**: `src/version/zerv/vars.rs` (update existing)

```rust
use crate::cli::version::args::resolved::ResolvedArgs;

impl ZervVars {
    /// Apply context overrides using resolved template values
    /// This replaces the existing apply_context_overrides method signature
    pub fn apply_context_overrides(&mut self, args: &ResolvedArgs) -> Result<(), ZervError> {
        // Keep ALL existing override logic, just use resolved values instead of templates

        // VCS overrides (unchanged)
        if let Some(tag_version) = &args.main.tag_version {
            // Existing tag_version parsing logic...
        }

        // Template-resolved version component overrides
        if let Some(major) = args.overrides.major {
            self.major = Some(major as u64);
        }
        if let Some(minor) = args.overrides.minor {
            self.minor = Some(minor as u64);
        }
        if let Some(patch) = args.overrides.patch {
            self.patch = Some(patch as u64);
        }
        if let Some(epoch) = args.overrides.epoch {
            self.epoch = Some(epoch as u64);
        }
        if let Some(post) = args.overrides.post {
            self.post = Some(post as u64);
        }
        if let Some(dev) = args.overrides.dev {
            self.dev = Some(dev as u64);
        }
        if let Some(pre_release_num) = args.overrides.pre_release_num {
            if let Some(pre_release) = &mut self.pre_release {
                pre_release.number = Some(pre_release_num as u64);
            }
        }

        // Template-resolved schema component overrides
        // args.overrides.core/extra_core/build are now Vec<String> with resolved INDEX=VALUE
        // Keep existing parsing and application logic...

        // All other existing override logic unchanged...

        Ok(())
    }
}
````

**File**: `src/version/zerv/bump/mod.rs` (update existing)

```rust
use crate::cli::version::args::resolved::ResolvedArgs;

impl Zerv {
    /// Apply component processing using resolved template values
    /// This replaces the existing apply_component_processing method signature
    pub fn apply_component_processing(&mut self, args: &ResolvedArgs) -> Result<(), ZervError> {
        // Keep ALL existing component processing logic, just use resolved values

        // Template-resolved field-based bumps
        if let Some(Some(bump_value)) = args.bumps.bump_major {
            self.vars.major = Some(self.vars.major.unwrap_or(0) + bump_value as u64);
        } else if let Some(None) = args.bumps.bump_major {
            self.vars.major = Some(self.vars.major.unwrap_or(0) + 1);
        }
        // Continue for all bump fields...

        // Access MainConfig fields as before (unchanged)
        if args.main.no_bump_context {
            // Keep existing no_bump_context logic
        }

        // Keep ALL existing precedence order logic
        // Keep ALL existing reset logic
        // Keep ALL existing schema-based bump logic

        Ok(())
    }
}
```

### Step 8: Update Output Formatter

**File**: `src/cli/utils/output_formatter.rs` (update existing)

```rust
use crate::cli::utils::template::Template;

impl OutputFormatter {
    pub fn format_output(
        zerv: &Zerv,
        output_format: &str,
        output_prefix: Option<&str>,
        output_template: &Option<Template<String>>, // UPDATED: was &Option<String>
    ) -> Result<String, ZervError> {
        // Handle template rendering for output
        let formatted = if let Some(template) = output_template {
            // LATE RENDERING: Use final processed ZervVars
            template.resolve(&zerv.vars)?
        } else {
            // Standard format conversion
            match output_format {
                formats::SEMVER => {
                    let semver = SemVer::from(zerv.clone());
                    semver.to_string()
                }
                formats::PEP440 => {
                    let pep440 = PEP440::from(zerv.clone());
                    pep440.to_string()
                }
                formats::ZERV => {
                    ron::to_string_pretty(zerv, ron::PrettyConfig::default())
                        .map_err(|e| ZervError::SerializationError(format!("RON serialization failed: {}", e)))?
                }
                _ => return Err(ZervError::UnsupportedFormat(output_format.to_string())),
            }
        };

        // Apply prefix if specified
        let result = if let Some(prefix) = output_prefix {
            format!("{}{}", prefix, formatted)
        } else {
            formatted
        };

        Ok(result)
    }
}
```

## Template Features (Phase 1)

### Available Template Variables

Based on the implemented `TemplateContext`:

**Core Version Fields:**

- `{{major}}`, `{{minor}}`, `{{patch}}` - Core version numbers
- `{{epoch}}` - PEP440 epoch number

**Metadata Fields:**

- `{{post}}` - Post-release number
- `{{dev}}` - Development number

**Pre-release Fields:**

- `{{pre_release.label}}` - Pre-release label (alpha, beta, rc)
- `{{pre_release.number}}` - Pre-release number

**VCS Fields:**

- `{{distance}}` - Commits since tag
- `{{dirty}}` - Working tree state (true/false)
- `{{bumped_branch}}` - Current branch name
- `{{bumped_commit_hash}}` - Full commit hash
- `{{bumped_commit_hash_short}}` - Short commit hash (7 chars)
- `{{bumped_timestamp}}` - Current commit timestamp

**Last Version Fields:**

- `{{last_branch}}` - Branch where last version was created
- `{{last_commit_hash}}` - Last version commit hash
- `{{last_commit_hash_short}}` - Short last commit hash
- `{{last_timestamp}}` - Last version creation timestamp

**Custom Variables:**

- `{{custom.field_name}}` - Access custom JSON fields
- `{{custom.build_id}}` - Example: build identifier
- `{{custom.environment}}` - Example: deployment environment

**Formatted Versions:**

- `{{pep440}}` - Complete PEP440 formatted version string
- `{{semver}}` - Complete SemVer formatted version string

### Custom Handlebars Helpers

**Math Helpers:**

- `{{add a b}}` - Addition (a + b)
- `{{subtract a b}}` - Subtraction (a - b)
- `{{multiply a b}}` - Multiplication (a \* b)

**String Helpers:**

- `{{hash input [length]}}` - Generate hex hash (default: 7 chars)
- `{{hash_int input [length]}}` - Generate integer hash
- `{{prefix string [length]}}` - Get prefix of string to length

**Timestamp Helpers:**

- `{{format_timestamp timestamp format=format_string}}` - Format timestamp

## Key Architectural Decisions

### ResolvedArgs Structure

**Decision:** Keep `pub main: MainConfig` instead of extracting individual fields.

**Benefits:**

- **Maintains existing schema** - No need to identify which specific fields are used
- **Simpler implementation** - Methods can continue accessing `args.main.input_format`, etc.
- **Future-proof** - New MainConfig fields automatically available
- **Less code changes** - Existing method signatures remain unchanged

**Implementation:**

```rust
/// Resolved version of VersionArgs with templates rendered
pub struct ResolvedArgs {
    pub overrides: ResolvedOverrides,
    pub bumps: ResolvedBumps,
    pub main: MainConfig, // Keep entire MainConfig for simplicity
}

impl ResolvedArgs {
    /// Delegate to main config for method access
    pub fn dirty_override(&self) -> bool {
        self.main.dirty_override()
    }
}
```

### Unified Early Rendering

**Problem Solved:** Ensures all early template rendering (overrides + bumps) happens at exactly the same time with the same `ZervVars` snapshot.

**Implementation:**

```rust
fn apply_early_template_rendering(&self, args: &VersionArgs, vars: &ZervVars) -> Result<Zerv, ZervError> {
    let mut zerv = self.to_base_zerv();

    // Both use identical snapshot - guaranteed consistency
    self.apply_overrides(&mut zerv, args, vars)?;
    self.apply_bumps(&mut zerv, args, vars)?;

    Ok(zerv)
}
```

**Benefits:**

- **Predictable behavior** - All templates see identical context
- **No order dependency** - Overrides don't affect bump calculations
- **Single snapshot** - Eliminates timing-related inconsistencies
- **Atomic operation** - All early rendering happens together

### Two-Phase Template Rendering

**Early Phase (before version processing):**

- **Context:** VCS state + base version from tag
- **Used for:** All overrides and bumps
- **Examples:** `--major "{{distance}}"`, `--bump-patch "{{dev}}"`

**Late Phase (after version processing):**

- **Context:** VCS state + fully computed final version
- **Used for:** Output template only
- **Examples:** `--output-template "v{{major}}.{{minor}}.{{patch}}"`

### Schema Override Simplification

**Approach:** Render entire `INDEX=VALUE` string as template
**Benefits:**

- **Keeps existing `Vec<String>` structure** - No CLI changes needed
- **Maximum flexibility** - Can template both index and value
- **Simple implementation** - Single template resolution per override

**Examples:**

````bash
zerv --core "0={{major}}"           # Template in value
zerv --core "{{custom.index}}=release" # Template in index
zerv --core "{{idx}}={{major}}.{{minor}}" # Template in both
```}}` - Working tree state (true/false)
- `{{bumped_branch}}` - Current branch name
- `{{bumped_commit_hash}}` - Full commit hash
- `{{bumped_commit_hash_short}}` - Short commit hash (7 chars)
- `{{bumped_timestamp}}` - Current commit timestamp

**Last Version Fields:**

- `{{last_branch}}` - Branch where last version was created
- `{{last_commit_hash}}` - Last version commit hash
- `{{last_commit_hash_short}}` - Short last commit hash
- `{{last_timestamp}}` - Last version creation timestamp

**Custom Variables:**

- `{{custom.field_name}}` - Access custom JSON fields
- `{{custom.build_id}}` - Example: build identifier
- `{{custom.environment}}` - Example: deployment environment

**Formatted Versions:**

- `{{pep440}}` - Complete PEP440 formatted version string
- `{{semver}}` - Complete SemVer formatted version string

### Custom Handlebars Helpers

**Math Helpers:**

- `{{add a b}}` - Addition (a + b)
- `{{subtract a b}}` - Subtraction (a - b)
- `{{multiply a b}}` - Multiplication (a \* b)

**String Helpers:**

- `{{hash input [length]}}` - Generate hex hash (default: 7 chars)
- `{{hash_int input [length]}}` - Generate integer hash
- `{{prefix string [length]}}` - Get prefix of string to length

**Timestamp Helpers:**

- `{{format_timestamp timestamp format=format_string}}` - Format timestamp

**Pre-defined Format Variables:**

- `iso_date` - ISO date format (`%Y-%m-%d`) → "2023-12-21"
- `iso_datetime` - ISO datetime format (`%Y-%m-%dT%H:%M:%S`) → "2023-12-21T12:34:56"
- `compact_date` - Compact date format (`%Y%m%d`) → "20231221"
- `compact_datetime` - Compact datetime format (`%Y%m%d%H%M%S`) → "20231221123456"

## Render Timing Control

### EARLY RENDERING (before version processing)

**Context**: VCS state + base version from tag
**Used for**: All overrides and bumps
**Available vars**: `distance`, `dirty`, `bumped_*`, `last_*`, base version fields (`major`, `minor`, etc. from tag)

**Examples:**
```bash
# Use VCS distance to set patch bump
zerv --bump-patch "{{distance}}"

# Use previous dev number as distance override
zerv --distance "{{dev}}"

# Use branch name in pre-release label
zerv --bump-pre-release-label "{{bumped_branch}}"
````

### LATE RENDERING (after version processing)

**Context**: VCS state + fully computed final version
**Used for**: Output template only
**Available vars**: All early vars PLUS computed version fields, formatted versions

**Examples:**

```bash
# Output with computed version
zerv --output-template "v{{major}}.{{minor}}.{{patch}}"

# Complex output with VCS and version data
zerv --output-template "{{semver}}+build.{{bumped_commit_hash_short}}"
```

## Implementation Notes

- **No duplicate functions**: Use `template.resolve(vars)` directly
- **Timing controlled by context**: Which `ZervVars` object is passed
- **No circular dependencies**: Overrides use historical data as input schema component overrides/bumps
  **Template Variables**: Current state before any bumping

```rust
// These render EARLY using current Zerv context:
pub major: Option<Template<u32>>,         // Field override
pub core: Vec<IndexValue>,                // Schema override (IndexValue.value is Template<String>)
pub bump_major: Option<Option<Template<u32>>>, // Bump values
```

### LATE RENDERING (after component processing)

**Context**: Final processed Zerv object with all bumps applied
**Used for**: Output formatting
**Template Variables**: Final state after all processing

```rust
// This renders LATE using final Zerv object:
pub output_template: Option<Template<String>>, // Output format
```

## Usage Examples

### Field Override Templates (EARLY RENDERING)

```bash
# Override version fields with templates
zerv version --major "{{add major 1}}" --minor "{{custom.target_minor}}"
# Renders templates using current context, then applies overrides

# Override with current VCS context
zerv version --dev "{{distance}}" --post "{{add distance 10}}"
# Uses current distance value in templates

# Complex field templates
zerv version --patch "{{multiply minor 10}}" --epoch "{{custom.release_year}}"
# Mathematical operations and custom variables
```

### Schema Component Override Templates

```bash
# Schema component overrides with templates
zerv version --core "0={{major}}" --core "1={{bumped_branch}}"
# Use current major version and branch name in schema

# Complex schema templates
zerv version --extra-core "0={{add post 1}}" --build "0={{hash bumped_commit_hash 8}}"
# Mathematical operations and hash generation
```

### Bump Templates (EARLY RENDERING)

```bash
# Bump with template values
zerv version --bump-major "{{distance}}" --bump-minor "{{custom.increment}}"
# Use VCS distance and custom increment values

# Conditional bumps
zerv version --bump-patch "{{#if dirty}}10{{else}}1{{/if}}"
# Different bump values based on dirty state
```

### Output Templates (LATE RENDERING)

```bash
# Custom output format after all processing
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{bumped_branch}}"
# Renders using final processed Zerv object

# Complex output with helpers
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}+{{hash bumped_commit_hash 7}}"
# Use hash helper for commit hash

# Output with custom variables and timestamps
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{custom.build_id}}.{{format_timestamp bumped_timestamp format=compact_date}}"
# Access to all final variables and computed fields
```

## Testing Strategy

### Unit Tests

**File**: `src/cli/utils/template.rs` (tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::ZervFixture;

    #[test]
    fn test_template_value_resolution() {
        let template = Template::Value(42u32);
        let zerv = ZervFixture::new().build();
        assert_eq!(template.resolve(&zerv.vars).unwrap(), 42);
    }

    #[test]
    fn test_template_string_resolution() {
        let template = Template::Template("{{major}}.{{minor}}.{{patch}}".to_string());
        let zerv = ZervFixture::new().with_version(1, 2, 3).build();
        assert_eq!(template.resolve(&zerv.vars).unwrap(), "1.2.3");
    }

    #[test]
    fn test_template_helpers() {
        let template = Template::Template("{{add major minor}}".to_string());
        let zerv = ZervFixture::new().with_version(1, 2, 3).build();
        assert_eq!(template.resolve(&zerv.vars).unwrap(), "3");
    }

    #[test]
    fn test_index_value_parsing() {
        let index_value: IndexValue = "0={{major}}".parse().unwrap();
        assert_eq!(index_value.index, 0);
        assert_eq!(index_value.value, Template::Template("{{major}}".to_string()));
    }

    #[test]
    fn test_template_context_creation() {
        let zerv = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_branch("main".to_string())
            .build();

        let context = TemplateContext::from_zerv_vars(&zerv.vars);
        assert_eq!(context.major, Some(1));
        assert_eq!(context.minor, Some(2));
        assert_eq!(context.patch, Some(3));
        assert_eq!(context.bumped_branch, Some("main".to_string()));
    }
}
```

### Integration Tests

**File**: `tests/integration_tests/version/template_integration.rs` (new)

```rust
use zerv::test_utils::*;

#[test]
fn test_template_override_integration() {
    let temp_dir = TempDir::new().unwrap();
    let git_repo = GitRepo::new(&temp_dir).with_initial_commit().with_tag("v1.0.0");

    let output = git_repo.zerv_version(&[
        "--major", "{{add major 1}}",
        "--output-template", "v{{major}}.{{minor}}.{{patch}}"
    ]).unwrap();

    assert_eq!(output.trim(), "v2.0.0");
}

#[test]
fn test_template_bump_integration() {
    let temp_dir = TempDir::new().unwrap();
    let git_repo = GitRepo::new(&temp_dir).with_initial_commit().with_tag("v1.0.0");

    let output = git_repo.zerv_version(&[
        "--bump-major", "{{distance}}",
        "--output-template", "{{major}}.{{minor}}.{{patch}}"
    ]).unwrap();

    // Should bump major by distance (0 in this case)
    assert_eq!(output.trim(), "1.0.0");
}
```

## Migration Strategy

### Phase 1: Add Template Infrastructure

1. Add handlebars dependency
2. Implement template types and helpers
3. Add template module exports

### Phase 2: Update CLI Arguments

1. Update MainConfig.output_template type
2. Update OverridesConfig field types
3. Update BumpsConfig field types
4. Update IndexValue to support templates

### Phase 3: Pipeline Integration

1. Add early rendering for overrides/bumps
2. Add late rendering for output templates
3. Update output formatter
4. Add error handling

### Phase 4: Testing and Documentation

1. Add comprehensive unit tests
2. Add integration tests
3. Update CLI help text
4. Add usage examples

## Success Criteria

- ✅ Template types replace primitive types in CLI arguments
- ✅ Handlebars templating works with variable substitution
- ✅ Custom helpers implemented and functional
- ✅ Early vs late rendering timing works correctly
- ✅ All existing functionality preserved
- ✅ Template validation and error handling
- ✅ Comprehensive test coverage
- ✅ Clean integration with existing codebase

## Benefits

1. **Dynamic Versioning**: Templates enable dynamic version generation based on VCS state
2. **Flexible Output**: Custom output templates for any format requirement
3. **Mathematical Operations**: Built-in helpers for version calculations
4. **Context Awareness**: Access to all VCS and custom variables
5. **Render Timing Control**: Early rendering for processing, late rendering for output
6. **Type Safety**: Template types maintain type safety while adding flexibility
7. **Backward Compatibility**: Existing literal values continue to work
8. **Extensible**: Easy to add new helpers and template variables

This updated plan leverages the solid foundation from Plans 19-24 and provides a comprehensive templating system that aligns with the ideal specification from Plan 11.
