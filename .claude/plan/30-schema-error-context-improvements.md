# Schema Error Context Improvements

## Status: Planned

## Priority: High

## Context

The current `InvalidBumpTarget` error messages in the schema parsing system are too generic and don't provide enough context for users to understand and fix their mistakes. The TODO in `tests/integration_tests/version/combinations/override_bump_interactions.rs:13` highlights this issue - users need to know which schema section (core, extra_core, build), field names, and available options rather than just seeing "Index 0 out of bounds".

## Goals

1. Replace generic index-based error messages with schema section context
2. Include schema section names (core, extra_core, build) in error messages
3. Show available field names in error messages
4. Provide simple suggestions for common mistakes
5. Ensure consistency across all `InvalidBumpTarget` usage patterns

## Implementation Plan

### 1. Create Simple ZervSchemaPart Structure

**File: `src/version/zerv/schema/part.rs`**

```rust
use std::fmt::{Display, Formatter};

use crate::version::zerv::components::Component;

/// Simple representation of a schema part for error context
#[derive(Debug, Clone)]
pub struct ZervSchemaPart<'a> {
    pub name: &'a str,
    pub components: &'a Vec<Component>,
}

impl<'a> ZervSchemaPart<'a> {
    pub fn new(name: &'a str, components: &'a Vec<Component>) -> Self {
        Self { name, components }
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

impl<'a> ZervSchemaPart<'a> {
    pub fn new(name: &'a str, components: &'a Vec<Component>) -> Self {
        Self { name, components }
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn suggest_valid_index_range(&self, invalid_index: isize) -> Option<String> {
        if self.components.is_empty() {
            return Some("The section is empty".to_string());
        }

        let len = self.components.len();
        let max_positive = len - 1;
        let min_negative = -(len as isize);

        // Show the valid range
        let range_suggestion = if len == 1 {
            format!("Valid indices: 0 or -1")
        } else {
            format!("Valid indices: 0 to {} or -1 to {}", max_positive, min_negative)
        };

        if invalid_index >= 0 {
            // Positive index out of bounds
            if invalid_index as usize >= len {
                Some(format!("{}. Did you mean index {}?", range_suggestion, max_positive))
            } else {
                Some(range_suggestion)
            }
        } else {
            // Negative index out of bounds
            if invalid_index < min_negative {
                Some(format!("{}. Did you mean index {}?", range_suggestion, min_negative))
            } else {
                Some(range_suggestion)
            }
        }
    }
}

impl Display for ZervSchemaPart<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.components.is_empty() {
            return write!(f, "{}: No fields available", self.name);
        }

        // Simple implementation, exactly like ZervSchema::Display
        let ron_string = ron::to_string(self.components).map_err(|_| std::fmt::Error)?;
        write!(f, "{}: {}", self.name, ron_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::components::{Component, Var};

    #[test]
    fn test_schema_part_core_section() {
        let components = vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
            Component::Var(Var::Patch),
        ];
        let part = ZervSchemaPart::new("core", &components);

        // Test Display implementation - assert exact expected output
        let display = format!("{}", part);
        assert_eq!(display, "core: [Var(Major), Var(Minor), Var(Patch)]");

        // Test suggestion
        let suggestion = part.suggest_valid_index_range(5);
        assert_eq!(suggestion.unwrap(), "Valid indices: 0 to 2 or -1 to -3. Did you mean index 2?");
    }

    #[test]
    fn test_schema_part_negative_index_suggestion() {
        let components = vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
            Component::Var(Var::Patch),
        ];
        let part = ZervSchemaPart::new("core", &components);

        let suggestion = part.suggest_valid_index_range(-5);
        assert_eq!(suggestion.unwrap(), "Valid indices: 0 to 2 or -1 to -3. Did you mean index -3?");
    }

    #[test]
    fn test_schema_part_empty_section() {
        let components = vec![];
        let part = ZervSchemaPart::new("build", &components);

        let display = format!("{}", part);
        assert_eq!(display, "build: No fields available");

        let suggestion = part.suggest_valid_index_range(0);
        assert_eq!(suggestion, Some("The section is empty".to_string()));
    }

    #[test]
    fn test_schema_part_mixed_components() {
        let components = vec![
            Component::Var(Var::Major),
            Component::Str("test".to_string()),
            Component::UInt(42),
        ];
        let part = ZervSchemaPart::new("mixed", &components);

        let display = format!("{}", part);
        assert_eq!(display, "mixed: [Var(Major), \"test\", 42]");
    }

    #[test]
    fn test_schema_part_len_and_empty() {
        let part = ZervSchemaPart::new("test", &vec![]);
        assert_eq!(part.len(), 0);
        assert!(part.is_empty());

        let part = ZervSchemaPart::new("test", &vec![Component::Var(Var::Major)]);
        assert_eq!(part.len(), 1);
        assert!(!part.is_empty());
    }

    #[test]
    fn test_schema_part_single_element_suggestion() {
        let components = vec![Component::Var(Var::Major)];
        let part = ZervSchemaPart::new("single", &components);

        let suggestion = part.suggest_valid_index_range(5);
        assert_eq!(suggestion.unwrap(), "Valid indices: 0 or -1. Did you mean index 0?");
    }

    #[test]
    fn test_schema_part_valid_indices_no_suggestion() {
        let components = vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
        ];
        let part = ZervSchemaPart::new("test", &components);

        // Valid index should return range suggestion but no specific index suggestion
        let suggestion = part.suggest_valid_index_range(1);
        assert_eq!(suggestion.unwrap(), "Valid indices: 0 to 1 or -1 to -2");
    }
}
```

### 2. Enhance ZervError with Simple Context

**File: `src/error.rs`**

```rust
use crate::version::zerv::schema::part::ZervSchemaPart;

#[derive(Debug)]
pub enum ZervError {
    // Existing variants...

    /// Invalid bump target with schema context
    InvalidBumpTarget {
        message: String,
        schema_part: ZervSchemaPart<'static>,
        suggestion: Option<String>,
    },
}

impl std::fmt::Display for ZervError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Existing cases...

            ZervError::InvalidBumpTarget {
                message,
                schema_part,
                suggestion
            } => {
                write!(f, "{}", message)?;

                // Add schema section information with RON formatting (uses Display trait)
                write!(f, "\nSchema section: {}", schema_part)?;

                // Add suggestions if available
                if let Some(suggestion) = suggestion {
                    write!(f, "\n{}", suggestion)?;
                }

                Ok(())
            }

            // Remove existing simple InvalidBumpTarget - replace with new one
        }
    }
}
```

### 3. Update parse_index Function

**File: `src/version/zerv/bump/schema_parsing.rs`**

```rust
use crate::error::ZervError;
use crate::version::zerv::schema::part::ZervSchemaPart;

fn parse_index(
    idx_str: &str,
    schema_part: ZervSchemaPart<'static>,
) -> Result<usize, ZervError> {
    let idx = idx_str.parse::<isize>().map_err(|_| {
        ZervError::InvalidBumpTarget {
            message: format!("Invalid index: '{}' is not a valid number", idx_str),
            schema_part: schema_part.clone(),
            index_suggestion: None,
        }
    })?;

    let schema_len = schema_part.len();

    if idx >= 0 {
        // Positive index: 0, 1, 2, ...
        let idx_usize = idx as usize;
        if idx_usize >= schema_len {
            return Err(ZervError::InvalidBumpTarget {
                message: format!(
                    "Index {} is out of bounds for {} (length: {})",
                    idx, schema_part, schema_len
                ),
                schema_part,
                suggestion: schema_part.suggest_valid_index_range(idx),
            });
        }
        Ok(idx_usize)
    } else {
        // Negative index: -1, -2, -3, ... (count from end)
        let calculated_idx = schema_len as isize + idx;
        if calculated_idx < 0 || calculated_idx >= schema_len as isize {
            return Err(ZervError::InvalidBumpTarget {
                message: format!(
                    "Negative index {} is out of bounds for {} (length: {})",
                    idx, schema_part, schema_len
                ),
                schema_part,
                suggestion: schema_part.suggest_valid_index_range(idx),
            });
        }
        Ok(calculated_idx as usize)
    }
}
```

### 4. Update Schema Processing Functions

**File: `src/version/zerv/bump/schema_processing.rs`**

```rust
use crate::version::zerv::schema::part::ZervSchemaPart;
use crate::error::ZervError;

pub fn process_schema_section(
    section_name: &str,
    schema: &ZervSchema,
) -> Result<ProcessedSection, ZervError> {
    let schema_part = match section_name {
        "core" => ZervSchemaPart::new("core", schema.core()),
        "extra_core" => ZervSchemaPart::new("extra_core", schema.extra_core()),
        "build" => ZervSchemaPart::new("build", schema.build()),
        unknown => {
            // Suggest correct section name
            let available_sections = vec!["core", "extra_core", "build"];
            let suggestion = available_sections
                .into_iter()
                .min_by_key(|section| simple_distance(unknown, section))
                .map(|suggestion| format!("Did you mean '{}'?", suggestion));

            return Err(ZervError::InvalidBumpTarget {
                message: format!("Unknown schema section: '{}'", unknown),
                schema_part: ZervSchemaPart::new("unknown", &vec![]), // Empty section for unknown
                suggestion,
            });
        }
    };

    // Continue processing with schema_part...
}

// Simple string distance for suggestions
fn simple_distance(a: &str, b: &str) -> usize {
    if a == b { return 0; }

    // Count character differences
    a.chars().zip(b.chars())
        .map(|(a, b)| if a == b { 0 } else { 1 })
        .sum()
}
```

### 5. Update Test Cases

**File: `tests/integration_tests/version/combinations/override_bump_interactions.rs`**

```rust
#[rstest]
fn test_build_override_fails_for_empty_build_section(base_fixture: ZervFixture) {
    let zerv_ron = base_fixture.build().to_string();

    let result = TestCommand::run_with_stdin_expect_fail(
        "version --source stdin --output-format zerv --build 0=test",
        zerv_ron.clone(),
    );

    println!("{result}");

    // Updated assertions for rich error messages
    assert!(result.contains("out of bounds"));
    assert!(result.contains("Schema section: build: No fields available"));
}

#[rstest]
fn test_core_section_invalid_index_with_suggestion(base_fixture: ZervFixture) {
    let zerv_ron = base_fixture.build().to_string();

    let result = TestCommand::run_with_stdin_expect_fail(
        "version --source stdin --output-format zerv --core 5=patch",
        zerv_ron.clone(),
    );

    println!("{result}");

    // Should show rich context with RON formatting and valid range
    assert!(result.contains("Index 5 is out of bounds"));
    assert!(result.contains("Schema section: core: ["));
    assert!(result.contains("Var(Major)"));
    assert!(result.contains("Var(Minor)"));
    assert!(result.contains("Var(Patch)"));
    assert!(result.contains("Valid indices: 0 to 2 or -1 to -3"));
    assert!(result.contains("Did you mean index 2?"));
}

#[rstest]
fn test_core_section_negative_index_with_suggestion(base_fixture: ZervFixture) {
    let zerv_ron = base_fixture.build().to_string();

    let result = TestCommand::run_with_stdin_expect_fail(
        "version --source stdin --output-format zerv --core -5=patch",
        zerv_ron.clone(),
    );

    println!("{result}");

    // Should show rich context with negative index info
    assert!(result.contains("Negative index -5 is out of bounds"));
    assert!(result.contains("Schema section: core: ["));
    assert!(result.contains("Valid indices: 0 to 2 or -1 to -3"));
    assert!(result.contains("Did you mean index -3?"));
}

#[rstest]
fn test_unknown_schema_section_with_suggestion(base_fixture: ZervFixture) {
    let zerv_ron = base_fixture.build().to_string();

    let result = TestCommand::run_with_stdin_expect_fail(
        "version --source stdin --output-format zerv --coer 0=major", // typo for "core"
        zerv_ron.clone(),
    );

    println!("{result}");

    assert!(result.contains("Unknown schema section: 'coer'"));
    assert!(result.contains("Did you mean 'core'?"));
}
```

## Testing Strategy

### 1. Unit Tests for ZervSchemaPart

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::components::{Component, Var};

    #[test]
    fn test_schema_part_core_section() {
        let components = vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
            Component::Var(Var::Patch),
        ];
        let part = ZervSchemaPart::new("core", &components);

        let available = format!("{}", part);
        // Should use RON formatting
        assert_eq!(available, "core: [Var(Major), Var(Minor), Var(Patch)]");

        let suggestion = part.suggest_valid_index_range(5);
        assert_eq!(suggestion.unwrap(), "Valid indices: 0 to 2 or -1 to -3. Did you mean index 2?");
    }

    #[test]
    fn test_schema_part_negative_index_suggestion() {
        let components = vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
            Component::Var(Var::Patch),
        ];
        let part = ZervSchemaPart::new("core", &components);

        let suggestion = part.suggest_valid_index_range(-5);
        assert_eq!(suggestion.unwrap(), "Valid indices: 0 to 2 or -1 to -3. Did you mean index -3?");
    }

    #[test]
    fn test_schema_part_empty_section() {
        let components = vec![];
        let part = ZervSchemaPart::new("build", &components);

        let available = format!("{}", part);
        assert_eq!(available, "build: No fields available");

        let suggestion = part.suggest_valid_index_range(0);
        assert_eq!(suggestion, Some("The section is empty".to_string()));
    }

    #[test]
    fn test_schema_part_ron_formatting() {
        let components = vec![
            Component::Var(Var::Major),
            Component::Str("test".to_string()),
            Component::UInt(42),
        ];
        let part = ZervSchemaPart::new("mixed", &components);

        let available = format!("{}", part);
        assert_eq!(available, "mixed: [Var(Major), \"test\", 42]");
    }
}
```

### 2. Integration Test Coverage

Update all existing tests that expect `InvalidBumpTarget` errors to verify context is included:

- Index out of bounds errors for each section type
- Unknown schema section errors
- Invalid index format errors

### 3. Error Message Regression Tests

```rust
#[rstest]
#[case("core", 5, "core section")]
#[case("build", 0, "build section")]
fn test_error_message_contains_section_name(
    #[case] section: &str,
    #[case] index: usize,
    #[case] expected_content: &str
) {
    let result = run_invalid_bump_command(section, index);
    assert!(result.contains(expected_content));
}
```

## Migration Strategy

### Phase 1: Add New Error Infrastructure

1. Add `ZervSchemaPart` module
2. Update `InvalidBumpTarget` variant to include context fields
3. Update error display formatting

### Phase 2: Update Error Sites

1. Update `parse_index` to use `ZervSchemaPart`
2. Update schema processing functions to create `ZervSchemaPart` instances
3. Update any other `InvalidBumpTarget` usage sites

### Phase 3: Update Tests

1. Update all existing tests to expect rich error messages
2. Add new test coverage for error context functionality

## Success Criteria

1. ✅ All `InvalidBumpTarget` errors include schema section names
2. ✅ Error messages show available field names
3. ✅ Simple suggestions are provided for common mistakes
4. ✅ All existing tests pass with updated error message expectations
5. ✅ New test coverage for error context functionality

## Documentation Updates

1. Update error handling documentation in `.claude/ref/standards/error-handling.md`
2. Add examples of new error messages to user documentation

## Impact Assessment

### Benefits

- **Better Developer Experience**: Users get clear, actionable error messages
- **Reduced Support Burden**: More self-service debugging capability
- **Simple Implementation**: Minimal complexity with `ZervSchemaPart`
- **Learning Opportunity**: Error messages teach users about schema structure

### Risks

- **Breaking Change**: Error message formats will change
- **Implementation Effort**: Need to update error sites throughout codebase

### Mitigation

- Comprehensive test coverage for new error messages
- Simple, focused implementation reduces risk
- Clear migration strategy with phased approach
