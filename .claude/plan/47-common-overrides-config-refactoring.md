# Plan 47: CommonOverridesConfig Refactoring

**Status**: Planned
**Priority**: Medium
**Context**: The `FlowOverridesConfig` and `VersionOverridesConfig` structs share ~90% identical fields, creating significant code duplication and maintenance overhead.

## Goals

1. Eliminate code duplication between flow and version override configurations
2. Maintain command-specific functionality while sharing common fields
3. Improve maintainability and reduce risk of divergence between similar structs

## Implementation Plan

### 1. Create CommonOverridesConfig Base Structure

- Extract all shared fields into `CommonOverridesConfig` struct
- Include all VCS override options (tag_version, distance, dirty states, etc.)
- Include shared version component overrides (major, minor, patch, epoch, post)
- Add common helper methods

### 2. Refactor Flow Overrides

- Update `FlowOverridesConfig` to compose `CommonOverridesConfig`
- Keep flow-specific `override_post()` method
- Maintain all existing CLI arguments and behavior

### 3. Refactor Version Overrides

- Update `VersionOverridesConfig` to compose `CommonOverridesConfig`
- Keep version-specific fields (dev, pre_release components, schema overrides)
- Keep `dirty_override()` helper method

### 4. Update Command Line Integration

- Ensure all CLI argument parsing still works correctly
- Update any code that directly accesses override fields to use composition
- Verify help text and argument validation remain intact

### 5. Comprehensive Testing

- Unit tests for new `CommonOverridesConfig`
- Integration tests for both flow and version commands
- Regression tests to ensure existing functionality unchanged

## Testing Strategy

1. **Unit Tests**: Test `CommonOverridesConfig` default values and methods
2. **Integration Tests**: Run existing flow and version command test suites
3. **CLI Tests**: Verify all command-line arguments work as before
4. **Regression Tests**: Ensure no breaking changes in existing functionality

## Success Criteria

- ✅ Code duplication reduced by ~70%
- ✅ All existing tests pass without modification
- ✅ CLI help and argument behavior unchanged
- ✅ Both flow and version commands work identically to before
- ✅ No performance regression

## Files to Modify

1. `src/cli/flow/args/overrides.rs` - Update to use composition
2. `src/cli/version/args/overrides.rs` - Update to use composition
3. `src/cli/args/common.rs` (create if needed) - Add `CommonOverridesConfig`
4. Update any direct field access in command implementations

## Breaking Changes

None - this is purely internal refactoring that maintains all existing public APIs and CLI behavior.

## Documentation Updates

- Update internal documentation about override configuration architecture
- No CLI documentation changes needed (user-facing behavior unchanged)
