# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

Zerv is a dynamic versioning CLI tool written in Rust that generates versions for any commit from git and other version control systems. It supports multiple version formats (SemVer, PEP440, CalVer) and is designed for CI/CD builds.

---

## 🎯 Critical Rules (Quick Reference)

**MANDATORY - These rules must ALWAYS be followed:**

1. ✅ **Use constants** instead of bare strings (fields, formats, sources, schemas) → Use `crate::utils::constants::*`
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
4. ❌ Create Git fixtures without proper isolation
5. ❌ Skip Docker test gating for Docker-dependent tests
6. ❌ Write generic error messages without context
7. ❌ Duplicate code instead of using existing utilities
8. ❌ Place `use` statements inside functions (except rare naming conflict cases)
9. ❌ Use comment dividers for test grouping (use `mod` blocks instead)

**📋 Run `/audit-all` to check for violations automatically**

---

## Essential Commands

### Development Setup

```bash
make setup_dev  # Install pre-commit hooks and cargo-tarpaulin
```

### Testing

```bash
make test       # Full test suite: Docker Git + Docker tests enabled (full coverage)
make test_flaky # Run 5 iterations to detect flaky tests
```

### Code Quality

```bash
make lint       # Check code formatting and clippy warnings
make update     # Update Rust toolchain and dependencies
```

### Slash Commands

```bash
/audit-all      # Run comprehensive code quality audit (comments, imports, constants, etc.)
```

---

## 📚 Always Check .dev First

**BEFORE performing ANY coding task, read `.dev/00-README.md` for complete project context and workflow.**

### .dev Document Numbering

- `00-***.md`: Current state (baseline)
- `01-***.md`: Next development phase
- `02-***.md`: Following phase

**Higher numbers = more recent/updated plans**. Always verify against actual codebase.

### Creating New Plans

**CRITICAL: When user asks you to "write a plan" or "create a plan", you MUST:**

1. ✅ **Check existing `.dev/` files** to find the highest number
2. ✅ **Create new plan** as `.dev/XX-descriptive-name.md` (increment XX)
3. ✅ **Do NOT start coding** until the plan is reviewed and approved
4. ✅ **Use ExitPlanMode tool** to present the plan for approval

**Example workflow:**

```bash
# Find latest plan number
ls -1 .dev/ | grep -E '^[0-9]+-.*\.md$' | sort -V | tail -1
# Returns: 27-integration-tests-revamp-plan.md

# Create next plan
# New file: .dev/28-logging-implementation-plan.md
```

**Plan Document Structure:**

- **Status**: Planned/In Progress/Completed
- **Priority**: High/Medium/Low
- **Context**: Why this work is needed
- **Goals**: What we want to achieve
- **Implementation Plan**: Detailed steps
- **Testing Strategy**: How to validate
- **Success Criteria**: Definition of done
- **Documentation Updates**: What docs need updating

---

## High-Level Architecture

### Pipeline Architecture

```
Input → VCS Detection → Version Parsing → Transformation → Format Output
```

**Key Modules:**

- **`src/vcs/`**: Version Control System abstraction (Git only) - extracts metadata
- **`src/version/`**: Version format implementations (PEP440, SemVer, Zerv)
- **`src/pipeline/`**: Data transformation layer
- **`src/schema/`**: Schema and preset management (RON-based)
- **`src/cli/`**: Command-line interface

### State-Based Versioning Tiers

- **Tier 1** (Tagged, clean): `major.minor.patch`
- **Tier 2** (Distance, clean): `major.minor.patch.post<distance>+branch.<commit>`
- **Tier 3** (Dirty): `major.minor.patch.dev<timestamp>+branch.<commit>`

### Test Infrastructure

- **Environment-aware Git testing**: Uses `DockerGit` locally, `NativeGit` in CI
- **`GitOperations` trait**: Unified interface for both implementations
- **`GitRepoFixture`**: Creates isolated test repositories with specific states
- **`TestDir`**: Temporary directory management with automatic cleanup

---

## 🚨 Code Style Policies

### Comment Policy

**Golden Rule**: If a comment just repeats what the code or function name already says, DELETE IT.

**Only comment when**:

- Explaining complex algorithms or non-obvious behavior
- Documenting domain-specific rules (e.g., "PEP440 ordering: dev < alpha < beta < rc")
- Clarifying surprising edge cases

**Never**:

- Restate function names: `/// Converts VCS data` for `fn vcs_data_to_zerv_vars()`
- Obvious comments: `// Initialize repo`, `// Return value`
- Section dividers: `// ======== Tests ========` (use `mod` blocks instead)

### Import Statement Policy

**Always place `use` statements at the top of the file or module, never inside functions.**

Exception: Rare naming conflicts requiring scope limitation (`use X as Y`)

### Test Organization Policy

**Use Rust modules for test organization, not comment-based grouping.**

```rust
// ✅ GOOD - Module-based
mod feature_basic {
    use super::*;
    #[test]
    fn test_something() { }
}

// ❌ BAD - Comment-based
// ============ Feature Basic Tests ============
#[test]
fn test_something() { }
```

### Line Length Policy

**Keep lines reasonably short when adding/updating code.**

Rustfmt enforces `max_width = 100` for code but can't break string literals. For long strings, use `format!()` or string continuation. Check with `/audit-all` periodically.

---

## 🔒 Error Handling Standards

### Rules

1. **ALWAYS use `zerv::error::ZervError`** for custom errors
2. **Use `io::Error::other()`** instead of `io::Error::new(io::ErrorKind::Other, ...)`
3. **Include context** in error messages (what failed, where, why)
4. **NEVER use `unwrap()` or `expect()`** in production code (only in tests)

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

// ❌ BAD - Generic error
let file = fs::read_to_string(&path)?;

// ❌ BAD - Old pattern
return Err(ZervError::Io(io::Error::new(io::ErrorKind::Other, "failed")));
```

---

## 🎯 Constants Usage - MANDATORY

**NEVER use bare string literals for field names, formats, sources, or schema names.**

### Available Constants

```rust
use crate::utils::constants::{fields, formats, sources, schema_names};

// Field names: fields::MAJOR, fields::MINOR, fields::PATCH, etc.
// Format names: formats::SEMVER, formats::PEP440, formats::ZERV
// Source names: sources::GIT, sources::STDIN
// Schema names: schema_names::ZERV_STANDARD, schema_names::ZERV_CALVER
```

### Example

```rust
// ✅ GOOD
use crate::utils::constants::{fields, formats};

match field_name.as_str() {
    fields::MAJOR => handle_major(),
    fields::MINOR => handle_minor(),
    _ => return Err(ZervError::UnknownField(field_name.to_string())),
}

// ❌ BAD
match field_name.as_str() {
    "major" => handle_major(),
    "minor" => handle_minor(),
    _ => return Err(ZervError::UnknownField(field_name.to_string())),
}
```

---

## 🧪 Testing Standards

### Environment Variables

- `ZERV_TEST_NATIVE_GIT=true`: Use native Git (set in CI)
- `ZERV_TEST_DOCKER=true`: Enable Docker-dependent tests

### Environment-Aware Git Testing

**MANDATORY: Always use `get_git_impl()` for environment-aware Git operations.**

```rust
use crate::test_utils::{GitOperations, get_git_impl};

// ✅ GOOD
let git_impl = get_git_impl();
git_impl.init_repo(&test_dir)?;

// ❌ BAD
let git = DockerGit::new();
git.init_repo(&test_dir)?;
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

```rust
#[test]
fn test_example() {
    // 1. Docker test gating (if needed)
    if !should_run_docker_tests() {
        return;
    }

    // 2. Setup - Create isolated resources
    let fixture = GitRepoFixture::tagged("v1.0.0")
        .expect("Failed to create tagged repo - check Docker availability");

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

**✅ DO:**

- Create fresh Git implementation per directory: `get_git_impl()`
- Isolate resources per test: `GitRepoFixture::tagged("v1.0.0")?`
- Verify state after setup
- Use detailed error messages in `expect()`
- Include context in assertions

**❌ DON'T:**

- Reuse Git implementations across directories (race conditions!)
- Share state between tests (use `static` fixtures)
- Skip verification steps
- Use generic error messages

### Docker Flaky Test Troubleshooting

If you encounter flaky Docker-related Git test failures, check the retry logic at `src/test_utils/git/docker.rs:208-227`.

**Common transient error patterns** (should auto-retry):

- `"cannot update ref"` - Git reference update race condition
- `"nonexistent object"` - Object not yet visible to Git
- `"is not a valid object"` - Object consistency check failure

Add new patterns to the retry condition if needed. Do NOT add retry logic for legitimate errors.

---

## 🔄 Code Reuse Standards

**MANDATORY: Always check existing utilities before creating new ones.**

1. **Check `src/test_utils/` first** before creating test utilities
2. **Reuse existing infrastructure**: `TestDir`, `GitOperations`, `GitRepoFixture`
3. **Use `get_git_impl()`** for environment-aware Git operations
4. **Prefer `GitOperations` trait methods** over direct Docker/Native calls

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

- Code formatting (rustfmt)
- Linting (clippy)
- Running tests

**IMPORTANT**: Ensure `make lint` and `make test` always pass before committing.

### GitHub Actions

- `ci-test.yml`: Runs tests across Linux, macOS, Windows
- `ci-pre-commit.yml`: Validates formatting and linting
- `cd.yml`: Release automation
- `security.yml`: Security scanning with SonarCloud

---

## 📋 Running Tests

```bash
# Run all tests
cargo test

# Run specific test
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

## 🛠️ Using Claude Code Features

### When to Use What

**Slash Commands** (Simple, predefined workflows):

- `/audit-all` - Run comprehensive code quality audit
- Use for: Deterministic tasks, checklists, audits

**Agents** (Complex, multi-step exploration):

- Codebase exploration: "How does version bumping work across the codebase?"
- Large refactoring: "Migrate all tests to use environment-aware pattern"
- Research tasks: "Find all API endpoints and document them"
- Use for: Open-ended tasks requiring autonomous decision-making

**Direct Questions** (Fastest for simple tasks):

- "Review this function for bugs"
- "Explain how this works"
- "Fix the error in this code"
- Use for: Quick reviews, explanations, single-file changes

**Rule of thumb**:

1. Try direct question first (fastest)
2. Use slash command for standardized workflows
3. Use agent only for complex multi-file exploration/refactoring
