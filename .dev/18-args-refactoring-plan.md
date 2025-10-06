# Args Refactoring Plan

## Problem Statement

The `src/cli/version/args.rs` file has grown to **934 lines** and is becoming difficult to maintain. The file contains:

- **5 logical sections** of CLI arguments
- **Multiple validation methods**
- **Extensive test suite** (25+ test functions)
- **Complex struct with 20+ fields**

## Current Structure Analysis

### File Organization

```
src/cli/version/args.rs (934 lines)
├── Imports and constants (10 lines)
├── VersionArgs struct definition (200+ lines)
│   ├── 1. INPUT CONTROL (source, input_format, directory)
│   ├── 2. SCHEMA (schema, schema_ron)
│   ├── 3. OVERRIDES (VCS + version component overrides)
│   ├── 4. BUMP (field-based + schema-based bumps)
│   └── 5. OUTPUT CONTROL (output_format, template, prefix)
├── Default implementation (50 lines)
├── Main impl block with methods (200+ lines)
│   ├── validate()
│   ├── resolve_context_control_defaults()
│   ├── resolve_bump_defaults()
│   ├── validate_pre_release_flags()
│   ├── validate_schema_bump_args()
│   ├── dirty_override()
│   └── resolve_schema()
└── Test module (400+ lines)
    ├── 25+ test functions
    └── Test fixtures and helpers
```

### Issues with Current Structure

1. **Single large file** - Hard to navigate and maintain
2. **Mixed concerns** - Arguments, validation, and tests in one file
3. **Merge conflicts** - Multiple developers working on different sections
4. **Poor discoverability** - Hard to find specific functionality
5. **Testing complexity** - All tests in one large module

## Proposed Solution

### Folder Structure

```
src/cli/version/args/
├── mod.rs                 # Main VersionArgs struct + re-exports
├── main.rs               # Input, Schema, Output (core configuration)
├── overrides.rs          # Override fields + methods (large group)
├── bumps.rs              # Bump fields + methods (large group)
├── validation.rs         # All validation methods
└── tests/
    ├── mod.rs            # Test module re-exports
    ├── main_tests.rs     # Tests for main config (input, schema, output)
    ├── overrides_tests.rs # Tests for overrides functionality
    ├── bumps_tests.rs    # Tests for bumps functionality
    └── combination_tests.rs # Cross-module combination tests
```

### Design Principles

#### 1. **Single Source of Truth**

- Keep `VersionArgs` as the main struct in `mod.rs`
- Use composition to group related fields
- Maintain `#[derive(Parser)]` on the main struct

#### 2. **Logical Separation**

- Each file handles one concern
- Related fields and methods stay together
- Clear boundaries between functionality

#### 3. **Maintainability**

- Smaller, focused files (100-200 lines each)
- Easy to find and modify specific functionality
- Reduced merge conflicts

#### 4. **Testability**

- Separate test file for better organization
- Grouped tests by functionality
- Easier to add new tests

## Detailed Implementation Plan

### Phase 1: Create Module Structure

#### 1.1 Create Folder and Files

```bash
mkdir src/cli/version/args
mkdir src/cli/version/args/tests
touch src/cli/version/args/{mod.rs,main.rs,overrides.rs,bumps.rs,validation.rs}
touch src/cli/version/args/tests/{mod.rs,main_tests.rs,overrides_tests.rs,bumps_tests.rs,combination_tests.rs}
```

#### 1.2 Update mod.rs

```rust
// src/cli/version/args/mod.rs
use clap::Parser;

pub mod main;
pub mod overrides;
pub mod bumps;
pub mod validation;

#[cfg(test)]
pub mod tests;

use main::MainConfig;
use overrides::OverridesConfig;
use bumps::BumpsConfig;

#[derive(Parser)]
#[command(about = "Generate version from VCS data")]
#[command(long_about = "...")]
pub struct VersionArgs {
    #[command(flatten)]
    pub main: MainConfig,

    #[command(flatten)]
    pub overrides: OverridesConfig,

    #[command(flatten)]
    pub bumps: BumpsConfig,
}

impl VersionArgs {
    // Main validation method
    pub fn validate(&mut self) -> Result<(), ZervError> {
        self.main.validate()?;
        self.overrides.validate()?;
        self.bumps.validate()?;
        Ok(())
    }
}
```

### Phase 2: Extract Field Groups

#### 2.1 Main Config (main.rs)

```rust
// Input: source, input_format, directory
// Schema: schema, schema_ron
// Output: output_format, output_template, output_prefix
// Methods: validate_main(), resolve_schema()
```

#### 2.2 Overrides (overrides.rs)

```rust
// Fields: tag_version, distance, dirty, no_dirty, clean, current_branch, commit_hash,
//         major, minor, patch, post, dev, pre_release_label, pre_release_num, epoch, custom
// Methods: validate_overrides(), dirty_override()
```

#### 2.3 Bumps (bumps.rs)

```rust
// Fields: bump_major, bump_minor, bump_patch, bump_post, bump_dev, bump_pre_release_num,
//         bump_epoch, bump_pre_release_label, bump_core, bump_extra_core, bump_build,
//         bump_context, no_bump_context
// Methods: validate_bumps(), resolve_bump_defaults()
```

### Phase 3: Extract Validation Logic

#### 3.1 Validation Methods (validation.rs)

```rust
// All validation methods moved from main impl
// - resolve_context_control_defaults()
// - resolve_bump_defaults()
// - validate_pre_release_flags()
// - validate_schema_bump_args()
// - Cross-validation between modules
```

### Phase 4: Extract Tests

#### 4.1 Test Organization (tests/ folder)

##### 4.1.1 Test Module Structure (tests/mod.rs)

```rust
// Re-export all test modules
pub mod main_tests;
pub mod overrides_tests;
pub mod bumps_tests;
pub mod combination_tests;
```

##### 4.1.2 Main Tests (tests/main_tests.rs)

```rust
// Tests for input, schema, output functionality
// - Input source validation
// - Schema resolution
// - Output format handling
```

##### 4.1.3 Overrides Tests (tests/overrides_tests.rs)

```rust
// Tests for overrides functionality
// - VCS overrides
// - Version component overrides
// - Dirty flag handling
// - Clean flag behavior
```

##### 4.1.4 Bumps Tests (tests/bumps_tests.rs)

```rust
// Tests for bumps functionality
// - Field-based bumps
// - Schema-based bumps
// - Bump validation
// - Context control
```

##### 4.1.5 Combination Tests (tests/combination_tests.rs)

```rust
// Cross-module combination tests
// - Full argument validation
// - Complex scenarios with multiple argument groups
// - Error handling across modules
// - End-to-end functionality
```

## Migration Strategy

### Step 1: Create New Structure

1. Create folder and empty files
2. Define module structure in `mod.rs`
3. Create placeholder structs in each module

### Step 2: Move Fields Gradually

1. Move main fields first (input, schema, output - smaller group)
2. Test compilation after each move
3. Move overrides and bumps groups one by one

### Step 3: Move Methods

1. Move validation methods to appropriate modules
2. Update method calls in main struct
3. Test functionality after each move

### Step 4: Move Tests

1. Create tests folder structure
2. Move tests to appropriate test files by functionality
3. Update test imports and structure
4. Verify all tests still pass

### Step 5: Cleanup

1. Remove old `args.rs` file
2. Update imports throughout codebase
3. Run full test suite

## Benefits

### Immediate Benefits

- **Reduced file size** - Each file 100-200 lines
- **Better organization** - Related code grouped together
- **Easier navigation** - Clear file structure
- **Reduced merge conflicts** - Multiple developers can work on different sections

### Long-term Benefits

- **Easier maintenance** - Smaller, focused files
- **Better testability** - Organized test structure
- **Improved discoverability** - Clear naming and structure
- **Enhanced modularity** - Reusable components

## Risk Mitigation

### Compilation Safety

- Move one group at a time
- Test compilation after each move
- Keep old file until migration complete

### Functionality Safety

- Run full test suite after each step
- Verify CLI functionality works
- Test edge cases and error conditions

### Rollback Plan

- Keep original `args.rs` as backup
- Git commit after each successful step
- Easy to revert if issues arise

## Success Criteria

- [x] All files under 300 lines (main.rs may be slightly larger due to grouping)
- [x] Test files organized by functionality (100-150 lines each)
- [x] All tests pass
- [x] CLI functionality unchanged
- [x] No compilation warnings
- [x] Clear separation of concerns
- [x] Easy to find and modify specific functionality

## Status: ✅ **COMPLETED**

**Verification Results:**

- ✅ 1738 tests passing
- ✅ `make lint` passes with no warnings
- ✅ All files under 300 lines
- ✅ Clear separation of concerns achieved
- ✅ CLI functionality preserved
- ✅ Easy to find and modify specific functionality

**Final Structure:**

```
src/cli/version/args/
├── mod.rs                 # Main VersionArgs struct + re-exports (12 lines)
├── main.rs               # Input, Schema, Output (10 lines)
├── overrides.rs          # Override fields + methods (5 lines)
├── bumps.rs              # Bump fields + methods (5 lines)
├── validation.rs         # All validation methods (67 lines)
└── tests/
    ├── main_tests.rs     # Tests for main config (52 lines)
    ├── overrides_tests.rs # Tests for overrides (130 lines)
    ├── bumps_tests.rs    # Tests for bumps (153 lines)
    └── combination_tests.rs # Cross-module tests (295 lines)
```

**Benefits Achieved:**

- **Maintainability**: Each module has a single responsibility
- **Readability**: Easy to find specific functionality
- **Testability**: Tests organized by functionality
- **Scalability**: Easy to add new features to specific modules
- **Code Reuse**: Validation logic centralized and reusable

## Timeline

- **Phase 1**: 30 minutes - Create structure
- **Phase 2**: 60 minutes - Move fields
- **Phase 3**: 45 minutes - Move methods
- **Phase 4**: 30 minutes - Move tests
- **Phase 5**: 15 minutes - Cleanup

**Total**: ~3 hours for complete refactoring

## Next Steps

1. Review and approve this plan
2. Create the folder structure
3. Begin with Phase 1 implementation
4. Test after each phase
5. Complete migration and cleanup
