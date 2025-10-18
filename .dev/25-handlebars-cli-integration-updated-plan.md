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

### ✅ What's Already Implemented (Steps 1-2)

1. **✅ Handlebars Integration**: `handlebars = "^6.3"` added to Cargo.toml
2. **✅ Template Module**: Complete template infrastructure implemented
    - `src/cli/utils/template/mod.rs` - Module exports
    - `src/cli/utils/template/types.rs` - Template<T> enum with resolve() method
    - `src/cli/utils/template/context.rs` - TemplateContext and PreReleaseContext
    - `src/cli/utils/template/helpers.rs` - Custom Handlebars helpers
    - `src/cli/utils/mod.rs` - Template module exported
    - `src/error.rs` - TemplateError variant added

### 🔄 What Needs Implementation (Steps 3-4)

3. **CLI Integration**: Update argument types to support templating
4. **Pipeline Integration**: Add early/late rendering and update processing logic

**Deviations from Plan**:

- `Template::resolve()` takes `&Zerv` instead of `&ZervVars` (cleaner API)
- `TemplateContext::from_zerv()` instead of `from_zerv_vars()` (matches implementation)
- Template module structure is cleaner than planned

**Current CLI Structure Analysis**:

- `MainConfig::output_template: Option<String>` - needs Template<String>
- `OverridesConfig` - all version fields are `Option<u32>` - need Template<u32>
- `OverridesConfig` - schema fields are `Vec<String>` - need Template<String>
- `BumpsConfig` - all bump fields are `Option<Option<u32>>` - need Template<u32>

## Schema Component Template Design

### Approach:

```rust
pub core: Vec<Template<String>>,  // Each can be: "0=value", "0={{major}}", etc.
```

### Processing Flow:

```rust
// 1. CLI args as templates
pub core: Vec<Template<String>>

// 2. Resolve templates
let resolved: Vec<String> = core.iter().map(|t| t.resolve(vars)).collect();

// 3. Use existing parsers
for spec in resolved {
    let (index, value) = parse_override_spec(&spec, schema_len)?;
    // Apply to schema...
}
```

## Implementation Plan

### ✅ Step 1: Add Handlebars Dependency - COMPLETED

**Status**: ✅ **COMPLETED** - `handlebars = "^6.3"` already in Cargo.toml

### ✅ Step 2: Template Module Implementation - COMPLETED

**Status**: ✅ **COMPLETED** - All template infrastructure implemented

### 🔄 Step 3: Update CLI Arguments with Template Types - IN PROGRESS

**Current State**: CLI args use primitive types, need Template<T> wrapper

**File**: `src/cli/version/args/main.rs` (update existing)

```rust
// ADD: Import template types
use crate::cli::utils::template::Template;

// CHANGE: Update output_template field type
pub struct MainConfig {
    // ... existing fields unchanged ...

    /// Output template for custom formatting (Handlebars syntax)
    pub output_template: Option<Template<String>>, // CHANGED: was Option<String>

    // ... rest unchanged ...
}
```

**File**: `src/cli/version/args/overrides.rs` (update existing)

```rust
// ADD: Import template types
use crate::cli::utils::template::Template;

// CHANGE: Update version component field types
pub struct OverridesConfig {
    // ... VCS override options unchanged ...

    // VERSION COMPONENT OVERRIDES - CHANGE TO TEMPLATE TYPES
    pub major: Option<Template<u32>>,         // CHANGED: was Option<u32>
    pub minor: Option<Template<u32>>,         // CHANGED: was Option<u32>
    pub patch: Option<Template<u32>>,         // CHANGED: was Option<u32>
    pub epoch: Option<Template<u32>>,         // CHANGED: was Option<u32>
    pub post: Option<Template<u32>>,          // CHANGED: was Option<u32>
    pub dev: Option<Template<u32>>,           // CHANGED: was Option<u32>
    pub pre_release_num: Option<Template<u32>>, // CHANGED: was Option<u32>

    // SCHEMA COMPONENT OVERRIDES - CHANGE TO TEMPLATE TYPES
    pub core: Vec<Template<String>>,          // CHANGED: was Vec<String>
    pub extra_core: Vec<Template<String>>,    // CHANGED: was Vec<String>
    pub build: Vec<Template<String>>,         // CHANGED: was Vec<String>

    // ... other fields unchanged ...
}
```

**File**: `src/cli/version/args/bumps.rs` (update existing)

```rust
// ADD: Import template types
use crate::cli::utils::template::Template;

// CHANGE: Update bump field types
pub struct BumpsConfig {
    // FIELD-BASED BUMP OPTIONS - CHANGE TO TEMPLATE TYPES
    pub bump_major: Option<Option<Template<u32>>>,     // CHANGED: was Option<Option<u32>>
    pub bump_minor: Option<Option<Template<u32>>>,     // CHANGED: was Option<Option<u32>>
    pub bump_patch: Option<Option<Template<u32>>>,     // CHANGED: was Option<Option<u32>>
    pub bump_post: Option<Option<Template<u32>>>,      // CHANGED: was Option<Option<u32>>
    pub bump_dev: Option<Option<Template<u32>>>,       // CHANGED: was Option<Option<u32>>
    pub bump_pre_release_num: Option<Option<Template<u32>>>, // CHANGED: was Option<Option<u32>>
    pub bump_epoch: Option<Option<Template<u32>>>,     // CHANGED: was Option<Option<u32>>

    // SCHEMA-BASED BUMP OPTIONS - CHANGE TO TEMPLATE TYPES
    pub bump_core: Vec<Template<String>>,              // CHANGED: was Vec<String>
    pub bump_extra_core: Vec<Template<String>>,        // CHANGED: was Vec<String>
    pub bump_build: Vec<Template<String>>,             // CHANGED: was Vec<String>

    // ... rest unchanged ...
}
```

### 📋 Step 4: Pipeline Integration with Render Timing - PENDING

**Key Architecture: ResolvedArgs Pattern**

To handle template resolution correctly, we need a ResolvedArgs pattern:

**File**: `src/cli/version/args/resolved.rs` (new)

```rust
use crate::cli::utils::template::Template;
use crate::cli::version::args::{VersionArgs, MainConfig};
use crate::version::Zerv;
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
}

impl ResolvedArgs {
    /// Resolve all templates in VersionArgs using Zerv snapshot
    pub fn resolve(args: &VersionArgs, zerv: &Zerv) -> Result<Self, ZervError> {
        let overrides = ResolvedOverrides::resolve(&args.overrides, zerv)?;
        let bumps = ResolvedBumps::resolve(&args.bumps, zerv)?;

        Ok(ResolvedArgs {
            overrides,
            bumps,
            main: args.main.clone(),
        })
    }
}

impl ResolvedOverrides {
    fn resolve(overrides: &OverridesConfig, zerv: &Zerv) -> Result<Self, ZervError> {
        Ok(ResolvedOverrides {
            major: Self::resolve_template(&overrides.major, zerv)?,
            minor: Self::resolve_template(&overrides.minor, zerv)?,
            patch: Self::resolve_template(&overrides.patch, zerv)?,
            epoch: Self::resolve_template(&overrides.epoch, zerv)?,
            post: Self::resolve_template(&overrides.post, zerv)?,
            dev: Self::resolve_template(&overrides.dev, zerv)?,
            pre_release_num: Self::resolve_template(&overrides.pre_release_num, zerv)?,
            core: Self::resolve_template_strings(&overrides.core, zerv)?,
            extra_core: Self::resolve_template_strings(&overrides.extra_core, zerv)?,
            build: Self::resolve_template_strings(&overrides.build, zerv)?,
        })
    }

    fn resolve_template<T>(template: &Option<Template<T>>, zerv: &Zerv) -> Result<Option<T>, ZervError>
    where
        T: FromStr + Clone,
        T::Err: Display,
    {
        match template {
            Some(t) => Ok(Some(t.resolve(zerv)?)),
            None => Ok(None),
        }
    }

    fn resolve_template_strings(templates: &[Template<String>], zerv: &Zerv) -> Result<Vec<String>, ZervError> {
        templates.iter()
            .map(|template| template.resolve(zerv))
            .collect()
    }
}

impl ResolvedBumps {
    fn resolve(bumps: &BumpsConfig, zerv: &Zerv) -> Result<Self, ZervError> {
        Ok(ResolvedBumps {
            bump_major: Self::resolve_bump(&bumps.bump_major, zerv)?,
            bump_minor: Self::resolve_bump(&bumps.bump_minor, zerv)?,
            bump_patch: Self::resolve_bump(&bumps.bump_patch, zerv)?,
            bump_epoch: Self::resolve_bump(&bumps.bump_epoch, zerv)?,
            bump_post: Self::resolve_bump(&bumps.bump_post, zerv)?,
            bump_dev: Self::resolve_bump(&bumps.bump_dev, zerv)?,
            bump_pre_release_num: Self::resolve_bump(&bumps.bump_pre_release_num, zerv)?,
        })
    }

    fn resolve_bump(bump: &Option<Option<Template<u32>>>, zerv: &Zerv) -> Result<Option<Option<u32>>, ZervError> {
        match bump {
            Some(Some(template)) => Ok(Some(Some(template.resolve(zerv)?))),
            Some(None) => Ok(Some(None)),
            None => Ok(None),
        }
    }
}
```

**File**: `src/cli/version/pipeline.rs` (update existing)

```rust
use crate::cli::utils::template::Template;

pub fn run_version_pipeline(mut args: VersionArgs) -> Result<String, ZervError> {
    // ... existing pipeline logic ...

    // 3. Convert to Zerv (EARLY RENDERING happens inside to_zerv)
    let zerv_object = zerv_draft.to_zerv(&args)?;

    // 4. Apply output formatting (LATE RENDERING for output_template)
    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.main.output_format,
        args.main.output_prefix.as_deref(),
        &args.main.output_template,
    )?;

    Ok(output)
}
```

**File**: `src/cli/utils/output_formatter.rs` (update existing)

```rust
use crate::cli::utils::template::Template;

impl OutputFormatter {
    pub fn format_output(
        zerv: &Zerv,
        output_format: &str,
        output_prefix: Option<&str>,
        output_template: &Option<Template<String>>,
    ) -> Result<String, ZervError> {
        // Handle template rendering for output
        let formatted = if let Some(template) = output_template {
            template.resolve(zerv)?
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

## Template Features

### Available Template Variables

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

**Pre-defined Format Variables:**

- `iso_date` - ISO date format (`%Y-%m-%d`) → "2023-12-21"
- `iso_datetime` - ISO datetime format (`%Y-%m-%dT%H:%M:%S`) → "2023-12-21T12:34:56"
- `compact_date` - Compact date format (`%Y%m%d`) → "20231221"
- `compact_datetime` - Compact datetime format (`%Y%m%d%H%M%S`) → "20231221123456"

## Render Timing Control

### EARLY RENDERING (before version processing)

**Context**: VCS state + base version from tag
**Used for**: All overrides and bumps
**Examples**: `--major "{{distance}}"`, `--bump-patch "{{dev}}"`

### LATE RENDERING (after version processing)

**Context**: VCS state + fully computed final version
**Used for**: Output template only
**Examples**: `--output-template "v{{major}}.{{minor}}.{{patch}}"`

## Usage Examples

### Field Override Templates (EARLY RENDERING)

```bash
# Override version fields with templates
zerv version --major "{{add major 1}}" --minor "{{custom.target_minor}}"

# Override with current VCS context
zerv version --dev "{{distance}}" --post "{{add distance 10}}"

# Complex field templates
zerv version --patch "{{multiply minor 10}}" --epoch "{{custom.release_year}}"
```

### Schema Component Override Templates

```bash
# Schema component overrides with templates
zerv version --core "0={{major}}" --core "1={{bumped_branch}}"

# Complex schema templates
zerv version --extra-core "0={{add post 1}}" --build "0={{hash bumped_commit_hash 8}}"
```

### Bump Templates (EARLY RENDERING)

```bash
# Bump with template values
zerv version --bump-major "{{distance}}" --bump-minor "{{custom.increment}}"

# Conditional bumps
zerv version --bump-patch "{{#if dirty}}10{{else}}1{{/if}}"
```

### Output Templates (LATE RENDERING)

```bash
# Custom output format after all processing
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{bumped_branch}}"

# Complex output with helpers
zerv version --output-template "v{{major}}.{{minor}}.{{patch}}+{{hash bumped_commit_hash 7}}"

# Output with custom variables and timestamps
zerv version --output-template "{{major}}.{{minor}}.{{patch}}+{{custom.build_id}}.{{format_timestamp bumped_timestamp format=compact_date}}"
```

## Migration Strategy

### ✅ Phase 1: Add Template Infrastructure - COMPLETED

1. ✅ Add handlebars dependency
2. ✅ Implement template types and helpers
3. ✅ Add template module exports

### 🔄 Phase 2: Update CLI Arguments - IN PROGRESS

1. 🔄 Update MainConfig.output_template type
2. 🔄 Update OverridesConfig field types
3. 🔄 Update BumpsConfig field types

### 📋 Phase 3: Pipeline Integration - PENDING

1. 📋 Add early rendering for overrides/bumps
2. 📋 Add late rendering for output templates
3. 📋 Update output formatter
4. 📋 Add error handling

### 📋 Phase 4: Testing and Documentation - PENDING

1. 📋 Add comprehensive unit tests
2. 📋 Add integration tests
3. 📋 Update CLI help text
4. 📋 Add usage examples

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::ZervFixture;

    #[test]
    fn test_template_value_resolution() {
        let template = Template::Value(42u32);
        let zerv = ZervFixture::new().build();
        assert_eq!(template.resolve(&zerv).unwrap(), 42);
    }

    #[test]
    fn test_template_string_resolution() {
        let template = Template::Template("{{major}}.{{minor}}.{{patch}}".to_string());
        let zerv = ZervFixture::new().with_version(1, 2, 3).build();
        assert_eq!(template.resolve(&zerv).unwrap(), "1.2.3");
    }

    #[test]
    fn test_template_helpers() {
        let template = Template::Template("{{add major minor}}".to_string());
        let zerv = ZervFixture::new().with_version(1, 2, 3).build();
        assert_eq!(template.resolve(&zerv).unwrap(), "3");
    }
}
```

### Integration Tests

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
```

## Success Criteria

- ✅ **Handlebars dependency added** - COMPLETED
- ✅ **Template infrastructure implemented** - COMPLETED
- ✅ **Template module exported** - COMPLETED
- ✅ **TemplateError handling added** - COMPLETED
- 🔄 **Template types replace primitive types in CLI arguments** - IN PROGRESS
- 📋 **Early vs late rendering timing works correctly** - PENDING
- 📋 **All existing functionality preserved** - PENDING
- 📋 **Template validation and error handling** - PENDING
- 📋 **Comprehensive test coverage** - PENDING
- 📋 **Clean integration with existing codebase** - PENDINGG

## Benefits

1. **Dynamic Versioning**: Templates enable dynamic version generation based on VCS state
2. **Flexible Output**: Custom output templates for any format requirement
3. **Mathematical Operations**: Built-in helpers for version calculations
4. **Context Awareness**: Access to all VCS and custom variables
5. **Render Timing Control**: Early rendering for processing, late rendering for output
6. **Type Safety**: Template types maintain type safety while adding flexibility
7. **Backward Compatibility**: Existing literal values continue to work
8. **Extensible**: Easy to add new helpers and template variables

## Key Architectural Decisions

### ResolvedArgs Pattern

**Decision**: Use ResolvedArgs to separate template resolution from processing
**Benefits**:

- **Predictable behavior** - All templates see identical context
- **No order dependency** - Overrides don't affect bump calculations
- **Single snapshot** - Eliminates timing-related inconsistencies
- **Atomic operation** - All early rendering happens together

### Two-Phase Rendering

**Early Phase**: Before version processing (overrides/bumps)
**Late Phase**: After version processing (output only)
**Benefits**:

- **Clear separation** - Processing vs output concerns
- **Consistent context** - Each phase has well-defined available variables
- **No circular dependencies** - Overrides use historical data as input

This updated plan leverages the solid foundation from Plans 19-24 and provides a comprehensive templating system that aligns with the ideal specification from Plan 11.
