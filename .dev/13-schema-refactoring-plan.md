# Schema Refactoring Plan: Separate Schema from Core

## Overview

Refactor the Zerv codebase to separate schema-related types and validation logic from the core Zerv struct, consolidating all schema functionality into a dedicated module.

## Current State

### Current Structure

```
src/version/zerv/
├── core.rs                    # Contains Zerv, ZervVars, ZervSchema, Component
├── display.rs
├── parser.rs
├── utils.rs
└── mod.rs

src/schema/validation/
├── component.rs               # Component validation logic
├── structure.rs              # Structure validation logic
└── mod.rs
```

### Problems with Current Structure

1. **Mixed Concerns**: `core.rs` contains both data structures and schema definitions
2. **Scattered Validation**: Schema validation logic is in separate `src/schema/validation/` directory
3. **Unclear Dependencies**: Schema validation depends on core types but is in different module
4. **Maintenance Overhead**: Changes to schema types require updates in multiple places

## Target State

### New Structure

```
src/version/zerv/
├── core.rs                    # Only Zerv, PreReleaseLabel, PreReleaseVar
├── vars.rs                    # ZervVars + vars validation logic
├── schema.rs                  # ZervSchema, Component + schema validation only
├── display.rs
├── parser.rs
├── utils.rs
└── mod.rs

# Remove src/schema/validation/ entirely
```

### Separation of Concerns

1. **Schema Validation** (`schema.rs`): Only validates ZervSchema structure and Component validity
2. **Vars Module** (`vars.rs`): ZervVars struct and vars-specific validation logic
3. **Zerv Object Creation** (`core.rs`): Validates compatibility between ZervVars and ZervSchema when creating Zerv objects

### Benefits

1. **Clear Separation**: Core data structures separate from schema and vars definitions
2. **Focused Validation**: Each module handles its own validation concerns
3. **Type Safety**: ZervVars validation through Rust's type system
4. **Better Organization**: Related code grouped together (schema, vars, core)
5. **Easier Maintenance**: Single source of truth for each concern
6. **Modular Design**: Each module can be imported independently

## Implementation Plan

### Phase 1: Create New Modules

#### 1.1 Create `src/version/zerv/vars.rs`

```rust
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ZervVars {
    // Core version fields
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub epoch: Option<u64>,
    pub pre_release: Option<PreReleaseVar>,
    pub post: Option<u64>,
    pub dev: Option<u64>,

    // VCS state fields
    pub distance: Option<u64>,
    pub dirty: Option<bool>,

    // Bumped fields (for template access)
    pub bumped_branch: Option<String>,
    pub bumped_commit_hash: Option<String>,
    pub bumped_commit_hash_short: Option<String>,
    pub bumped_timestamp: Option<u64>,

    // Last version fields (for template access)
    pub last_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_timestamp: Option<u64>,

    // Custom variables
    #[serde(default = "default_custom_value")]
    pub custom: serde_json::Value,
}

/// Default value for custom field - returns an empty JSON object
fn default_custom_value() -> serde_json::Value {
    serde_json::json!({})
}

impl ZervVars {
    // Vars-specific validation methods can go here
}
```

#### 1.2 Create `src/version/zerv/schema.rs`

```rust
use serde::{Deserialize, Serialize};
use crate::constants::{ron_fields, timestamp_patterns};
use crate::error::ZervError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZervSchema {
    pub core: Vec<Component>,
    pub extra_core: Vec<Component>,
    pub build: Vec<Component>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "str")]
    String(String),
    #[serde(rename = "int")]
    Integer(u64),
    #[serde(rename = "var")]
    VarField(String),
    #[serde(rename = "ts")]
    VarTimestamp(String),
}

// Schema validation only - validates ZervSchema structure and Component validity
pub mod validation {
    // Component validation (field names, timestamp patterns)
    // Schema structure validation (non-empty schema, valid components)
    // NO ZervVars validation - that's handled by type system
}
```

#### 1.3 Move Types from `core.rs`

- Move `ZervSchema` struct to `schema.rs`
- Move `Component` enum to `schema.rs`
- Move `ZervVars` struct to `vars.rs`
- Update imports in `core.rs`

#### 1.4 Move Schema Validation Logic

- Move `src/schema/validation/component.rs` → `src/version/zerv/schema.rs` (component validation only)
- Move schema structure validation from `src/schema/validation/structure.rs` → `src/version/zerv/schema.rs`
- **EXCLUDE**: ZervVars validation logic (lines 27-110 in structure.rs) - this belongs in Zerv object creation
- Consolidate into focused schema validation module

### Phase 2: Update Core Module

#### 2.1 Clean Up `core.rs`

Keep only:

```rust
use serde::{Deserialize, Serialize};
use crate::version::zerv::schema::{ZervSchema, Component};
use crate::version::zerv::vars::ZervVars;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreReleaseLabel {
    Alpha,
    Beta,
    Rc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zerv {
    pub schema: ZervSchema,  // Import from schema module
    pub vars: ZervVars,      // Import from vars module
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreReleaseVar {
    pub label: PreReleaseLabel,
    pub number: Option<u64>,
}

impl Zerv {
    /// Create a new Zerv object with validation
    pub fn new(schema: ZervSchema, vars: ZervVars) -> Result<Self, ZervError> {
        // Validate schema-vars compatibility here
        Self::validate_schema_vars_compatibility(&schema, &vars)?;
        Ok(Self { schema, vars })
    }

    /// Validate that ZervVars has values for all required schema fields
    fn validate_schema_vars_compatibility(schema: &ZervSchema, vars: &ZervVars) -> Result<(), ZervError> {
        // Move ZervVars validation logic here from structure.rs
    }
}
```

#### 2.2 Move ZervVars Validation Logic

- Move ZervVars validation logic from `src/schema/validation/structure.rs` (lines 27-110) to `Zerv::new()` method
- This validates that ZervVars has values for all required schema fields when creating Zerv objects
- Keep schema validation separate from ZervVars validation

### Phase 3: Update Module Exports

#### 3.1 Update `src/version/zerv/mod.rs`

```rust
pub mod core;
pub mod vars;     // New module
pub mod schema;   // New module
mod display;
mod parser;
#[cfg(test)]
mod test_schema_assertions;
#[cfg(test)]
pub mod test_utils;
pub mod utils;

// Core types
pub use core::{PreReleaseLabel, PreReleaseVar, Zerv};
// Vars types
pub use vars::ZervVars;
// Schema types
pub use schema::{Component, ZervSchema};
// Utilities
pub use utils::{normalize_pre_release_label, resolve_timestamp};
```

### Phase 4: Update Dependencies

#### 4.1 Update Import References

Search and replace throughout codebase:

- `use crate::version::zerv::{ZervSchema, Component}` → `use crate::version::zerv::schema::{ZervSchema, Component}`
- `use crate::version::zerv::{ZervVars}` → `use crate::version::zerv::vars::{ZervVars}`
- `use crate::schema::validation::*` → `use crate::version::zerv::schema::validation::*`

#### 4.2 Update CLI Modules

- `src/cli/version.rs`
- `src/cli/utils/format_handler.rs`
- `src/cli/utils/output_formatter.rs`

#### 4.3 Update Pipeline Modules

- `src/pipeline/vcs_data_to_zerv_vars.rs`
- `src/pipeline/parse_version_from_tag.rs`

#### 4.4 Update Test Files

- All test files that import schema types
- Update test utilities in `src/test_utils/zerv/`

### Phase 5: Remove Old Schema Module

#### 5.1 Delete Old Files

- `src/schema/validation/component.rs`
- `src/schema/validation/structure.rs`
- `src/schema/validation/mod.rs`
- `src/schema/validation/` (entire directory)

#### 5.2 Update Schema Module

- Update `src/schema/mod.rs` to remove validation exports
- Update `src/schema/presets/` if they depend on validation

### Phase 6: Testing and Validation

#### 6.1 Run Tests

```bash
make test
make lint
```

#### 6.2 Verify Functionality

- Test schema parsing
- Test validation logic
- Test CLI commands
- Test integration tests

#### 6.3 Check for Breaking Changes

- Ensure all public APIs still work
- Verify serialization/deserialization
- Check error messages are still correct

## Migration Checklist

### During Migration

- [ ] Create `src/version/zerv/schema.rs`
- [ ] Move `ZervSchema` and `Component` types
- [ ] Move validation logic from `src/schema/validation/`
- [ ] Update `core.rs` to remove moved types
- [ ] Update `mod.rs` exports
- [ ] Update all import references
- [ ] Remove old validation files
- [ ] Run tests after each major step

### Post-Migration

- [ ] All tests pass
- [ ] No linting errors
- [ ] CLI commands work correctly
- [ ] Integration tests pass
- [ ] Documentation updated
- [ ] Performance unchanged

## Risk Assessment

### Low Risk

- Moving types between modules (Rust compiler will catch issues)
- Updating import statements (find/replace operation)

### Medium Risk

- Consolidating validation logic (ensure no logic is lost)
- Updating test utilities (may need test updates)

### High Risk

- Breaking public API (ensure all exports are maintained)
- Serialization changes (ensure JSON/RON compatibility)

## Rollback Plan

If issues arise:

1. Revert to backup branch
2. Identify specific problems
3. Fix issues incrementally
4. Re-run migration with fixes

## Success Criteria

- [ ] All existing functionality preserved
- [ ] Code organization improved with clear separation of concerns
- [ ] Schema validation focused on schema concerns only
- [ ] ZervVars validation moved to Zerv object creation
- [ ] Type safety maintained through Rust's type system
- [ ] No performance regression
- [ ] All tests pass
- [ ] Documentation updated

## Timeline

- **Phase 1-2**: 2-3 hours (create schema module, move types)
- **Phase 3-4**: 1-2 hours (update exports, fix imports)
- **Phase 5**: 30 minutes (cleanup old files)
- **Phase 6**: 1-2 hours (testing, validation)

**Total Estimated Time**: 4-7 hours

## Notes

- This refactoring improves code organization without changing functionality
- All public APIs remain the same
- Serialization/deserialization should be unaffected
- Consider this a "pure refactoring" - no new features or behavior changes
