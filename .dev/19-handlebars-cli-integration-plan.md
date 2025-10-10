# Handlebars CLI Integration Plan

## Prerequisites

**CRITICAL**: This plan must be implemented **AFTER** Plan 20 (Component VarField Enum Refactoring) is completed.

**Reason**: Template validation requires the `Var` enum from Plan 20 for field name validation and type safety.

## Problem

Multiple CLI arguments need Handlebars templating support with **render-at-use** timing:

**1. Early Rendering** (before `apply_component_processing`):

- `--dev <TEMPLATE>` - Dev number override
- `--post <TEMPLATE>` - Post number override
- `--major <TEMPLATE>` - Major version override
- `--minor <TEMPLATE>` - Minor version override
- `--patch <TEMPLATE>` - Patch version override
- `--epoch <TEMPLATE>` - Epoch override
- `--pre-release-num <TEMPLATE>` - Pre-release number override
- Schema component overrides: `--core`, `--extra-core`, `--build`
- Schema component bumps: `--bump-core`, `--bump-extra-core`, `--bump-build`

**2. Late Rendering** (after `apply_component_processing`):

- `--output-template <TEMPLATE>` - Final output formatting

**Current Implementation Issues:**

- All args use `Option<u32>` or `Vec<String>` - no templating support
- No type-level indication of template capability
- No render-at-use timing control
- Missing handlebars dependency

## Solution: Template Types with Render-at-Use

### Core Concept

Replace primitive types with template-aware types that render when used:

```rust
// Before
pub dev: Option<u32>,

// After
pub dev: Option<Template<u32>>,
```

Each template type knows **when** to render based on **where** it's used in the pipeline.

### Step 1: Add Handlebars Dependency

**File**: `Cargo.toml`

```toml
[dependencies]
handlebars = "4.4"
# serde_json already exists for template context serialization
```

### Step 2: Template Types

**File**: `src/cli/utils/template.rs` (following existing utils pattern)

```rust
use std::str::FromStr;
use std::fmt::Display;
use handlebars::Handlebars;
use serde_json::Value;
use crate::version::zerv::{Zerv, components::Var};
use crate::error::ZervError;

/// Template-aware type that can hold a direct value or template string
#[derive(Debug, Clone, PartialEq)]
pub enum Template<T> {
    Value(T),
    Template(String), // Handlebars template string
}

impl<T> Template<T>
where
    T: FromStr,
    T::Err: Display,
{
    /// Resolve template using Zerv context, return final value
    pub fn resolve(&self, zerv: &Zerv) -> Result<T, ZervError> {
        match self {
            Template::Value(v) => Ok(v.clone()),
            Template::Template(template) => {
                // Validate template variables against Var enum before rendering
                Self::validate_template_variables(template)?;
                let rendered = Self::render_template(template, zerv)?;
                let parsed = rendered.parse::<T>()
                    .map_err(|e| ZervError::TemplateError(format!("Failed to parse '{}': {}", rendered, e)))?;
                Ok(parsed)
            }
        }
    }

    /// Render Handlebars template using Zerv object as context
    fn render_template(template: &str, zerv: &Zerv) -> Result<String, ZervError> {
        let mut handlebars = Handlebars::new();

        // Register custom Zerv helpers
        Self::register_custom_helpers(&mut handlebars)?;

        // Create template context with vars + formatted versions + custom data
        let template_context = zerv.create_template_context();
        let context = serde_json::to_value(template_context)
            .map_err(|e| ZervError::TemplateError(format!("Serialization error: {}", e)))?;

        handlebars.render_template(template, &context)
            .map_err(|e| ZervError::TemplateError(format!("Template render error: {}", e)))
    }

    /// Validate template variables against Var enum (requires Plan 20)
    fn validate_template_variables(template: &str) -> Result<(), ZervError> {
        // TODO: Implement after Plan 20 - validate template variables against Var enum
        // This ensures template variables like {{major}}, {{bumped_branch}} are valid
        Ok(())
    }

    /// Register custom Zerv helpers for Handlebars
    fn register_custom_helpers(handlebars: &mut Handlebars) -> Result<(), ZervError> {
        // TODO: Implement custom helpers: add, subtract, multiply, hash, sanitize
        Ok(())
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



/// INDEX=VALUE pair for schema component arguments
#[derive(Debug, Clone, PartialEq)]
pub struct IndexValue {
    pub index: usize,
    pub value: String,
}

impl FromStr for IndexValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (index_str, value_str) = s.split_once('=')
            .ok_or_else(|| format!("Invalid INDEX=VALUE format: {}", s))?;

        let index = index_str.parse::<usize>()
            .map_err(|_| format!("Invalid index: {}", index_str))?;

        Ok(IndexValue {
            index,
            value: value_str.to_string(),
        })
    }
}


```

### Step 3: Update CLI Arguments with Template Types

**File**: `src/cli/version/args/main.rs` (Update existing)

```rust
// Add to existing imports
use crate::cli::utils::template::Template;

/// Output template for custom formatting (Handlebars syntax)
/// Examples:
///   "{{major}}.{{minor}}.{{patch}}"
///   "v{{major}}.{{minor}}.{{patch}}-{{bumped_branch}}"
///   "{{major}}.{{minor}}.{{patch}}+{{custom.build_id}}"
#[arg(
    long,
    help = "Output template using Handlebars syntax for custom formatting"
)]
pub output_template: Option<Template<String>>, // MIGRATION: was Option<String>
```

**File**: `src/cli/version/args/overrides.rs` (Migration from existing)

```rust
// Add to existing imports
use crate::cli::utils::template::{Template, IndexValue};

/// Override major version number (supports Handlebars templating)
/// Examples: --major 2, --major "{{add major 1}}", --major "{{custom.version}}"
#[arg(long, help = "Override major version number (supports Handlebars templating)")]
pub major: Option<Template<u32>>, // MIGRATION: was Option<u32>

/// Override minor version number (supports Handlebars templating)
#[arg(long, help = "Override minor version number (supports Handlebars templating)")]
pub minor: Option<Template<u32>>, // MIGRATION: was Option<u32>

/// Override patch version number (supports Handlebars templating)
#[arg(long, help = "Override patch version number (supports Handlebars templating)")]
pub patch: Option<Template<u32>>, // MIGRATION: was Option<u32>

/// Override dev number (supports Handlebars templating)
#[arg(long, help = "Override dev number (supports Handlebars templating)")]
pub dev: Option<Template<u32>>, // MIGRATION: was Option<u32>

/// Override post number (supports Handlebars templating)
#[arg(long, help = "Override post number (supports Handlebars templating)")]
pub post: Option<Template<u32>>, // MIGRATION: was Option<u32>

/// Override epoch number (supports Handlebars templating)
#[arg(long, help = "Override epoch number (supports Handlebars templating)")]
pub epoch: Option<Template<u32>>, // MIGRATION: was Option<u32>

/// Override pre-release number (supports Handlebars templating)
#[arg(long, help = "Override pre-release number (supports Handlebars templating)")]
pub pre_release_num: Option<Template<u32>>, // MIGRATION: was Option<u32>

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
pub core: Vec<Template<IndexValue>>, // MIGRATION: was Vec<String>

/// Override extra-core schema component by index=value (VALUE supports Handlebars)
#[arg(
    long,
    value_name = "INDEX=VALUE",
    num_args = 1..,
    help = "Override extra-core schema component by index=value (VALUE supports Handlebars templating)"
)]
pub extra_core: Vec<Template<IndexValue>>, // MIGRATION: was Vec<String>

/// Override build schema component by index=value (VALUE supports Handlebars)
#[arg(
    long,
    value_name = "INDEX=VALUE",
    num_args = 1..,
    help = "Override build schema component by index=value (VALUE supports Handlebars templating)"
)]
pub build: Vec<Template<IndexValue>>, // MIGRATION: was Vec<String>
```

**File**: `src/cli/version/args/bumps.rs` (Create new file)

```rust
use clap::Parser;
use crate::cli::utils::template::{Template, IndexValue};

/// Add to major version (supports Handlebars templating, default: 1)
#[arg(long, help = "Add to major version (supports Handlebars templating, default: 1)")]
pub bump_major: Option<Template<u32>>, // NEW: bump functionality

/// Add to minor version (supports Handlebars templating, default: 1)
#[arg(long, help = "Add to minor version (supports Handlebars templating, default: 1)")]
pub bump_minor: Option<Template<u32>>, // NEW: bump functionality

/// Add to patch version (supports Handlebars templating, default: 1)
#[arg(long, help = "Add to patch version (supports Handlebars templating, default: 1)")]
pub bump_patch: Option<Template<u32>>, // NEW: bump functionality

/// Add to dev number (supports Handlebars templating, default: 1)
#[arg(long, help = "Add to dev number (supports Handlebars templating, default: 1)")]
pub bump_dev: Option<Template<u32>>, // NEW: bump functionality

/// Add to post number (supports Handlebars templating, default: 1)
#[arg(long, help = "Add to post number (supports Handlebars templating, default: 1)")]
pub bump_post: Option<Template<u32>>, // NEW: bump functionality

/// Bump core schema component by index[=value] (VALUE supports Handlebars)
/// Examples:
///   --bump-core "0=5"           (literal value)
///   --bump-core "1={{major}}"   (use current major)
///   --bump-core "2={{bumped_branch}}-{{patch}}" (template)
#[arg(
    long,
    value_name = "INDEX[=VALUE]",
    num_args = 1..,
    help = "Bump core schema component by index[=value] (VALUE supports Handlebars templating)"
)]
pub bump_core: Vec<Template<IndexValue>>, // NEW: bump functionality

/// Bump extra-core schema component by index[=value] (VALUE supports Handlebars)
#[arg(
    long,
    value_name = "INDEX[=VALUE]",
    num_args = 1..,
    help = "Bump extra-core schema component by index[=value] (VALUE supports Handlebars templating)"
)]
pub bump_extra_core: Vec<Template<IndexValue>>, // NEW: bump functionality

/// Bump build schema component by index[=value] (VALUE supports Handlebars)
#[arg(
    long,
    value_name = "INDEX[=VALUE]",
    num_args = 1..,
    help = "Bump build schema component by index[=value] (VALUE supports Handlebars templating)"
)]
pub bump_build: Vec<Template<IndexValue>>, // NEW: bump functionality
```

### Step 4: Clap Integration

No custom parsers needed! Clap automatically uses `FromStr` implementation for `Template<T>` types.

### Step 5: Pipeline Integration with Render-at-Use

**File**: `src/cli/version/pipeline.rs` (Updated)

```rust
// Current pipeline remains mostly the same, but template rendering happens at use points:

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

    // 3. Convert to Zerv (applies overrides internally)
    // EARLY RENDERING happens here in apply_context_overrides
    let zerv_object = zerv_draft.to_zerv(&args)?;

    // 4. Apply output formatting with enhanced options
    // LATE RENDERING happens here for output_template
    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.main.output_format,
        args.main.output_prefix.as_deref(),
        &args.main.output_template, // Now Option<Template<String>>
    )?;

    Ok(output)
}
```

**File**: `src/version/zerv/vars.rs` (Updated apply_context_overrides)

```rust
/// Apply all CLI overrides to ZervVars including VCS and version components
/// EARLY RENDERING: Templates are rendered here using current Zerv context
pub fn apply_context_overrides(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
    // Apply VCS-level overrides first
    self.apply_vcs_overrides(args)?;

    // Apply clean flag (overrides VCS settings if specified)
    self.apply_clean_flag(args)?;

    // Apply version-specific field overrides
    self.apply_tag_version_overrides(args)?;

    // EARLY RENDERING: Apply templated field overrides
    self.apply_templated_field_overrides(args)?;

    // Apply context control logic
    self.apply_context_control(args)?;

    Ok(())
}

/// Apply templated field overrides (EARLY RENDERING)
fn apply_templated_field_overrides(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
    // Create template context from current vars
    let template_context = Zerv::create_template_context(self);

    // Render and apply individual field overrides
    if let Some(major_template) = &args.overrides.major {
        let major_value = major_template.resolve(&template_context)?;
        self.major = Some(major_value as u64);
    }
    if let Some(minor_template) = &args.overrides.minor {
        let minor_value = minor_template.resolve(&template_context)?;
        self.minor = Some(minor_value as u64);
    }
    if let Some(patch_template) = &args.overrides.patch {
        let patch_value = patch_template.resolve(&template_context)?;
        self.patch = Some(patch_value as u64);
    }
    if let Some(dev_template) = &args.overrides.dev {
        let dev_value = dev_template.resolve(&template_context)?;
        self.dev = Some(dev_value as u64);
    }
    if let Some(post_template) = &args.overrides.post {
        let post_value = post_template.resolve(&template_context)?;
        self.post = Some(post_value as u64);
    }
    if let Some(epoch_template) = &args.overrides.epoch {
        let epoch_value = epoch_template.resolve(&template_context)?;
        self.epoch = Some(epoch_value as u64);
    }
    if let Some(pre_num_template) = &args.overrides.pre_release_num {
        let pre_num_value = pre_num_template.resolve(&template_context)?;
        // Update existing pre_release or create new one
        match &mut self.pre_release {
            Some(pre) => pre.number = Some(pre_num_value as u64),
            None => self.pre_release = Some(PreReleaseVar {
                label: PreReleaseLabel::Alpha, // Default
                number: Some(pre_num_value as u64),
            }),
        }
    }

    Ok(())
}
```

**File**: `src/cli/utils/output_formatter.rs` (Updated)

```rust
/// Format the Zerv object according to the specified output format and options
pub fn format_output(
    zerv_object: &Zerv,
    output_format: &str,
    output_prefix: Option<&str>,
    output_template: &Option<Template<String>>, // Now Option<Template<String>>
) -> Result<String, ZervError> {
    // 1. Generate base output according to format
    let mut output = Self::format_base_output(zerv_object, output_format)?;

    // 2. LATE RENDERING: Apply template if specified
    if let Some(template) = output_template {
        let template_str = template.resolve(zerv_object)?;
        output = template_str; // Template completely replaces base output
    }

    // 3. Apply prefix if specified
    if let Some(prefix) = output_prefix {
        output = format!("{prefix}{output}");
    }

    Ok(output)
}
```

### Step 6: Error Handling Integration

**File**: `src/error.rs` (Add to existing ZervError)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ZervError {
    // ... existing variants ...

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}
```

**File**: `src/cli/utils/mod.rs` (Update existing)

```rust
// Add to existing mod.rs
pub mod template;
```

## Usage Examples

### 1. Field Override Templates (EARLY RENDERING)

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

### 2. Schema Component Templates (EARLY RENDERING)

```bash
# Override schema components with templates
zerv version --core "1={{bumped_branch}}" --core "2=build-{{major}}"
# Templates render using current Zerv context

# Bump schema components with templates
zerv version --bump-core "0={{major}}" --bump-build "1={{bumped_branch}}-{{patch}}"
# Current version values used in bump templates

# Mixed literal and template values
zerv version --core "0=5" --core "1={{bumped_branch}}" --core "2={{custom.env}}"
# Combines literal values with templated values
```

### 3. Output Templates (LATE RENDERING)

```bash
# Custom output format after all processing
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{bumped_branch}}"
# Renders using final processed Zerv object

# Complex output with conditionals
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}{{#if pre_release}}-{{pre_release}}{{/if}}"
# Conditional rendering based on final state

# Output with custom variables
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{custom.build_id}}.{{bumped_commit_hash_short}}"
# Access to all final variables and computed fields
```

### 4. Render Timing Examples

```bash
# EARLY: Field override uses current context
zerv version --major "{{add major 1}}" --output-template "Final: {{major}}"
# major template renders with current major (e.g., 1 -> 2)
# output template renders with final major (2)
# Output: "Final: 2"

# EARLY: Schema override, LATE: Output template
zerv version --core "0={{major}}" --bump-major --output-template "{{major}}.{{minor}}.{{patch}}"
# core[0] gets current major value (e.g., 1)
# bump-major increments to 2
# output template sees final major (2)
# But core[0] still has the original value (1)

# Template context timing
zerv version --dev "{{distance}}" --bump-dev --output-template "dev={{dev}}"
# dev override uses current distance (e.g., 5)
# bump-dev adds to overridden value (5 + 1 = 6)
# output template sees final dev value (6)
```

### 5. Advanced Template Features

```bash
# Mathematical helpers
zerv version --major "{{add major 1}}" --minor "{{multiply minor 2}}"
# Built-in math operations

# Conditional logic
zerv version --output-template "{{major}}.{{minor}}.{{patch}}{{#if (gt distance 0)}}.post{{distance}}{{/if}}"
# Conditional post-release suffix

# Custom variable access
zerv version --custom '{"env":"prod","build":123}' --output-template "{{major}}.{{minor}}.{{patch}}-{{custom.env}}.{{custom.build}}"
# Deep access to custom JSON variables

# Hash generation
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{hash bumped_commit_hash 8}}"
# Generate consistent hash from commit

# String sanitization for version compatibility
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{sanitize bumped_branch}}"
# "feature/test-0051" → "1.2.3+feature.test.51"

# Custom sanitization options
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{sanitize bumped_branch separator="-" lowercase=true}}"
# "Feature/Test-0051" → "1.2.3-feature-test-51"
```

## Template Context and Variables

### Template Context Creation

**File**: `src/version/zerv/mod.rs` (Add to Zerv impl)

```rust
impl Zerv {
    /// Create template context for rendering
    /// Context includes: ZervVars + PEP440/SemVer representations + Custom variables
    pub fn create_template_context(vars: &ZervVars) -> TemplateContext {
        TemplateContext {
            // Raw ZervVars fields
            vars: vars.clone(),

            // PEP440 representation
            pep440: vars.to_pep440_string(),

            // SemVer representation
            semver: vars.to_semver_string(),

            // Custom variables (if any)
            custom: vars.custom.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateContext {
    /// Raw ZervVars for direct field access
    #[serde(flatten)]
    pub vars: ZervVars,

    /// PEP440 formatted version string
    pub pep440: String,

    /// SemVer formatted version string
    pub semver: String,

    /// Custom variables from JSON
    pub custom: serde_json::Value,
}
```

### Available Template Variables

#### Core Version Fields (aligned with Plan 20 Var enum)

- `{{major}}` - Major version number (u64) → `Var::Major`
- `{{minor}}` - Minor version number (u64) → `Var::Minor`
- `{{patch}}` - Patch version number (u64) → `Var::Patch`
- `{{epoch}}` - PEP440 epoch number (u64) → `Var::Epoch`
- `{{post}}` - Post-release number (u64) → `Var::Post`
- `{{dev}}` - Development number (u64) → `Var::Dev`

#### Pre-release Fields

- `{{pre_release}}` - Pre-release label and number → `Var::PreRelease`

#### VCS Context Fields (aligned with Plan 20 Var enum)

- `{{distance}}` - Commits since last tag (u64) → `Var::Distance`
- `{{dirty}}` - Working tree dirty state (boolean) → `Var::Dirty`
- `{{bumped_branch}}` - Bumped branch name (string) → `Var::BumpedBranch`
- `{{bumped_commit_hash}}` - Full bumped commit hash (string) → `Var::BumpedCommitHash`
- `{{bumped_commit_hash_short}}` - Short bumped commit hash (string, 7 chars) → `Var::BumpedCommitHashShort`
- `{{bumped_timestamp}}` - Bumped commit timestamp (string) → `Var::BumpedTimestamp`
- `{{last_commit_hash}}` - Last version commit hash (string) → `Var::LastCommitHash`
- `{{last_branch}}` - Branch where last version was created (string) → `Var::LastBranch`
- `{{last_timestamp}}` - Last version creation timestamp (string) → `Var::LastTimestamp`

#### Formatted Version Strings

- `{{pep440}}` - Complete PEP440 formatted version string
- `{{semver}}` - Complete SemVer formatted version string

#### Custom Variables (aligned with Plan 20 Var enum)

- `{{custom.field_name}}` - Access custom JSON fields → `Var::Custom("field_name")`
- `{{custom.build_id}}` - Example: build identifier → `Var::Custom("build_id")`
- `{{custom.environment}}` - Example: deployment environment → `Var::Custom("environment")`
- `{{custom.nested.field}}` - Example: nested JSON access → `Var::Custom("nested.field")`

#### Built-in Handlebars Helpers (Already Available)

- `{{#if condition}}...{{/if}}` - Conditional blocks
- `{{#unless condition}}...{{/unless}}` - Negative conditional blocks
- `{{eq a b}}`, `{{gt a b}}`, `{{lt a b}}` - Comparison helpers
- `{{gte a b}}`, `{{lte a b}}`, `{{ne a b}}` - Additional comparison helpers
- `{{and a b}}`, `{{or a b}}`, `{{not condition}}` - Logical helpers

#### Custom Zerv Helpers (Need Implementation)

- `{{add a b}}` - Addition (a + b)
- `{{subtract a b}}` - Subtraction (a - b)
- `{{multiply a b}}` - Multiplication (a \* b)
- `{{hash input [length]}}` - Generate hex hash from input (default: 7 chars)
- `{{hash_int input [length] allow_leading_zero=false}}` - Generate integer hash from input
- `{{prefix string [length]}}` - Get prefix of string to length
- `{{format_timestamp timestamp format=format_string}}` - Format unit timestamp to string
- `{{sanitize input [separator="."] [keep_zeros=false] [max_length] [lowercase=false]}}` - Sanitize strings for version compatibility

## String Sanitization Helper

### `sanitize` Helper

The `sanitize` helper transforms strings to be compatible with version standards (SemVer, PEP440) by replacing invalid characters and normalizing numeric segments.

**Syntax**: `{{sanitize input [options]}}`

#### Options

- **`separator`** (string, default: `"."`) - Character to replace invalid characters
- **`keep_zeros`** (boolean, default: `false`) - Whether to preserve leading zeros in numeric segments
- **`max_length`** (number, optional) - Truncate result to maximum length
- **`lowercase`** (boolean, default: `false`) - Convert to lowercase

#### Transformation Rules

**Invalid characters replaced with `separator`**:

- `/` → separator
- `-` → separator
- `_` → separator (if different from chosen separator)
- spaces → separator
- Special chars (`@`, `#`, `%`, etc.) → separator

**Leading zero removal** (unless `keep_zeros=true`):

- `0051` → `51`
- `001` → `1`
- `0` → `0` (single zero preserved)

#### Usage Examples

```bash
# Basic usage - most common case
{{sanitize "feature/test-rtet-0051"}}
# Output: "feature.test.rtet.51"

# Custom separator for SemVer build metadata
{{sanitize "feature/test-0051" separator="-"}}
# Output: "feature-test-51"

# Keep leading zeros for fixed-width identifiers
{{sanitize "feature/test-0051" keep_zeros=true}}
# Output: "feature.test.rtet.0051"

# Lowercase for consistency
{{sanitize "Feature/Test-RTET" lowercase=true}}
# Output: "feature.test.rtet"

# Multiple options combined
{{sanitize "Feature/Test-0051" separator="_" lowercase=true max_length=15}}
# Output: "feature_test_51" (or truncated if longer)

# Advanced: underscore separator with zero preservation
{{sanitize "build/env-prod-0123" separator="_" keep_zeros=true}}
# Output: "build_env_prod_0123"
```

#### Real-World Use Cases

```bash
# SemVer compatible branch in build metadata
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{sanitize bumped_branch}}"
# "feature/user-auth-v2-0051" → "1.2.3+feature.user.auth.v2.51"

# PEP440 compatible with dashes
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{sanitize bumped_branch separator=\"-\"}}"
# "feature/user-auth-0051" → "1.2.3+feature-user-auth-51"

# Custom pre-release label from sanitized branch
zerv version --pre-release-label "{{sanitize bumped_branch lowercase=true max_length=10}}"
# "Feature/Long-Branch-Name" → "feature.lon" (truncated)

# Environment-specific versioning
zerv version --custom '{"env":"staging/test-env-001"}' --output-template "{{major}}.{{minor}}.{{patch}}-{{sanitize custom.env}}"
# Output: "1.2.3-staging.test.env.1"

# Complex build metadata with multiple sanitized components
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{sanitize bumped_branch}}.{{sanitize custom.build_env separator=\"-\"}}"
# Branch: "feature/api-v2", Env: "prod/us-east-1" → "1.2.3+feature.api.v2.prod-us-east-1"
```

#### Benefits

1. **Version Standard Compliance**: Makes any string compatible with SemVer/PEP440 requirements
2. **Flexible Configuration**: Customizable separator and transformation rules
3. **Predictable Output**: Clear, documented transformation behavior
4. **Zero Handling**: Smart numeric segment normalization with override option
5. **Length Control**: Prevents overly long version strings
6. **Case Normalization**: Consistent lowercase output when needed

## Render Timing Control

### EARLY RENDERING (before `apply_component_processing`)

**Context**: Current VCS state + any tag version overrides
**Used for**: Field overrides, schema component overrides/bumps
**Template Variables**: Current state before any bumping

```rust
// These render EARLY using current Zerv context:
pub major: Option<Template<u32>>,         // Field override
pub core: Vec<Template<IndexValue>>,        // Schema override
pub bump_core: Vec<Template<IndexValue>>,   // Schema bump
```

### LATE RENDERING (after `apply_component_processing`)

**Context**: Final processed Zerv object with all bumps applied
**Used for**: Output formatting
**Template Variables**: Final state after all processing

```rust
// This renders LATE using final Zerv object:
pub output_template: Option<Template<String>>, // Output format
```

## Integration with Component::Var Enum (From Plan 20)

**CRITICAL DEPENDENCY**: This plan requires Plan 20's `Var` enum to be implemented first.

Template variables will be validated against the `Var` enum:

```rust
// Template validation using Var enum from Plan 20
pub fn validate_template_variables(template: &str) -> Result<(), ZervError> {
    let handlebars = Handlebars::new();
    let ast = handlebars.compile_template(template)?;

    // Extract variable references from AST
    let variables = extract_template_variables(&ast);

    // Validate against Var enum from Plan 20
    for var_name in variables {
        // Use Plan 20's Var enum for validation
        if !is_valid_template_variable(&var_name) {
            return Err(ZervError::TemplateError(
                format!("Unknown template variable '{}'. Valid variables: {:?}",
                    var_name, get_valid_template_variables())
            ));
        }
    }

    Ok(())
}

// Helper functions that use Plan 20's Var enum
fn is_valid_template_variable(var_name: &str) -> bool {
    // Implementation depends on Plan 20's Var enum
    // Check if var_name matches any Var enum variant
    true // Placeholder
}

fn get_valid_template_variables() -> Vec<String> {
    // Return list of valid template variables from Plan 20's Var enum
    vec![] // Placeholder
}
```

## Implementation Plan

### Prerequisites (CRITICAL)

- **MUST COMPLETE FIRST**: Plan 20 (Component VarField Enum Refactoring)
- **Reason**: Template validation requires `Var` enum for field name validation
- **Dependencies**:
    - `Var` enum with all template variable variants
    - `strum` crate for enum string conversion
    - Updated `Component::Var(Var)` structure

### Migration Strategy

- **Backward Compatibility**: All existing CLI usage must continue working
- **Gradual Migration**: Change argument types without breaking existing behavior
- **Auto-detection**: Template vs literal values detected automatically

### Phase 1: Template Type Foundation

1. **Add Handlebars dependency** to `Cargo.toml`
2. **Create template types** in `src/cli/utils/template/types.rs`:
    - `Template<T>` - Template-aware type for any T
    - `IndexValue` - Simple INDEX=VALUE type that works with Template<T>
    - `render_template()` function
3. **Clap integration**: Uses `FromStr` implementation directly
    - Auto-detection of template vs literal values
4. **Error handling** integration with existing `ZervError`

### Phase 2: CLI Arguments Migration

1. **Update existing argument types** in `args/overrides.rs`:
    - `pub major: Option<u32>` → `pub major: Option<Template<u32>>`
    - `pub minor: Option<u32>` → `pub minor: Option<Template<u32>>`
    - `pub patch: Option<u32>` → `pub patch: Option<Template<u32>>`
    - `pub dev: Option<u32>` → `pub dev: Option<Template<u32>>`
    - `pub post: Option<u32>` → `pub post: Option<Template<u32>>`
    - `pub epoch: Option<u32>` → `pub epoch: Option<Template<u32>>`
    - `pub pre_release_num: Option<u32>` → `pub pre_release_num: Option<Template<u32>>`
    - `pub core: Vec<String>` → `pub core: Vec<Template<IndexValue>>`
    - `pub extra_core: Vec<String>` → `pub extra_core: Vec<Template<IndexValue>>`
    - `pub build: Vec<String>` → `pub build: Vec<Template<IndexValue>>`
2. **Create new `args/bumps.rs`** with bump functionality
3. **Update `output_template`** in `args/main.rs`:
    - `pub output_template: Option<String>` → `pub output_template: Option<Template<String>>`
4. **Backward compatibility testing**: Ensure all existing CLI usage still works

### Phase 3: Pipeline Integration

1. **Early rendering** in `ZervVars::apply_context_overrides()`:
    - Add `apply_templated_field_overrides()` method
    - Render field override templates using current context
    - Render schema component templates using current context
2. **Late rendering** in `OutputFormatter::format_output()`:
    - Render output template using final Zerv object
    - Replace existing basic template logic
3. **Template context creation**:
    - Minimal Zerv object for early rendering context
    - Full Zerv object for late rendering context

### Phase 4: Advanced Features

1. **Custom Handlebars helpers**:
    - Math operations: `add`, `subtract`, `multiply`
    - Hash generation: `hash`, `hash_int`
    - Conditional helpers: enhanced `if` logic
2. **Template validation**:
    - Integration with Var enum for field validation
    - Syntax validation at parse time
    - Runtime validation with helpful error messages
3. **Performance optimization**:
    - Template compilation caching
    - Minimal context creation for early rendering

## Benefits

### 1. **Intuitive Render-at-Use Timing**

- **Early rendering**: Field overrides use current context (before processing)
- **Late rendering**: Output templates use final context (after processing)
- **Natural timing**: Templates render when the argument is actually used
- **Predictable behavior**: Users understand when template variables are resolved

### 2. **Type-Level Template Awareness**

- **Explicit templating**: `Template<T>` clearly indicates template support
- **Natural CLI mapping**: `Option<Template<T>>` follows standard Clap patterns
- **Auto-detection**: Handlebars syntax automatically detected vs literals
- **Type safety**: Template resolution returns strongly-typed values
- **Clap integration**: Seamless CLI parsing with custom value parsers

### 3. **Flexible Template Context**

- **Current state access**: Early templates can reference existing version state
- **VCS context**: Access to branch, commit hash, distance, dirty state
- **Custom variables**: JSON custom fields accessible in templates
- **Computed fields**: Short hashes, timestamps, derived values

### 4. **Backward Compatibility**

- **Existing usage preserved**: Literal values work exactly as before
- **Opt-in templating**: Only arguments with `{{}}` syntax are templated
- **Gradual adoption**: Users can mix literal and templated arguments
- **No breaking changes**: Current CLI behavior unchanged

### 5. **Integration with Var Enum**

- **Field validation**: Template variables validated against Var enum
- **IDE support**: Auto-completion for valid template variables
- **Compile-time safety**: Invalid field references caught early
- **Consistent naming**: Template variables match Var enum variants

This approach provides powerful templating with intuitive timing control while maintaining full backward compatibility and type safety.
