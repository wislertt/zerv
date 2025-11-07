# Implement Flexible Schema System for Zerv Version

**Status**: ✅ **COMPLETE**
**Priority**: High
**Context**: Implement flexible --schema system for zerv version command to support granular control over version components and build context inclusion/exclusion.

## Final Implementation Summary

**✅ CORE IMPLEMENTATION COMPLETE**

- ✅ **Smart schema system** with intelligent component selection based on repository state
- ✅ **22 schema variants** (11 standard + 11 calver) including new NoContext variants
- ✅ **Three context control modes**:
    - **Smart** (`standard`/`calver`) - Context only for dirty/distance states
    - **Always** (`standard-context`/`calver-context`) - Context always included
    - **Never** (`standard-no-context`/`calver-no-context`) - Context never included
- ✅ **Intelligent logic**:
    - Dirty → `BasePrereleasePostDev` schema
    - Distance → `BasePrereleasePost` schema
    - Clean tagged → Minimal appropriate schema (base/prerelease/post only)
- ✅ **All 2265 tests passing**
- ✅ **All linting checks passing**
- ✅ **Backward compatibility maintained** through deprecation warnings

## Available Schema Options

### Standard Schema Family

- **`standard`** - Smart context (dirty/distance get context, clean tagged don't)
- **`standard-no-context`** - Never includes context
- **`standard-context`** - Always includes context
- **`standard-base`** - Base version only (e.g., `1.2.3`)
- **`standard-base-prerelease`** - Base + prerelease (e.g., `1.2.3-alpha.1`)
- **`standard-base-prerelease-post`** - Base + prerelease + post (e.g., `1.2.3-alpha.1.post.2`)
- **`standard-base-prerelease-post-dev`** - Base + prerelease + post + dev (e.g., `1.2.3-alpha.1.post.2.dev.123`)
- Plus context variants of all base schemas

### CalVer Schema Family (same pattern)

- **`calver`** - Smart context (dirty/distance get context, clean tagged don't)
- **`calver-no-context`** - Never includes context
- **`calver-context`** - Always includes context
- **`calver-base`** - Base version only (e.g., `2024.11`)
- **`calver-base-prerelease`** - Base + prerelease (e.g., `2024.11-alpha.1`)
- Plus all other component combinations and context variants

## Smart Logic Implementation

### Repository State Detection

- **Dirty** → `BasePrereleasePostDev` schema
- **Has distance from tag** → `BasePrereleasePost` schema
- **Clean tagged with prerelease + post** → `BasePrereleasePost` schema
- **Clean tagged with prerelease only** → `BasePrerelease` schema
- **Clean tagged (base only)** → `Base` schema

### Context Control

- **Smart variants** - Add context only for dirty/distance states
- **Always context** - Always add build context
- **Never context** - Never add build context

## Implementation Summary

### Core Changes Made

- **✅ Smart schema system** implemented in `src/schema/flexible.rs`
- **✅ New variants added**: `StandardNoContext`, `CalverNoContext` with complete enum and parsing support
- **✅ Smart logic implemented**: Intelligent component selection based on repository state
- **✅ Preset functions updated**: Both `get_standard_schema()` and `get_calver_schema()` now use smart system
- **✅ All tests updated**: Fixed failing tests to match new smart behavior
- **✅ Backward compatibility**: Old schemas work with deprecation warnings

### Files Modified

- `src/schema/flexible.rs` - Core implementation with smart logic
- `src/schema/presets/standard.rs` - Updated to use smart system
- `src/schema/presets/calver.rs` - Updated to use smart system
- `src/schema/presets/mod.rs` - Cleaned up unused tier logic
- `src/cli/version/zerv_draft.rs` - Updated test expectations
- `src/schema/mod.rs` - Re-exported SchemaContextExt trait

### Migration Path

- **Old schemas**: `zerv_standard_tier_1/2/3`, `zerv_calver_tier_1/2/3` still work with warnings
- **New schemas**: Use `standard`, `standard-context`, `standard-no-context` (same for calver)
- **Preset functions**: Now use intelligent smart schema detection

## Next Steps: Deprecated Method Removal

### 🎯 Goal: Remove Legacy Tier Methods

Since the smart schema system is complete and all tests use the new system, we can now remove the deprecated tier methods.

**Current Usage Analysis:**

- `zerv_standard_tier_1()` used in: `src/test_utils/zerv/schema.rs`, `src/version/zerv/schema/core.rs`
- Similar usage for other tier methods

### 📋 Removal Plan

#### Step 1: Remove Standard Tier Methods

**Target**: `src/schema/presets/standard.rs:8-21`
**Status**: ✅ **COMPLETE**
**Actions:**

- ✅ Remove `zerv_standard_tier_1()` method
- ✅ Remove `zerv_standard_tier_2()`, `zerv_standard_tier_3()` methods
- ✅ Update test fixtures to use new schema variants
- ✅ All 2261 tests passing

#### Step 2: Remove CalVer Tier Methods

**Target**: `src/schema/presets/calver.rs`
**Actions:**

- Remove `zerv_calver_tier_1()`, `zerv_calver_tier_2()`, `zerv_calver_tier_3()` methods
- Update any direct usage in tests to use new schema variants

#### Step 3: Update Test Fixtures

**Target**: `src/test_utils/zerv/schema.rs`, `src/version/zerv/schema/core.rs`
**Actions:**

- Replace `ZervSchema::zerv_standard_tier_1()` with `VersionSchema::StandardBase.schema()`
- Replace similar CalVer tier methods with appropriate new variants
- Update test expectations if needed

#### Step 4: Update Deprecation Mapping

**Target**: `src/schema/presets/mod.rs`
**Actions:**

- Remove deprecation mapping logic for old schema names
- Old schemas will now produce proper errors instead of warnings
- Update help text if needed

### 🎯 Expected Benefits

1. **Cleaner codebase** - Remove legacy tier-based system completely
2. **Simpler implementation** - No need to maintain backward compatibility
3. **Clear migration path** - Users forced to use new, better schema system
4. **Reduced complexity** - Fewer code paths to maintain
