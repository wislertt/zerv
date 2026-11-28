# Integration Test Patterns

This document outlines the standard patterns for integration tests in Zerv.

## Default Pattern: `TestCommand::run_with_stdin()` (90% of tests)

Use this for simple stdin → stdout success tests:

```rust
let zerv_ron = ZervFixture::new()
    .with_version(1, 2, 3)
    .build()
    .to_string();

let output = TestCommand::run_with_stdin(
    "version --source stdin --output-format semver",
    zerv_ron,
);

assert_eq!(output, "1.2.3");
```

## Builder Pattern (10% of tests)

Use ONLY when you need:

- Stderr inspection
- Failure testing
- Multiple output assertions

```rust
// Failure testing
let output = TestCommand::new()
    .args_from_str("version --invalid-flag")
    .assert_failure();
assert!(output.stderr().contains("error"));

// Stderr inspection
let output = TestCommand::new()
    .args_from_str("version --source stdin -v")
    .stdin(zerv_ron)
    .assert_success();
assert!(output.stderr().contains("debug"));
assert_eq!(output.stdout().trim(), "1.2.3");
```

## rstest Usage

**STANDARD: rstest fixtures and parameterization are covered in detail in** [testing/overview.md](./overview.md#rstest-testing-framework).

**Key reminders for integration tests:**

- Use rstest fixtures for common test setup
- Use `#[case]` for testing multiple input/output variations
- Organize related tests in modules with `//!` documentation
- Follow the patterns documented in overview.md

## Module Documentation

Document test modules with `//!` inner doc comments explaining what they test:

```rust
mod schema_preset_standard {
    //! Tests for the built-in zerv-standard schema preset.
    //!
    //! Covers all three tiers:
    //! - Tier 1: Tagged, clean (major.minor.patch)
    //! - Tier 2: Distance, clean (major.minor.patch+distance)
    //! - Tier 3: Dirty (major.minor.patch-dev.timestamp+metadata)

    use super::*;

    #[rstest]
    fn test_tier_1() {
        // Test implementation
    }
}
```

## Test Constants

Define constants at module level for better readability and maintainability:

```rust
mod vcs_overrides {
    use super::*;

    const DEV_TIMESTAMP: u64 = 1234567890;
    const TEST_BRANCH: &str = "feature.branch";
    const TEST_COMMIT_HASH: &str = "abc123def456";

    #[rstest]
    fn test_with_branch() {
        let fixture = ZervFixture::new()
            .with_vcs_data(None, Some(TEST_BRANCH), None, None);
        // Test implementation
    }
}
```

## Common Mistakes to Avoid

**Don't create helper functions - use rstest fixtures**:

```rust
// ❌ BAD: Helper function requiring manual calls
fn create_fixture(version: (u64, u64, u64)) -> ZervFixture {
    ZervFixture::new().with_version(version.0, version.1, version.2)
}

// ✅ GOOD: rstest fixture with automatic injection
#[fixture]
fn base_fixture() -> ZervFixture {
    ZervFixture::new().with_version(1, 0, 0)
}
```

**Don't use comment dividers - use mod blocks**:

```rust
// ❌ BAD: Comment dividers
// ============================================================================
// Output Format Tests
// ============================================================================

// ✅ GOOD: Module organization
mod output_format {
    //! Tests for output format conversions
    use super::*;
}
```

**Don't use builder for simple tests - use run_with_stdin()**:

```rust
// ❌ BAD: Verbose builder for simple stdout test
let output = TestCommand::new()
    .args_from_str("version --source stdin")
    .stdin(zerv_ron)
    .assert_success();
assert_eq!(output.stdout().trim(), "1.2.3");

// ✅ GOOD: Concise helper
let output = TestCommand::run_with_stdin("version --source stdin", zerv_ron);
assert_eq!(output, "1.2.3");
```

## Complete Example

Combining all patterns together:

```rust
use rstest::{fixture, rstest};
use crate::test_utils::ZervFixture;
use crate::util::TestCommand;

#[fixture]
fn clean_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 0, 0)
        .with_vcs_data(Some(0), Some(false), None, None)
}

mod basic_output {
    //! Tests for basic version output in different formats
    use super::*;

    #[rstest]
    #[case::semver("semver", "1.0.0")]
    #[case::pep440("pep440", "1.0.0")]
    fn test_format(
        clean_fixture: ZervFixture,
        #[case] format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = clean_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --output-format {format}"),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }
}

mod error_handling {
    //! Tests for error conditions and validation
    use super::*;

    #[rstest]
    fn test_invalid_format(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --output-format invalid")
            .stdin(zerv_ron)
            .assert_failure();

        assert!(output.stderr().contains("invalid"));
    }
}
```

## Quick Reference

```rust
// Default: Simple stdin → stdout test (90% of cases)
let output = TestCommand::run_with_stdin("version --source stdin", zerv_ron);
assert_eq!(output, "1.2.3");

// Failure testing
let output = TestCommand::new()
    .args_from_str("version --invalid")
    .assert_failure();
assert!(output.stderr().contains("error"));

// Stderr inspection
let output = TestCommand::new()
    .args_from_str("version -v")
    .assert_success();
assert!(output.stderr().contains("debug"));
```
