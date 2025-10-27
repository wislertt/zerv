# Reset Logic Schema Component Issue

## Issue Summary

During test refactoring in `src/version/zerv/bump/vars_primary.rs`, we discovered a design flaw in the reset logic for version bumping.

## Problem Description

When bumping major/minor/patch versions, the current reset logic only resets `ZervVars` fields but doesn't handle schema components that depend on those fields.

### Current Behavior

- `ZervVars.pre_release` gets reset to `None` ✅
- `Component::VarField("pre_release")` in schema remains untouched ❌
- Build metadata and other non-ZervVars components persist ❌

### Expected Behavior (Semantic Versioning)

- Major bump: `1.5.2-rc.1+build.456` → `2.0.0` (all lower precedence removed)
- Minor bump: `1.5.2-rc.1+build.456` → `1.6.0` (patch, pre-release, build removed)

## Technical Details

### Components Affected

- `Component::String(String)` - static, should remain
- `Component::Integer(u64)` - static, should remain
- `Component::VarField(String)` - **ISSUE**: should be reset when referenced ZervVar is reset
- `Component::VarTimestamp(String)` - static pattern, should remain

### Current Reset Implementation

Located in `/src/version/zerv/bump/reset.rs`:

- Only resets `ZervVars` fields (major, minor, patch, pre_release, post, dev, epoch)
- Doesn't touch schema components in `extra_core` or `build`

## Test Cases That Revealed the Issue

```rust
// These tests currently fail due to the bug:
#[case("1.0.0+build.123", 1, "2.0.0")]           // Expected: no build metadata
#[case("1.5.2-rc.1+build.456", 1, "2.0.0")]      // Expected: no pre-release or build

// Current (incorrect) behavior:
#[case("1.0.0+build.123", 1, "2.0.0+build.123")] // Build metadata persists
#[case("1.5.2-rc.1+build.456", 1, "2.0.0+build.456")] // Build metadata persists
```

## Proposed Solutions

### Option A: Extend Reset Logic

Modify `reset_lower_precedence_components` to also clean schema components:

- Remove `VarField` components that reference reset ZervVars
- Keep static `String`/`Integer` components

### Option B: Schema-Aware Reset

Create separate method to reset schema components based on precedence rules.

### Option C: Architectural Change

Move all dynamic data into `ZervVars` so reset logic is centralized.

## Impact

This affects semantic versioning compliance and could lead to incorrect version strings being generated during automated bumping.

## Status

- **Discovered**: During test refactoring
- **Severity**: Medium (affects SemVer compliance)
- **Complexity**: High (requires careful design consideration)
- **Decision**: Deferred for later architectural review

## Files Involved

- `src/version/zerv/bump/reset.rs` - Current reset implementation
- `src/version/zerv/bump/vars_primary.rs` - Test cases that revealed issue
- `src/version/zerv/components.rs` - Component definitions
- `src/version/semver/from_zerv.rs` - Conversion logic that shows the problem

## Next Steps

1. Review overall reset architecture
2. Decide on approach (A, B, or C above)
3. Implement solution with comprehensive tests
4. Ensure SemVer compliance across all scenarios
