# Consolidate Version Parsing Functions

## Status

**Planned**

## Priority

**Medium** - Code quality improvement that should be done before the git tag resolution fix

## Context

There are currently two similar version parsing functions in the codebase that serve overlapping purposes:

1. **`VersionObject::parse_with_format`** - Located in `src/version/version_object.rs`
    - Takes `tag: &str` and `format_str: &str`
    - Returns `Option<VersionObject>`
    - Supports "semver", "pep440" formats (case insensitive)
    - Returns `None` for unknown formats or parse failures

2. **`InputFormatHandler::parse_version_string`** - Located in `src/cli/utils/format_handler.rs`
    - Takes `version_str: &str` and `input_format: &str`
    - Returns `Result<VersionObject, ZervError>`
    - Supports "semver", "pep440", "auto" formats (case insensitive)
    - Returns detailed error messages for different failure scenarios

## Current Problems

1. **Code Duplication**: Both functions implement similar parsing logic for SemVer and PEP440
2. **Inconsistent Error Handling**: One returns `Option`, the other returns `Result`
3. **Different Format Support**: `InputFormatHandler` supports "auto" format, `VersionObject` doesn't
4. **Maintenance Overhead**: Changes to version parsing logic need to be made in two places
5. **Confusing API**: Developers don't know which function to use in different contexts

## Goals

1. **Consolidate parsing logic** into a single, authoritative implementation
2. **Standardize error handling** across the codebase
3. **Maintain all existing functionality** including "auto" format detection
4. **Provide clear migration path** for existing code
5. **Improve API consistency** for better developer experience

## Analysis of Current Usage

### VersionObject::parse_with_format Usage

- Used in places where optional parsing is acceptable
- Primarily used in version parsing pipelines
- Called from tag resolution, version object creation

### InputFormatHandler::parse_version_string Usage

- Used in CLI command handlers where detailed error messages are needed
- Used when parsing user input from command line
- Used in contexts where error messages must be user-friendly

## Proposed Solution

### Option 1: Enhance VersionObject::parse_with_format (Recommended)

Enhance `VersionObject::parse_with_format` to support all functionality from `InputFormatHandler::parse_version_string`:

```rust
impl VersionObject {
    /// Enhanced parsing with auto-detection and detailed error handling
    pub fn parse_with_format(
        tag: &str,
        format_str: &str
    ) -> Result<Self, ZervError> {
        match format_str.to_lowercase().as_str() {
            "semver" => SemVer::from_str(tag)
                .map(VersionObject::SemVer)
                .map_err(|e| ZervError::InvalidFormat(format!("Invalid SemVer format '{tag}': {e}"))),
            "pep440" => PEP440::from_str(tag)
                .map(VersionObject::PEP440)
                .map_err(|e| ZervError::InvalidFormat(format!("Invalid PEP440 format '{tag}': {e}"))),
            "auto" => Self::parse_auto_detect(tag),
            _ => Err(ZervError::UnknownFormat(format!(
                "Unknown input format '{format_str}'. Supported formats: semver, pep440, auto"
            ))),
        }
    }

    /// Auto-detect version format (try SemVer first, then PEP440)
    fn parse_auto_detect(version_str: &str) -> Result<Self, ZervError> {
        // Try SemVer first
        if let Ok(semver) = SemVer::from_str(version_str) {
            return Ok(VersionObject::SemVer(semver));
        }

        // Fall back to PEP440
        if let Ok(pep440) = PEP440::from_str(version_str) {
            return Ok(VersionObject::PEP440(pep440));
        }

        Err(ZervError::InvalidVersion(format!(
            "Version '{version_str}' is not valid SemVer or PEP440 format"
        )))
    }
}
```

### Option 2: Create a New Centralized Function

Create a new centralized parsing function and migrate both existing functions to use it:

```rust
pub fn parse_version_string(
    version_str: &str,
    input_format: &str
) -> Result<VersionObject, ZervError> {
    // Implementation here
}
```

### Option 3: Keep Both but Clarify Responsibilities

Keep both functions but clearly document their intended use cases:

- `VersionObject::parse_with_format` - For internal parsing where None is acceptable
- `InputFormatHandler::parse_version_string` - For user input parsing with error messages

## Implementation Plan

### Step 1: Choose the Approach

- Analyze which option provides the best balance of consistency and maintainability
- Consider the impact on existing code that calls these functions
- Evaluate which approach provides the cleanest migration path

### Step 2: Implement the Solution

- If Option 1: Enhance `VersionObject::parse_with_format` to return `Result`
- If Option 2: Create new centralized function and update both existing functions
- If Option 3: Add clear documentation and possibly thin wrapper functions

### Step 3: Update All Callers

- Find all calls to `InputFormatHandler::parse_version_string`
- Update them to use the consolidated function
- Ensure error handling is preserved appropriately
- Update tests to work with the new API

### Step 4: Remove Redundant Code

- Remove the deprecated function (if Option 1 or 2)
- Update imports and dependencies
- Clean up any remaining references

### Step 5: Update Documentation

- Update inline documentation for the consolidated function
- Add migration guide for developers
- Update any external documentation that references the old functions

## Migration Strategy

### For Option 1 (Enhance VersionObject::parse_with_format)

1. **Phase 1**: Add enhanced version that returns `Result`
2. **Phase 2**: Update all callers to handle `Result` instead of `Option`
3. **Phase 3**: Remove `InputFormatHandler::parse_version_string`
4. **Phase 4**: Clean up imports and dependencies

### Backward Compatibility Considerations

- Function signature changes will require updating all callers
- Error handling logic may need to be updated in some places
- Tests will need to be updated to handle new error types

## Testing Strategy

### Unit Tests

- Test all format types: "semver", "pep440", "auto"
- Test invalid inputs and error messages
- Test case-insensitive format handling
- Test edge cases and boundary conditions

### Integration Tests

- Test that existing functionality continues to work
- Test error handling in CLI contexts
- Test auto-detection with ambiguous versions

### Regression Tests

- Ensure no functionality is lost during consolidation
- Verify that error messages are preserved or improved
- Test performance impact of the changes

## Success Criteria

1. ✅ Single, authoritative version parsing function
2. ✅ All existing functionality preserved
3. ✅ Consistent error handling across the codebase
4. ✅ Clear migration path for developers
5. ✅ No breaking changes in public APIs (or clear migration guide)
6. ✅ Reduced code duplication and maintenance burden

## Risk Mitigation

- **Breaking changes**: Carefully analyze all callers and provide migration guidance
- **Error message quality**: Ensure user-facing error messages are maintained or improved
- **Performance**: Ensure consolidation doesn't negatively impact parsing performance
- **Compatibility**: Test with all existing use cases to ensure no regressions

## Decision Criteria

Choose Option 1 if:

- You want a single authoritative parsing function
- You're comfortable changing function signatures
- You want to minimize code duplication

Choose Option 2 if:

- You want to maintain existing function signatures
- You prefer a gradual migration approach
- You want a clean separation between internal and user-facing parsing

Choose Option 3 if:

- The two functions serve genuinely different purposes
- The risk of breaking existing code is too high
- You prefer to clarify rather than consolidate

## Recommendation

**Option 1 (Enhance VersionObject::parse_with_format)** is recommended because:

1. `VersionObject` is the natural home for version parsing logic
2. It eliminates duplication completely
3. It provides a single source of truth for version parsing
4. The migration is straightforward and manageable
5. It improves overall code organization and maintainability
