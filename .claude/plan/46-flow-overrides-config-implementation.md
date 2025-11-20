# Plan 46: Flow Overrides Config Implementation

**Status:** Planned
**Priority:** Medium
**Context:** Implement comprehensive override arguments for the `zerv flow` command, similar to the existing version command overrides.

## Goals

1. Create a new `OverridesConfig` struct for flow command with VCS and version component override options
2. Move existing `bumped_branch` override from `FlowArgs` to the new `OverridesConfig`
3. Add comprehensive override capabilities matching the version command functionality
4. Ensure proper integration with existing flow logic and validation

## Implementation Plan

### Step 1: Create overrides.rs module

- Create `/Users/wisl/Desktop/vault/personal-repo/zerv/src/cli/flow/args/overrides.rs`
- Define `OverridesConfig` struct with the specified fields:
    - VCS OVERRIDE OPTIONS: `tag_version`, `distance`, `dirty`, `no_dirty`, `clean`, `bumped_branch`, `bumped_commit_hash`, `bumped_timestamp`
    - VERSION COMPONENT OVERRIDE OPTIONS: `major`, `minor`, `patch`, `epoch`, `post`, `dev`
- Add `dirty_override()` helper method
- Add proper imports and documentation

### Step 2: Update mod.rs

- Add `pub mod overrides;` to `/Users/wisl/Desktop/vault/personal-repo/zerv/src/cli/flow/args/mod.rs`
- Export the new module

### Step 3: Integrate into FlowArgs

- Remove existing `bumped_branch` field from `FlowArgs` in `/Users/wisl/Desktop/vault/personal-repo/zerv/src/cli/flow/args/main.rs`
- Add `#[command(flatten)] pub overrides: OverridesConfig,` to `FlowArgs`
- Update `Default` implementation to include the new overrides field
- Update struct documentation to mention override capabilities

### Step 4: Update validation logic

- Modify `validate()` method in `FlowArgs` to handle overrides validation
- Add validation for override conflicts (e.g., `--clean` vs `--distance`/`--dirty`)
- Ensure overrides don't break existing flow logic

### Step 5: Update flow command implementation

- Modify flow command logic in `/Users/wisl/Desktop/vault/personal-repo/zerv/src/cli/flow/mod.rs`
- Apply overrides before version calculation
- Ensure overrides integrate properly with branch rules and schema system
- Handle override precedence (overrides should take priority over detected values)

### Step 6: Update tests

- Create tests for new `OverridesConfig` struct
- Update existing `FlowArgs` tests to account for moved `bumped_branch`
- Add integration tests for override functionality
- Test override validation and conflict detection
- Ensure existing functionality remains unaffected

### Step 7: Update help documentation

- Update help text and examples in `FlowArgs` to include override options
- Document override behavior and precedence
- Add examples showing override usage

## Testing Strategy

1. **Unit Tests:**
    - Test `OverridesConfig` struct creation and default values
    - Test `dirty_override()` method logic
    - Test validation of override conflicts

2. **Integration Tests:**
    - Test flow command with various override combinations
    - Test override precedence over detected values
    - Test interaction with branch rules and schemas

3. **Regression Tests:**
    - Ensure existing flow functionality works unchanged
    - Test that moving `bumped_branch` doesn't break existing usage
    - Verify all existing tests still pass

## Success Criteria

1. ✅ New `OverridesConfig` struct implemented with all specified fields
2. ✅ `bumped_branch` successfully moved from `FlowArgs` to `OverridesConfig`
3. ✅ Override validation working correctly (conflict detection)
4. ✅ Flow command properly applies overrides before version calculation
5. ✅ All existing tests pass (no regressions)
6. ✅ New comprehensive test coverage for override functionality
7. ✅ Documentation updated with override examples

## Files to Modify

1. `src/cli/flow/args/overrides.rs` - (new file)
2. `src/cli/flow/args/mod.rs` - add module export
3. `src/cli/flow/args/main.rs` - integrate overrides into FlowArgs
4. `src/cli/flow/mod.rs` - apply overrides in flow command logic
5. Test files in `src/cli/flow/args/main.rs` - update and add tests

## Dependencies

- `crate::cli::utils::template::Template` - for template support in version component overrides
- Existing flow command infrastructure
- Validation patterns from version command overrides

## Documentation Updates

- Update flow command help text with override options
- Add override examples to command documentation
- Document override precedence rules

## Risk Assessment

**Low Risk:**

- Following established pattern from version command overrides
- Moving existing `bumped_branch` field (no new functionality)
- Comprehensive test coverage planned

**Mitigations:**

- Extensive testing including regression tests
- Step-by-step implementation with validation at each stage
- Following existing override patterns from version command
