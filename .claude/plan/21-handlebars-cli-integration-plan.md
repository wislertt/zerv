<!-- TODO: review before implement-->

# Plan 21: Handlebars CLI Integration (Phase 1 Only)

## Prerequisites

**CRITICAL**: This plan must be implemented **AFTER** Plans 19 and 20 are completed.

**Reason**: Template validation requires the `Var` enum from Plan 20 for field name validation and sanitization from Plan 19.

## Dependencies

- **Plan 19**: String Sanitization Utils (must be implemented first)
- **Plan 20**: Component Resolution Centralization (must be implemented second)

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

## Phase 1 Scope

Focus only on core template processing with variable substitution. Leave advanced features (conditionals, loops, helpers) for future phases.

## Core Components

### Template Processing

```rust
pub struct TemplateProcessor {
    handlebars: Handlebars<'static>,
}

impl TemplateProcessor {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        Self::register_custom_helpers(&mut handlebars);
        Self { handlebars }
    }

    pub fn process(&self, template: &str, vars: &ZervVars) -> Result<String, TemplateError>

    fn register_custom_helpers(handlebars: &mut Handlebars) {
        // Register Zerv-specific helpers
    }
}
```

### Template Types

**File**: `src/cli/utils/template.rs`

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

### Variable Context

```rust
pub struct TemplateContext {
    // Prepared context for Handlebars
}

impl TemplateContext {
    pub fn from_zerv_vars(vars: &ZervVars) -> Self
    pub fn to_handlebars_context(&self) -> serde_json::Value
}
```

## Integration with Plans 19 & 20

### Sanitization Integration

- Use Plan 19 sanitizers for cleaning template variables
- Apply appropriate sanitization based on context

### Component Resolution Integration

- Use Plan 20 resolution methods for variable lookup
- Leverage centralized component resolution logic

## Template Features (Phase 1)

### Variable Substitution

- Basic `{{variable}}` syntax
- Nested variable access `{{object.field}}`
- Safe variable handling with defaults

### Custom Handlebars Helpers

- `{{add a b}}` - Addition (a + b)
- `{{subtract a b}}` - Subtraction (a - b)
- `{{multiply a b}}` - Multiplication (a \* b)
- `{{hash input [length]}}` - Generate hex hash (default: 7 chars)
- `{{sanitize input [options]}}` - String sanitization for version compatibility

### Error Handling

- Template parsing errors
- Variable resolution errors
- Missing variable handling

## CLI Integration

### Update CLI Arguments with Template Types

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

### New Command Options

```bash
zerv template --input template.hbs --output result.txt
zerv template --template "Version: {{major}}.{{minor}}.{{patch}}"
```

### Template File Support

- Read templates from files
- Support standard Handlebars syntax
- Output to files or stdout

## Pipeline Integration with Render-at-Use

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

## Implementation Files

### New Files

- `src/template/mod.rs` - Template module
- `src/template/processor.rs` - Core template processing
- `src/template/context.rs` - Variable context handling
- `src/template/errors.rs` - Template error types
- `src/cli/utils/template.rs` - Template types and CLI integration

### Updated Files

- `src/lib.rs` - Add template module export
- `Cargo.toml` - Add handlebars dependency
- `src/cli/version/args/main.rs` - Update output_template type
- `src/cli/version/args/overrides.rs` - Update all override types
- `src/cli/version/pipeline.rs` - Add template rendering
- `src/error.rs` - Add TemplateError variants

## Testing Strategy

### Unit Tests

- Test template processing with various inputs
- Test variable context creation
- Test error conditions

### Integration Tests

- Test with real ZervVars data
- Test CLI template commands
- Test file input/output

### Template Tests

- Test common version templates
- Test edge cases and error scenarios
- Test variable sanitization
- Test custom helpers (add, subtract, multiply, hash, sanitize)
- Test helper error conditions

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

### Output Templates (LATE RENDERING)

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

## Success Criteria

- Basic template processing works
- Variable substitution is reliable
- Clean integration with existing codebase
- Foundation ready for future phases
- CLI commands functional
- Template validation against Var enum
- Custom helpers implemented and tested
- Render timing works correctly (early vs late)
