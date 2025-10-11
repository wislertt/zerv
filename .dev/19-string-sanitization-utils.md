# Plan 19: String Sanitization Utils

## Problem

Multiple components need string sanitization with format-specific rules:

1. **Plan 21 (Handlebars)**: `{{sanitize input}}` helper needs configurable string cleaning
2. **Plan 20 (Component Resolution)**: Format conversions need sanitized strings for compatibility
3. **Version Standards**: SemVer, PEP440 have different character/format requirements
4. **User Input**: Branch names, custom fields need sanitization for version compatibility

## Current Issues

- No centralized sanitization logic
- Each format handles string cleaning differently
- Handlebars helpers would duplicate sanitization code
- No consistent rules for invalid characters, case, separators

## Solution: Pure String Sanitization Utils

### Core Concept

Create reusable string-to-string sanitization utilities that can be used by:

- Handlebars helpers (`{{sanitize input}}`)
- Component resolution (format-specific cleaning)
- Any other string cleaning needs

```rust
// Pure string utilities - no dependency on Component types
pub fn sanitize_string(input: &str, config: &SanitizeConfig) -> String { ... }
```

## Core Components

### SanitizeTarget Enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SanitizeTarget {
    /// Clean string for version identifiers (alphanumeric + separator)
    Str,
    /// Extract digits as unsigned integer string
    UInt,
}
```

### Sanitizer Struct

```rust
#[derive(Debug, Clone)]
pub struct Sanitizer {
    /// What type of output to produce
    pub target: SanitizeTarget,

    /// Replace non-alphanumeric characters with this separator, or None to keep unchanged (Str target only)
    pub separator: Option<String>,

    /// Convert to lowercase (String target only)
    pub lowercase: bool,

    /// Keep leading zeros in numeric segments
    pub keep_zeros: bool,

    /// Maximum length (truncate if longer)
    pub max_length: Option<usize>,
}

impl Default for Sanitizer {
    fn default() -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: Some(".".to_string()),
            lowercase: false,
            keep_zeros: false,
            max_length: None,
        }
    }
}

impl Sanitizer {
    /// Apply sanitization to input string
    pub fn sanitize(&self, input: &str) -> String {
        match self.target {
            SanitizeTarget::Str => self.sanitize_to_string(input),
            SanitizeTarget::UInt => self.sanitize_to_integer(input),
        }
    }

    /// Sanitize to clean string
    fn sanitize_to_string(&self, input: &str) -> String

    /// Extract unsigned integer from string
    fn sanitize_to_integer(&self, input: &str) -> String

    /// Replace non-alphanumeric characters with separator or keep unchanged
    fn replace_non_alphanumeric(&self, input: &str) -> String

    /// Remove leading zeros from numeric segments
    fn remove_leading_zeros(&self, input: &str) -> String
}
```

### Predefined Sanitizers

```rust
impl Sanitizer {
    /// PEP440 compatible: lowercase, dots, no leading zeros
    pub fn pep440() -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: Some(".".to_string()),
            lowercase: true,
            keep_zeros: false,
            max_length: None,
        }
    }

    /// SemVer compatible: preserve case, dots
    pub fn semver() -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: Some(".".to_string()),
            lowercase: false,
            keep_zeros: false,
            max_length: None,
        }
    }

    /// Extract unsigned integer from string
    pub fn uint(keep_zeros: bool) -> Self {
        Self {
            target: SanitizeTarget::UInt,
            separator: None, // Not used for integers
            lowercase: false, // Not used for integers
            keep_zeros,
            max_length: None,
        }
    }

    /// Custom string sanitizer
    pub fn str(
        separator: Option<&str>,
        lowercase: bool,
        keep_zeros: bool,
        max_length: Option<usize>
    ) -> Self {
        Self {
            target: SanitizeTarget::Str,
            separator: separator.map(|s| s.to_string()),
            lowercase,
            keep_zeros,
            max_length,
        }
    }
}
```

## Implementation Files

### New Files

- `src/utils/sanitize.rs` - Core sanitization logic
- `src/utils/mod.rs` - Utils module exports

### Updated Files

- `src/lib.rs` - Add utils module export

## Sanitization Rules

### For UInt Target

- Extract all digits from input
- Remove non-numeric characters
- Handle leading zeros based on `keep_zeros` flag
- Return "0" for empty/invalid input

### For Str Target

- Replace invalid characters with separator
- Handle case conversion if `lowercase = true`
- Remove leading zeros from numeric segments (unless `keep_zeros = true`)
- Apply length truncation if `max_length` is set

### Invalid Character Replacement

**Characters replaced with separator:**

- `/` → separator
- `-` → separator (if different from separator)
- `_` → separator (if different from separator)
- Spaces → separator
- Special chars (`@`, `#`, `%`, `!`, etc.) → separator

### Leading Zero Handling

**When `keep_zeros = false` (default):**

- `0051` → `51`
- `001` → `1`
- `0000` → `0`
- `abc0051` → `abc0051` (non-numeric segments unchanged)

**When `keep_zeros = true`:**

- All numeric segments preserved as-is

### Case Conversion

**When `lowercase = true`:**

- All characters converted to lowercase
- Applied before other transformations

### Length Limiting

**When `max_length` is set:**

- Result truncated to maximum length
- Applied after all other transformations

## Testing Strategy

### Unit Tests

- Test each predefined sanitizer (pep440, semver, uint)
- Test edge cases (empty, invalid, special chars)
- Test target-specific behavior (Str vs UInt)
- Test configuration options (separator, lowercase, keep_zeros, max_length)
- Test leading zero removal logic
- Test character replacement rules

### Integration Tests

- Test sanitizer combinations
- Test with real-world input scenarios (branch names, build IDs)
- Test format compatibility (PEP440, SemVer)

### Comprehensive Test Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sanitization() {
        let sanitizer = Sanitizer::default();

        assert_eq!(sanitizer.sanitize("feature/test-branch"), "feature.test.branch");
        assert_eq!(sanitizer.sanitize("Build-ID-0051"), "Build.ID.51");
        assert_eq!(sanitizer.sanitize("test@#$%branch"), "test.branch");
    }

    #[test]
    fn test_pep440_sanitization() {
        let sanitizer = Sanitizer::pep440();

        assert_eq!(sanitizer.sanitize("Feature/API-v2"), "feature.api.v2");
        assert_eq!(sanitizer.sanitize("Build-ID-0051"), "build.id.51");
        assert_eq!(sanitizer.sanitize("TEST_BRANCH"), "test.branch");
    }

    #[test]
    fn test_semver_sanitization() {
        let sanitizer = Sanitizer::semver();

        assert_eq!(sanitizer.sanitize("Feature/API-v2"), "Feature.API.v2");
        assert_eq!(sanitizer.sanitize("build-id-0051"), "build.id.51");
    }

    #[test]
    fn test_uint_extraction() {
        let sanitizer = Sanitizer::uint(false);

        assert_eq!(sanitizer.sanitize("abc123def456"), "123456");
        assert_eq!(sanitizer.sanitize("0051"), "51");
        assert_eq!(sanitizer.sanitize("no-digits"), "0");

        let sanitizer_keep_zeros = Sanitizer::uint(true);
        assert_eq!(sanitizer_keep_zeros.sanitize("0051"), "0051");
    }

    #[test]
    fn test_custom_config() {
        let sanitizer = Sanitizer::str(Some("_"), true, true, Some(10));

        assert_eq!(sanitizer.sanitize("Feature/Test-0051"), "feature_te");
        assert_eq!(sanitizer.sanitize("Build-ID-0051"), "build_id_0");
    }

    #[test]
    fn test_leading_zeros() {
        let sanitizer_remove = Sanitizer { keep_zeros: false, ..Default::default() };
        let sanitizer_keep = Sanitizer { keep_zeros: true, ..Default::default() };

        assert_eq!(sanitizer_remove.sanitize("test-0051"), "test.51");
        assert_eq!(sanitizer_keep.sanitize("test-0051"), "test.0051");
        assert_eq!(sanitizer_remove.sanitize("test-0000"), "test.0");
    }

    #[test]
    fn test_max_length() {
        let sanitizer = Sanitizer { max_length: Some(10), ..Default::default() };

        assert_eq!(sanitizer.sanitize("very-long-branch-name"), "very.long.");
    }
}
```

## Usage Examples

### Direct Usage

```rust
use crate::utils::sanitize::Sanitizer;

// Quick sanitization for common formats
let pep440_sanitizer = Sanitizer::pep440();
let clean_pep440 = pep440_sanitizer.sanitize("feature/API-v2");

let semver_sanitizer = Sanitizer::semver();
let clean_semver = semver_sanitizer.sanitize("feature/API-v2");

// Custom configuration
let custom_sanitizer = Sanitizer::str(Some("_"), true, false, Some(20));
let clean_custom = custom_sanitizer.sanitize("Feature/Long-Branch-Name");

// Integer extraction
let uint_sanitizer = Sanitizer::uint(false);
let extracted_number = uint_sanitizer.sanitize("build-0051-final");
```

### Real-World Examples

```rust
// Branch name sanitization
let branch = "feature/user-auth-v2-0051";
let pep440_branch = Sanitizer::pep440().sanitize(branch);
// Result: "feature.user.auth.v2.51"

let semver_branch = Sanitizer::semver().sanitize(branch);
// Result: "feature.user.auth.v2.51"

// Build ID extraction
let build_string = "build-id-0123-final";
let build_number = Sanitizer::uint(false).sanitize(build_string);
// Result: "123"

// Custom field sanitization
let env_string = "staging/test-env-001";
let clean_env = Sanitizer::str(Some("-"), true, false, None).sanitize(env_string);
// Result: "staging-test-env-1"
```

## Integration Points

### Plan 21 (Handlebars) Integration

```rust
// Handlebars helper will use these sanitizers
fn register_sanitize_helper(handlebars: &mut Handlebars) {
    // Parse helper options into Sanitizer configuration
    // Apply sanitization using Sanitizer::sanitize()
}
```

### Plan 20 (Component Resolution) Integration

```rust
// Component resolution will use format-specific sanitizers
impl Var {
    pub fn resolve_for_pep440(&self, vars: &ZervVars) -> Option<String> {
        let value = self.resolve_value(vars)?;
        let sanitizer = Sanitizer::pep440();
        Some(sanitizer.sanitize(&value.as_string()?))
    }

    pub fn resolve_for_semver(&self, vars: &ZervVars) -> Option<String> {
        let value = self.resolve_value(vars)?;
        let sanitizer = Sanitizer::semver();
        Some(sanitizer.sanitize(&value.as_string()?))
    }
}
```

## Benefits

1. **Reusable**: Pure string functions usable anywhere
2. **Configurable**: Format-specific rules via configuration
3. **Testable**: Easy to test string transformations in isolation
4. **Consistent**: Same sanitization logic across Handlebars and Component resolution
5. **Extensible**: Easy to add new sanitization rules or formats
6. **Performance**: No unnecessary allocations or complex logic
7. **Type Safety**: Clear separation between string and integer sanitization

## Success Criteria

- All sanitizers pass comprehensive tests
- Clean API with consistent naming (`sanitize()` method)
- Reusable across different components
- No external dependencies beyond std
- Format-specific sanitizers (PEP440, SemVer, UInt)
- Configurable transformation rules
- Proper handling of edge cases (empty input, special characters, leading zeros)
