# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

Zerv is a dynamic versioning CLI tool written in Rust that generates versions for any commit from git and other version control systems. It supports multiple version formats (SemVer, PEP440, CalVer) and is designed for CI/CD builds.

---

## 🚨 CRITICAL: Code Comment Policy

**IMPORTANT**: This is an open-source Rust project. Keep code clean and readable. Only add comments when they provide value that code cannot.

### The Golden Rule

**If a comment just repeats what the code or function name already says, DELETE IT.**

### When to Comment (Rarely)

1. **Complex algorithms** - Explain the approach or why it works
2. **Non-obvious behavior** - Domain-specific rules (e.g., "PEP440 ordering: dev < alpha < beta < rc")
3. **Surprising edge cases** - Behavior that might confuse future readers
4. **Public API with non-obvious usage** - Only if the signature doesn't explain itself

### When NOT to Comment (Almost Always)

1. **Never restate function names** - `vcs_data_to_zerv_vars` needs no comment explaining it converts VCS data
2. **Never restate what code does** - `is_dirty()` doesn't need "Returns dirty status"
3. **Never add obvious comments** - No "Initialize repo", "Create variable", "Return value"
4. **Never document arguments that are self-explanatory** - `user_id: &str` doesn't need explanation

### Self-Documenting Code First

1. **Use clear function names** - `format_pep440_prerelease` vs `format_pre`
2. **Use clear variable names** - `commit_distance` vs `dist`
3. **Extract complex logic** - Create well-named helper functions
4. **Let types document contracts** - `Result<Version, ParseError>` is self-documenting

### Examples

````rust
// ✅ PERFECT - No comment needed, function name is clear
pub fn vcs_data_to_zerv_vars(vcs_data: &VcsData, version_obj: &VersionObject) -> HashMap<String, String> {
    let prerelease = match &version_obj.pre {
        Some(pre) => format_pep440_prerelease(pre),
        None => String::new(),
    };
}

// ✅ GOOD - Comment adds value (explains non-obvious domain rule)
/// PEP440 pre-release ordering: dev < alpha < beta < rc < final
pub fn format_pep440_prerelease(pre: &PreRelease) -> String {
    match pre.label.as_str() {
        "dev" => format!("dev{}", pre.number),
        "a" => format!("a{}", pre.number),
        "b" => format!("b{}", pre.number),
        "rc" => format!("rc{}", pre.number),
        _ => String::new(),
    }
}

// ✅ GOOD - Example clarifies non-obvious behavior
/// Strips common version prefixes like "v", "release-", "ver"
///
/// # Example
/// ```
/// parse_version_tag("v1.2.3") // Returns "1.2.3"
/// parse_version_tag("release-2.0.0") // Returns "2.0.0"
/// ```
pub fn parse_version_tag(tag: &str) -> String {
    // Implementation
}

// ❌ BAD - Just repeats function name
/// Converts VCS metadata to Zerv template variables.
pub fn vcs_data_to_zerv_vars(vcs_data: &VcsData, version_obj: &VersionObject) -> HashMap<String, String> {}

// ❌ BAD - Obvious from function name
/// Checks if the repository is dirty.
pub fn is_dirty(&self) -> bool {
    self.dirty
}

// ❌ BAD - Restates what code does
pub fn calculate_distance(&self) -> usize {
    // Get the commit count
    let count = self.commits.len();
    // Return the count
    count
}

// ❌ BAD - Useless inline comments
let prerelease = match &version_obj.pre {
    Some(pre) => format_prerelease(pre), // Format the prerelease
    None => String::new(), // Return empty string
};
````

**Bottom Line**: If you're about to write a comment, first try to make the code clearer. If the code can't be clearer, then add the comment.

### 🚨 ZERO TOLERANCE: Comment Violations

**ABSOLUTELY FORBIDDEN** - These patterns will be immediately rejected:

1. **Function name restatements**:

    ```rust
    // ❌ FORBIDDEN
    /// Converts VCS data to Zerv variables
    pub fn vcs_data_to_zerv_vars() { }

    // ✅ INSTEAD: Make function name clear or remove comment entirely
    pub fn vcs_data_to_zerv_vars() { }  // Name is self-explanatory
    ```

2. **Obvious parameter documentation**:

    ```rust
    // ❌ FORBIDDEN
    /// Processes the version
    /// @param version: The version to process
    fn process_version(version: Version) { }
    ```

3. **Section divider comments** - Use `mod` blocks instead:

    ```rust
    // ❌ FORBIDDEN
    // ============================================================================
    // Builder Pattern Methods
    // ============================================================================

    // ✅ INSTEAD: Use module organization or no comments at all
    ```

4. **Process step comments**:
    ```rust
    // ❌ FORBIDDEN
    let result = calculate(); // Calculate the result
    return result; // Return the result
    ```

**If you see these patterns during code review, REMOVE THEM IMMEDIATELY.**

---

## 🔒 Import Statement Policy

**MANDATORY: Always place `use` statements at the top of the file or module, never inside functions.**

### The Rule

All `use` statements must be declared at:

1. **Top of file** (preferred for most cases)
2. **Top of test module** (for test-specific imports in `#[cfg(test)] mod tests { ... }`)

**NEVER** place `use` statements inside individual functions, even in tests.

### Why This Matters

1. **Readability** - All dependencies visible at a glance
2. **Maintainability** - Easy to see what the file/module depends on
3. **Rust Conventions** - Standard Rust style guidelines
4. **IDE Support** - Better auto-imports and refactoring
5. **Code Review** - Easier to spot unused or unnecessary imports

### Examples

```rust
// ✅ GOOD - Imports at top of file
use rstest::rstest;
use zerv::test_utils::{ZervFixture, ZervSchemaFixture};
use zerv::version::zerv::{Component, Var, ZervSchema};

#[test]
fn test_something() {
    let schema = ZervSchema::new_with_precedence(
        vec![Component::Var(Var::Major)],
        vec![],
        vec![],
        PrecedenceOrder::default(),
    ).unwrap();
}

// ✅ GOOD - Test-specific imports at top of test module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestHelper;

    #[test]
    fn test_helper_usage() {
        TestHelper::new();
    }
}

// ❌ BAD - Imports inside function
#[test]
fn test_something() {
    use zerv::version::zerv::{Component, Var, ZervSchema};  // ❌ WRONG!
    use zerv::version::zerv::bump::precedence::PrecedenceOrder;  // ❌ WRONG!

    let schema = ZervSchema::new_with_precedence(
        vec![Component::Var(Var::Major)],
        vec![],
        vec![],
        PrecedenceOrder::default(),
    ).unwrap();
}

// ❌ BAD - Imports scattered throughout file
fn function_a() {
    use some_crate::TypeA;  // ❌ WRONG!
    // ...
}

fn function_b() {
    use some_crate::TypeB;  // ❌ WRONG!
    // ...
}
```

### Exception (Very Rare)

The only acceptable use of inline imports is when deliberately limiting scope to prevent naming conflicts:

```rust
// ✅ ACCEPTABLE - Intentional scope limitation
fn convert_format() {
    use external_crate::Format as ExternalFormat;  // Avoids naming conflict with local Format
    // Use ExternalFormat here only
}
```

**If you see inline `use` statements during code review or refactoring, move them to the top of the file immediately.**

---

## 📦 Test Organization Policy

**MANDATORY: Use Rust modules for test organization, not comment-based grouping.**

### The Rule

When organizing tests into logical groups:

1. **Use `mod` blocks** for structural grouping
2. **NEVER use comment dividers** like `// ============ Section Name ============`
3. Each module should use `use super::*;` to import parent scope

**Benefits over comment-based grouping:**

- Rust enforces module boundaries
- IDEs can collapse/expand modules
- Test names include module path for clarity
- Can add module-level helpers
- Consistent with codebase patterns

### Test Organization Pattern

```rust
// ✅ GOOD - Module-based organization
use rstest::rstest;
use crate::util::TestCommand;

mod feature_basic {
    use super::*;

    #[test]
    fn test_something() { /* ... */ }

    #[rstest]
    #[case::scenario_a("input", "expected")]
    fn test_parameterized(#[case] input: &str, #[case] expected: &str) { /* ... */ }
}

mod feature_advanced {
    use super::*;

    #[test]
    fn test_complex_behavior() { /* ... */ }

    // Module-level helper function
    fn setup_complex_fixture() -> Fixture {
        // ...
    }
}

mod feature_error_handling {
    use super::*;

    #[test]
    fn test_error_case() { /* ... */ }
}

// ❌ BAD - Comment-based grouping
// ============================================================================
// Feature Basic Tests
// ============================================================================

#[test]
fn test_something() { /* ... */ }

// ============================================================================
// Feature Advanced Tests
// ============================================================================

#[test]
fn test_complex_behavior() { /* ... */ }
```

### Real Example from Codebase

**Before (comment-based):**

```rust
// ============================================================================
// Schema Preset Tests - zerv-standard
// ============================================================================

#[test]
fn test_schema_standard_tier_1() { /* ... */ }

// ============================================================================
// Schema Validation and Error Handling
// ============================================================================

#[test]
fn test_schema_unknown_preset_error() { /* ... */ }
```

**After (module-based):**

```rust
mod schema_preset_standard {
    use super::*;

    #[test]
    fn test_schema_standard_tier_1() { /* ... */ }
}

mod schema_validation {
    use super::*;

    #[test]
    fn test_schema_unknown_preset_error() { /* ... */ }
}
```

**Test output includes module path:**

```
test integration_tests::version::main::schemas::schema_preset_standard::test_schema_standard_tier_1 ... ok
test integration_tests::version::main::schemas::schema_validation::test_schema_unknown_preset_error ... ok
```

### When to Use Modules

Use module-based organization when:

- File has 3+ logical test groups
- Tests naturally cluster by feature/behavior
- Need to share setup code within a group
- File exceeds ~200 lines

### Module Naming Conventions

- Use snake_case: `mod schema_preset_standard`
- Be descriptive: `mod schema_validation` not `mod validation`
- Match test function prefixes when possible
- Keep names short but clear

**If you see comment-based test grouping during refactoring, convert it to module-based organization.**

---

## Essential Commands

### Development Setup

```bash
make setup_dev  # Install pre-commit hooks and cargo-tarpaulin
```

### Testing

```bash
make test_easy  # Quick tests: Docker Git + Docker tests skipped (fast, coverage gaps)
make test       # Full test suite: Docker Git + Docker tests enabled (full coverage)
make test_flaky # Run 5 iterations to detect flaky tests
```

### Building and Running

```bash
make run        # Run the CLI with cargo run
cargo build     # Build debug version
cargo build --release  # Build optimized release version
```

### Code Quality

```bash
make lint       # Check code formatting and clippy warnings
make update     # Update Rust toolchain and dependencies
```

### Coverage

```bash
make test              # Generates coverage reports
make open_coverage     # Open HTML coverage report
```

### Documentation

```bash
make docs       # Generate documentation via cargo xtask
```

---

## 📚 MANDATORY: Always Check .dev First

**BEFORE performing ANY coding task, read `.dev/00-README.md` for complete project context and workflow.**

### .dev Document Numbering

All `.dev/` documents use sequential numbering to indicate creation order:

- `00-***.md`: Current state (baseline)
- `01-***.md`: Next development phase
- `02-***.md`: Following phase

**Higher numbers = more recent/updated plans**. Always verify against actual codebase - higher numbered docs are more likely to be current.

---

## High-Level Architecture

### Pipeline Architecture

The core processing follows a clear pipeline pattern:

```
Input → VCS Detection → Version Parsing → Transformation → Format Output
```

**Key Modules:**

- **`src/vcs/`**: Version Control System abstraction (currently Git only)
    - Detects VCS repositories and extracts metadata
    - `VcsData` struct contains tag versions, distance, commits, branches, timestamps

- **`src/version/`**: Version format implementations
    - `VersionObject`: Universal internal representation
    - `PEP440`: Python versioning standard
    - `SemVer`: Semantic versioning
    - `Zerv`: Custom component-based format with variable references

- **`src/pipeline/`**: Data transformation layer
    - `parse_version_from_tag()`: Extracts version from git tags
    - `vcs_data_to_zerv_vars()`: Converts VCS metadata to Zerv variables

- **`src/schema/`**: Schema and preset management
    - Presets for common versioning schemes (standard, calver)
    - RON-based schema parsing for custom formats

- **`src/cli/`**: Command-line interface
    - Main commands: `version`, `check`
    - Output formatting and display logic

### State-Based Versioning Tiers

Zerv uses a three-tier system based on repository state:

- **Tier 1** (Tagged, clean): `major.minor.patch`
- **Tier 2** (Distance, clean): `major.minor.patch.post<distance>+branch.<commit>`
- **Tier 3** (Dirty): `major.minor.patch.dev<timestamp>+branch.<commit>`

### Test Infrastructure

The project has extensive test utilities in `src/test_utils/`:

- **Environment-aware Git testing**: Uses `DockerGit` locally, `NativeGit` in CI
- **`GitOperations` trait**: Unified interface for both implementations
- **`GitRepoFixture`**: Creates isolated test repositories with specific states
- **`TestDir`**: Temporary directory management with automatic cleanup

---

## 🔒 Error Handling Standards

**MANDATORY: Follow these error handling rules strictly.**

### Rules

1. **ALWAYS use `zerv::error::ZervError` for custom errors**
    - Never create ad-hoc error types
    - Use proper error propagation with `?` operator

2. **Use `io::Error::other()` instead of `io::Error::new(io::ErrorKind::Other, ...)`**
    - Shorter and more idiomatic

3. **Include context in error messages**
    - Say what failed, where, and why
    - Include relevant file paths, values, or states

4. **NEVER use `unwrap()` or `expect()` in production code**
    - Only acceptable in test code
    - In tests, use `expect()` with detailed context

### Examples

```rust
// ✅ GOOD
let file = fs::read_to_string(&path)
    .map_err(|e| ZervError::Io(io::Error::other(
        format!("Failed to read config file at {}: {}", path.display(), e)
    )))?;

// ✅ GOOD - Test code with context
let fixture = GitRepoFixture::tagged("v1.0.0")
    .expect("Failed to create tagged repo - check Docker availability");

// ❌ BAD - Generic error message
let file = fs::read_to_string(&path)?;

// ❌ BAD - Old error pattern
return Err(ZervError::Io(io::Error::new(
    io::ErrorKind::Other,
    "Something failed"
)));

// ❌ BAD - Production unwrap
let config = load_config().unwrap();
```

---

## 🎯 Constants Usage - MANDATORY

**NEVER use bare string literals for field names, formats, sources, or schema names.**

**Why**: Type safety, refactoring safety, consistency, IDE support, maintainability.

### Available Constants

```rust
use crate::utils::constants::{fields, formats, sources, schema_names};

// Field names
fields::MAJOR
fields::MINOR
fields::PATCH
fields::EPOCH
fields::PRE_RELEASE
fields::POST
fields::DEV
fields::DISTANCE
fields::DIRTY
fields::BUMPED_BRANCH
fields::BUMPED_COMMIT_HASH
fields::LAST_COMMIT_HASH
fields::LAST_TIMESTAMP
fields::LAST_BRANCH

// Format names
formats::SEMVER
formats::PEP440
formats::ZERV
formats::AUTO

// Source names
sources::GIT
sources::STDIN

// Schema names
schema_names::ZERV_STANDARD
schema_names::ZERV_CALVER
```

### Examples

```rust
// ✅ GOOD - Using constants
use crate::utils::constants::{fields, formats};

match field_name.as_str() {
    fields::MAJOR => handle_major(),
    fields::MINOR => handle_minor(),
    fields::PATCH => handle_patch(),
    _ => return Err(ZervError::UnknownField(field_name.to_string())),
}

match format.as_str() {
    formats::SEMVER => convert_to_semver()?,
    formats::PEP440 => convert_to_pep440()?,
    _ => return Err(ZervError::UnknownFormat(format.to_string())),
}

// ❌ BAD - Bare strings (will be flagged during code review)
match field_name.as_str() {
    "major" => handle_major(),
    "minor" => handle_minor(),
    "patch" => handle_patch(),
    _ => return Err(ZervError::UnknownField(field_name.to_string())),
}
```

---

## 🧪 Testing Standards

### Environment Variables

- `ZERV_TEST_NATIVE_GIT=true`: Use native Git (set in CI for platform testing)
- `ZERV_TEST_DOCKER=true`: Enable Docker-dependent tests (requires Docker)

### Environment-Aware Git Testing Pattern

**MANDATORY: Always use `get_git_impl()` for environment-aware Git operations.**

```rust
use crate::test_utils::{GitOperations, get_git_impl};

#[test]
fn test_git_operations() {
    // ✅ GOOD - Environment-aware
    let git_impl = get_git_impl();
    git_impl.init_repo(&test_dir)?;

    // ❌ BAD - Direct implementation usage
    let git = DockerGit::new();
    git.init_repo(&test_dir)?;
}
```

### Docker Test Gating

**MANDATORY: Use `should_run_docker_tests()` for all Docker-dependent tests.**

```rust
use crate::test_utils::should_run_docker_tests;

#[test]
fn test_docker_functionality() {
    if !should_run_docker_tests() {
        return;
    }
    // Docker-dependent test code
}
```

### Test Structure Template

**IMPORTANT: Follow this structure for all tests to prevent flaky behavior.**

```rust
#[test]
fn test_example() {
    // 1. Docker test gating (if needed)
    if !should_run_docker_tests() {
        return;
    }

    // 2. Setup - Create isolated resources
    let fixture = GitRepoFixture::tagged("v1.0.0")
        .expect("Failed to create tagged repo - check Docker availability and Git operations");

    // 3. Verify Setup - Check preconditions
    assert!(fixture.path().join(".git").exists(),
        "Git repository should exist at: {}", fixture.path().display());

    // 4. Execute - Run the test operation
    let result = some_operation(&fixture);

    // 5. Verify Results - Check postconditions with detailed assertions
    let output = result.unwrap_or_else(|e| {
        panic!("Operation should succeed for tagged repo at {}: {}",
               fixture.path().display(), e);
    });

    assert!(output.contains("expected"),
        "Output should contain expected content. Got: {output}");

    // 6. Cleanup - Automatic via Drop trait
}
```

### Flaky Test Prevention - CRITICAL RULES

**These patterns MUST be followed to prevent race conditions and flaky tests.**

#### ✅ DO: Atomic Operations

```rust
// ✅ GOOD - Fresh Git implementation per directory
let test_dir = TestDir::new()?;
let git_impl = get_git_impl();
git_impl.init_repo(&test_dir)?;

// ✅ GOOD - State verification
assert!(test_dir.path().join(".git").exists(),
    "Git repository should exist at: {}", test_dir.path().display());

// ✅ GOOD - Detailed error context
let fixture = GitRepoFixture::tagged("v1.0.0")
    .expect("Failed to create tagged repo - check Docker availability");

// ✅ GOOD - Isolated resources per test
#[test]
fn test_something() {
    let fixture = GitRepoFixture::tagged("v1.0.0")?;
}

// ✅ GOOD - Detailed assertions
assert!(output.contains("schema"),
    "Output should contain 'schema' field. Got: {output}");
```

#### ❌ DON'T: Anti-Patterns

```rust
// ❌ BAD - Reusing Git implementations across directories (race conditions!)
let fixture = GitRepoFixture::tagged("v1.0.0")?;
fixture.git_impl.init_repo(&different_dir)?;

// ❌ BAD - Multi-step operations without verification
let fixture = GitRepoFixture::with_distance("v1.0.0", 1)?;
fixture.git_impl.execute_git(&fixture.test_dir, &["tag", "-d", "v1.0.0"])?;

// ❌ BAD - Generic error messages
let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed");

// ❌ BAD - Shared state between tests
static SHARED_FIXTURE: OnceCell<GitRepoFixture> = OnceCell::new();

// ❌ BAD - Assertions without context
assert!(output.contains("schema"));
```

### Make Commands

- `make test_easy`: Docker Git + Docker tests skipped (fast, coverage gaps)
- `make test`: Docker Git + Docker tests enabled (full coverage)
- CI: Native Git + Docker tests on Linux only

---

## 🔄 Code Reuse Standards

**MANDATORY: Always check existing utilities before creating new ones.**

### Rules

1. **Check `src/test_utils/` first** before creating test utilities
2. **Reuse existing infrastructure**: `TestDir`, `GitOperations`, `GitRepoFixture`
3. **Use `get_git_impl()`** for environment-aware Git operations
4. **Prefer `GitOperations` trait methods** over direct Docker/Native calls
5. **Avoid duplicating code** across different files
6. **Look for existing helpers** before implementing new ones

### Examples

```rust
// ✅ GOOD - Reusing test infrastructure
use crate::test_utils::{TestDir, GitOperations, get_git_impl, GitRepoFixture};

let fixture = GitRepoFixture::tagged("v1.0.0")?;
let test_dir = TestDir::new()?;
let git_impl = get_git_impl();

// ❌ BAD - Reimplementing test utilities
fn create_test_dir() -> PathBuf {
    // Custom temp directory logic that already exists in TestDir
}
```

---

## ⚡ Performance Standards

- Parse 1000+ versions in <100ms
- Minimal VCS command calls (batch when possible)
- Use compiled regex patterns for speed
- Zero-copy string operations where possible

---

## 🚀 CI/CD

### Multi-Platform Testing

- **Linux**: Native Git + Docker tests enabled
- **macOS**: Native Git + Docker tests skipped
- **Windows**: Native Git + Docker tests skipped

### Pre-commit Hooks

The project uses pre-commit hooks for:

- Code formatting (rustfmt)
- Linting (clippy)
- Running tests

**IMPORTANT**: Ensure `make lint` and `make test` always pass before committing.

### GitHub Actions

Main workflows:

- `ci-test.yml`: Runs tests across Linux, macOS, Windows
- `ci-pre-commit.yml`: Validates formatting and linting
- `cd.yml`: Release automation
- `security.yml`: Security scanning with SonarCloud

---

## 📋 Running Tests

```bash
# Run all tests
cargo test

# Run git-related tests
cargo test git

# Run specific test file
cargo test --test integration_test_name

# Run with features
cargo test --features test-utils

# Run a single test
cargo test test_name -- --exact

# Run without Docker (fast, coverage gaps)
ZERV_TEST_DOCKER=false cargo test

# Run with full coverage
ZERV_TEST_DOCKER=true cargo test
```

---

## ⚙️ Configuration

Centralized config in `src/config.rs`:

- Loads environment variables (`ZERV_TEST_NATIVE_GIT`, `ZERV_TEST_DOCKER`)
- Validates boolean parsing
- Single source of truth for environment configuration

---

## 📖 CLI Implementation Standards

### Core Commands

- `zerv version [OPTIONS]` - Main version processing pipeline
- `zerv check <version> [OPTIONS]` - Validation-only command

### Pipeline Architecture

```
Input → Version Object → Zerv Object → Transform → Output Version Object → Display
```

### Essential CLI Options

**Input Sources:**

- `--source git` (default) - Auto-detect Git
- `--source stdin` - Read version from stdin

**Schema Control:**

- `--schema zerv-default` (default) - Tier-aware schema
- `--schema-ron <ron>` - Custom RON schema

**Output Control:**

- `--output-format <format>` - Target format: pep440, semver
- `--output-template <template>` - Custom template string
- `--output-prefix [prefix]` - Add prefix (defaults to "v")

---

## ✅ Validation Checks

When user mentions specific checks, run comprehensive audits:

### Error Standards Check

Triggers: "check error standards", "find error violations", "audit error handling"

Search for:

- `io::Error::new(io::ErrorKind::Other` patterns
- Missing `ZervError` usage
- Direct `unwrap()` or `expect()` in production code
- Error messages without context

### Constants Check

Triggers: "check constants usage", "find bare strings", "audit string literals"

Search for:

- Bare string literals in match statements
- Hardcoded field names
- Magic strings in validation logic

### Code Reuse Check

Triggers: "check code reuse", "find duplicated code", "audit code duplication"

Search for:

- Duplicated test setup patterns
- Direct `DockerGit`/`NativeGit` usage instead of `get_git_impl()`
- Reimplemented Git operations
- Similar helper functions across files

### Flaky Test Check

Triggers: "check for flaky tests", "test stability", "race condition"

Action: Run `make test_flaky` (5 iterations) to detect instability

---

## 🎯 Summary of Critical Rules

**MANDATORY - These rules must ALWAYS be followed:**

1. ✅ **Use constants** instead of bare strings (fields, formats, sources, schemas)
2. ✅ **Use `ZervError`** for custom errors and `io::Error::other()` for IO errors
3. ✅ **Use `get_git_impl()`** for environment-aware Git operations
4. ✅ **Use `should_run_docker_tests()`** for Docker-dependent tests
5. ✅ **Never reuse Git implementations** across different directories
6. ✅ **Include detailed error context** in all error messages
7. ✅ **NO useless comments** - only comment when code cannot explain itself
8. ✅ **Check existing utilities** in `src/test_utils/` before creating new ones
9. ✅ **Place `use` statements at top of file/module** - never inside functions
10. ✅ **Use `mod` blocks for test organization** - not comment dividers

**NEVER do these:**

1. ❌ Use bare string literals for field/format/source names
2. ❌ Use `unwrap()` or `expect()` in production code
3. ❌ Add comments that just repeat function names or restate code
4. ❌ Write useless comments that restate what's already obvious from function/variable names
5. ❌ Create Git fixtures without proper isolation
6. ❌ Skip Docker test gating for Docker-dependent tests
7. ❌ Write generic error messages without context
8. ❌ Duplicate code instead of using existing utilities
9. ❌ Place `use` statements inside functions (except rare naming conflict cases)
10. ❌ Use comment dividers for test grouping (use `mod` blocks instead)
