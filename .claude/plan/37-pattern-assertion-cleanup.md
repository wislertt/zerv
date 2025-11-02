# Clean Pattern Assertion System Refactor

**Status**: Planned
**Priority**: Medium
**Created**: 2025-11-02

## Context

The current `assert_version_expectation` function in `src/test_utils/pattern_assertions.rs` is overly complex with 140+ lines of manual tokenization, position tracking, and multiple unused placeholder mappings. This makes the code hard to maintain and extend.

## Goals

1. **Simplify** the assertion logic from 140+ lines to ~30 lines
2. **Improve readability** with clean pattern syntax using `regex::escape()`
3. **Maintain backward compatibility** with existing `{commit_hash_7}` usage
4. **Add flexibility** with custom regex placeholder support
5. **Provide better error messages** for debugging

## Implementation Plan

### Phase 1: Core Pattern Parser

**File**: `src/test_utils/pattern_assertions.rs`

#### 1.1 Create `parse_readable_pattern()` function

```rust
fn parse_readable_pattern(pattern: &str) -> Result<Regex, ZervError> {
    // Use regex::escape() for automatic literal escaping
    // Support placeholders: {commit_hash_7} and {regex:pattern}
    // Add ^ and $ anchors for full match
    // Return compiled regex or clear error
}
```

#### 1.2 Placeholder system

- **Predefined**: `{commit_hash_7}` → `[a-f0-9]{7}`
- **Custom regex**: `{regex:pattern}` → direct regex insertion
- **Auto-escape**: All literal parts using `regex::escape()`

#### 1.3 Error handling

- Unknown placeholders → `ZervError::InvalidPattern`
- Invalid regex → `ZervError::InvalidPattern`
- Clear error messages with context

### Phase 2: Refactor Main Function

#### 2.1 Simplify `assert_version_expectation()`

```rust
pub fn assert_version_expectation(expectation: &str, actual: &str) {
    let regex = parse_readable_pattern(expectation)
        .expect("Invalid pattern format");

    assert!(
        regex.is_match(actual),
        "Version assertion failed\nExpected pattern: '{}'\nActual: '{}'\nCompiled regex: '{}'",
        expectation,
        actual,
        regex.as_str()
    );
}
```

#### 2.2 Remove old complexity

- Delete `get_fixed_length_from_placeholder_name()` function
- Remove manual tokenization and position tracking
- Eliminate complex segment length calculations

### Phase 3: Testing Strategy

#### 3.1 Preserve existing tests

```rust
#[case("0.7.74+dev.4.{commit_hash_7}", "0.7.74+dev.4.d4738bb")]
#[case("1.0.0+dev.1.{commit_hash_7}", "1.0.0+dev.1.a1b2c3d")]
#[case("prefix-{commit_hash_7}-suffix", "prefix-d4738bb-suffix")]
```

#### 3.2 Add new functionality tests

```rust
// Custom regex placeholders
#[case("1.0.0-{regex:[a-z]+\\d+}+build.{commit_hash_7}", "1.0.0-alpha123+build.a1b2c3d")]

// Multiple placeholders
#[case("{commit_hash_7}-{regex:\\d{2,4}}-{commit_hash_7}", "a1b2c3d-1234-d4738bb")]

// Special characters (auto-escaped)
#[case("version-(1.0.0){regex:[+*?]}+test.{commit_hash_7}", "version-(1.0.0)+test.a1b2c3d")]
```

#### 3.3 Error case tests

```rust
#[test]
fn test_unknown_placeholder_error() {
    let result = parse_readable_pattern("1.0.0-{unknown}");
    assert!(result.is_err());
}

#[test]
fn test_invalid_regex_error() {
    let result = parse_readable_pattern("1.0.0-{regex:[invalid}");
    assert!(result.is_err());
}
```

### Phase 4: Integration Examples

#### 4.1 Complex pattern examples

```rust
// Mixed literal and regex patterns
"build-{{\\d{3}}}-{regex:v[0-9]+\\.[0-9]+}-test-{commit_hash_7}"

// Auto-escaped special characters
"version-(1.0.0){regex:[+*?]}+test.{commit_hash_7}"

// Multiple custom regex
"release-{regex:[a-z]+}-{regex:\\d{3}}-{commit_hash_7}"
```

#### 4.2 Performance validation

- Single regex compilation per assertion
- No repeated tokenization during matching
- Efficient string building operations

## Implementation Details

### Placeholder Processing Logic

```rust
match placeholder_content {
    "commit_hash_7" => "[a-f0-9]{7}",
    custom if custom.starts_with("regex:") => &custom[6..],
    unknown => return Err(ZervError::InvalidPattern(...))
}
```

### Example Transformations

| Readable Pattern               | Compiled Regex               | Matching Examples  |
| ------------------------------ | ---------------------------- | ------------------ |
| `1.0.0-{commit_hash_7}`        | `^1\.0\.0\.[a-f0-9]{7}$`     | `1.0.0.a1b2c3d`    |
| `{regex:[a-z]+\d+}+build`      | `^[a-z]+\d+\+build$`         | `alpha123+build`   |
| `version-(1.0.0){regex:[+*?]}` | `^version-\(1\.0\.0\)[+*?]$` | `version-(1.0.0)+` |

### Benefits

#### Code Quality

- **Lines of code**: 140+ → ~30 lines
- **Complexity**: High → Low
- **Maintainability**: Poor → Excellent
- **Testability**: Hard → Easy

#### User Experience

- **Readability**: `"1.0.0-{commit_hash_7}"` vs raw regex
- **Flexibility**: Custom regex support
- **Errors**: Clear compilation and matching messages
- **Compatibility**: Existing usage unchanged

## Success Criteria

1. ✅ **All existing tests pass** without modification
2. ✅ **New functionality works** as demonstrated in examples
3. ✅ **Code is simpler** (target: <30 lines for main logic)
4. ✅ **Error messages are clear** and actionable
5. ✅ **No performance regression** in assertion execution
6. ✅ **Documentation updated** with new pattern syntax

## Files to Modify

- `src/test_utils/pattern_assertions.rs` - Main implementation
- Tests within the same file - Update and add new test cases

## Dependencies

- `regex` crate (already available)
- `regex::escape()` function (already available)

No additional dependencies required.

## Notes

- This is a **pure refactoring** - no external API changes
- **Backward compatibility** is maintained for existing usage
- The implementation leverages **standard Rust libraries** (`regex::escape()`)
- Future extensions can easily add new placeholder types as needed
