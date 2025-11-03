# Implement Flexible Schema System for Zerv Version

**Status**: Planned
**Priority**: High
**Context**: Implement flexible --schema system for zerv version command to support granular control over version components and build context inclusion/exclusion.

## Current State

- `zerv version --schema` has only 3 tiers (zerv_standard_tier_1/2/3 and zerv_calver_tier_1/2/3)
- No fine-grained control over build context inclusion
- Old schema naming convention is not intuitive and lacks flexibility
- Both `standard` and `calver` schema families need the same flexibility

## Proposed Schema System

### Base Versions (no context)

1. **standard** - smart auto-detection (2/3/4 based on available components)
2. **standard-base** - `1.1.0`
3. **standard-base-prerelease** - `1.1.0-alpha.1`
4. **standard-base-prerelease-post** - `1.1.0-alpha.1.post.2`
5. **standard-base-prerelease-post-dev** - `1.1.0-alpha.1.post.2.dev.1729924622`

### Context Versions (with +build context)

6. **standard-base-context** - `1.1.0+main.2.a1b2c3d`
7. **standard-base-prerelease-context** - `1.1.0-alpha.1+main.2.a1b2c3d`
8. **standard-base-prerelease-post-context** - `1.1.0-alpha.1.post.2+main.2.a1b2c3d`
9. **standard-base-prerelease-post-dev-context** - `1.1.0-alpha.1.post.2.dev.1729924622+main.2.a1b2c3d`
10. **standard-context** - smart auto-detection with context

### CalVer Schema Family (Same pattern as standard)

#### Base Versions (no context)

11. **calver** - smart auto-detection (12/13/14 based on available components)
12. **calver-base** - `2024.11`
13. **calver-base-prerelease** - `2024.11-alpha.1`
14. **calver-base-prerelease-post** - `2024.11-alpha.1.post.2`
15. **calver-base-prerelease-post-dev** - `2024.11-alpha.1.post.2.dev.1729924622`

#### Context Versions (with +build context)

16. **calver-base-context** - `2024.11+main.2.a1b2c3d`
17. **calver-base-prerelease-context** - `2024.11-alpha.1+main.2.a1b2c3d`
18. **calver-base-prerelease-post-context** - `2024.11-alpha.1.post.2+main.2.a1b2c3d`
19. **calver-base-prerelease-post-dev-context** - `2024.11-alpha.1.post.2.dev.1729924622+main.2.a1b2c3d`
20. **calver-context** - smart auto-detection with context

## Implementation Plan

### Phase 1: Update zerv version schema system

#### 1.1 Define new schema enum

- Add `VersionSchema` enum with all 20 variants (10 standard + 10 calver)
- Implement `FromStr` trait for string matching
- Handle kebab-case to camelCase conversion internally
- Update help text and documentation
- Ensure both standard and calver families follow same component pattern

#### 1.2 Update schema logic

- Modify version formatting logic to handle each schema variant
- Implement smart detection for `standard`/`standard-context` and `calver`/`calver-context`
- Ensure backward compatibility with existing `--schema` options
- Add context inclusion/exclusion logic
- Handle both SemVer and CalVer base formats in the same framework

#### 1.3 Deprecate old schema system

- Mark `zerv_standard_tier_1/2/3` and `zerv_calver_tier_1/2/3` as deprecated
- Add deprecation warnings when old schemas are used
- Map old schemas to new schema equivalents:
    - `zerv_standard_tier_1` → `standard-base-prerelease`
    - `zerv_standard_tier_2` → `standard-base-prerelease-post`
    - `zerv_standard_tier_3` → `standard-base-prerelease-post-dev`
    - `zerv_calver_tier_1` → `calver-base-prerelease`
    - `zerv_calver_tier_2` → `calver-base-prerelease-post`
    - `zerv_calver_tier_3` → `calver-base-prerelease-post-dev`
- Update tests to use new schema names
- Add migration guide for users

#### 1.4 Update CLI arguments

- Update `--schema` argument help text to list new options
- Ensure all existing commands continue to work (with deprecation warnings)
- Add validation for new schema names
- Hide deprecated schemas from help text but keep them functional

#### 1.5 Update tests

- Add tests for all new schema variants (20 total)
- Update existing tests that use `--schema`
- Add edge case tests for smart detection
- Test backward compatibility
- Test deprecation warnings for old schemas
- Add tests for old schema to new schema mapping
- Test both standard and calver schema families
- Test context inclusion/exclusion for both families

#### 1.6 Update documentation

- Update CLI help text
- Update any existing documentation
- Add examples for new schemas
- Document deprecation timeline and removal plan

### Phase 2: Testing and Validation

#### 2.1 Integration testing

- Test all 20 schema variants across different repository states
- Test smart detection for `standard`/`standard-context` and `calver`/`calver-context`
- Test backward compatibility with existing scripts and CI/CD pipelines
- Test performance impact with large repositories

#### 2.2 Validation testing

- Test all component combinations (base, prerelease, post, dev, context)
- Test edge cases (empty components, malformed versions, etc.)
- Test error handling and deprecation warnings
- Test migration from old schemas to new schemas

## Testing Strategy

### Unit Tests

- Test each schema variant produces correct output
- Test branch pattern matching
- Test pre-release label/number resolution
- Test post distance calculation (both modes)
- Test build context inclusion/exclusion

### Integration Tests

- Test `zerv version --schema` with all 20 variants
- Test compatibility with existing scripts and CI/CD configurations
- Test edge cases (dirty working directory, tags, etc.)
- Test schema behavior across different VCS states

### Regression Tests

- Ensure existing `zerv version` functionality unchanged
- Test backward compatibility with existing scripts
- Validate performance impact is minimal
- Test deprecation warnings and mapping functionality

## Migration and Deprecation Strategy

### Phase 1: Soft Deprecation (Current Implementation)

- Old schemas continue to work with deprecation warnings
- Updated help text shows only new schemas
- Documentation updated with migration guide
- Internal code refactored to use new enum variants

### Phase 2: Hard Deprecation (Next Major Version)

- Old schemas become errors instead of warnings
- Clear error messages guide users to equivalent new schemas
- Migration guide remains available

### Phase 3: Removal (Future Major Version)

- Remove old schema code completely
- Simplify implementation by removing mapping logic

## Success Criteria

1. ✅ All 20 schema variants work correctly in `zerv version` (10 standard + 10 calver)
2. ✅ Backward compatibility maintained for existing `zerv version` usage
3. ✅ Comprehensive test coverage for new functionality
4. ✅ Documentation updated and examples working
5. ✅ Old schemas deprecated with clear migration path
6. ✅ No breaking changes for existing users
7. ✅ Consistent behavior between standard and calver schema families
8. ✅ Build context inclusion/exclusion works correctly for all variants

## Future Considerations

- Additional schema variants if needed
- Performance optimization for large repositories
- Timeline for old schema removal (based on user feedback)
- Foundation for future `zerv flow` command implementation

---

**Dependencies**: None
**Estimated Effort**: Medium-High
**Risk Level**: Medium (schema system changes require careful backward compatibility handling)
