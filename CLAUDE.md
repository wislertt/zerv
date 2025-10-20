# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Zerv is a dynamic versioning CLI tool written in Rust that generates versions for any commit from git and other version control systems. It supports multiple version formats (SemVer, PEP440, CalVer) and is designed for CI/CD builds.

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

- **`src/cli/`**: Command-line interface (in development)
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

## Testing Standards

### Environment Variables

- `ZERV_TEST_NATIVE_GIT=true`: Use native Git (set in CI for platform testing)
- `ZERV_TEST_DOCKER=true`: Enable Docker-dependent tests (requires Docker)

### Git Testing Pattern

ALWAYS use the environment-aware pattern:

```rust
use crate::test_utils::{GitOperations, get_git_impl};

let git_impl = get_git_impl(); // Returns DockerGit or NativeGit based on environment
git_impl.init_repo(&test_dir)?;
```

### Docker Test Gating

For Docker-dependent tests:

```rust
use crate::test_utils::should_run_docker_tests;

#[test]
fn test_docker_functionality() {
    if !should_run_docker_tests() {
        return; // Skip when Docker tests are disabled
    }
    // Test code here
}
```

### Flaky Test Prevention

- Use `GitOperations` trait methods for atomic operations
- Create fresh Git implementations for each test directory
- Include detailed error messages with context
- Verify intermediate states (e.g., `.git` directory exists)
- Never reuse Git implementations across different directories

## Coding Standards

### Error Handling

- **ALWAYS** use `zerv::error::ZervError` for custom errors
- Use `io::Error::other()` instead of `io::Error::new(io::ErrorKind::Other, ...)`
- Include context in error messages for debugging
- Proper error propagation with `?` operator

### Constants Usage

**MANDATORY**: Always use constants instead of bare strings:

```rust
// GOOD
use crate::utils::constants::{fields, formats, sources, schema_names};
match field_name.as_str() {
    fields::MAJOR => // ...
    fields::MINOR => // ...
}

// BAD - Never use bare strings
match field_name.as_str() {
    "major" => // ...
}
```

### Code Reuse

Before implementing new test utilities:

- Check `src/test_utils/` for existing infrastructure
- Reuse `TestDir`, `GitOperations` trait, and other helpers
- Use `get_git_impl()` for environment-aware Git operations
- Avoid duplicating code across files

### Performance Standards

- Parse 1000+ versions in <100ms
- Minimal VCS command calls (batch when possible)
- Use compiled regex patterns for speed
- Zero-copy string operations where possible

## CI/CD

### Multi-Platform Testing

- **Linux**: Native Git + Docker tests enabled
- **macOS**: Native Git + Docker tests skipped
- **Windows**: Native Git + Docker tests skipped

### Pre-commit Hooks

The project uses pre-commit hooks for:

- Code formatting (rustfmt)
- Linting (clippy)
- Running tests

### GitHub Actions

Main workflows:

- `ci-test.yml`: Runs tests across Linux, macOS, Windows
- `ci-pre-commit.yml`: Validates formatting and linting
- `cd.yml`: Release automation
- `security.yml`: Security scanning with SonarCloud

## Important Files

### Development Documentation

Read `.dev/00-README.md` FIRST before any coding task. All `.dev/` documents use sequential numbering (00, 01, 02...) where higher numbers indicate more recent plans.

### Cursor Rules (Apply to Claude Code)

Key rules in `.cursor/rules/`:

- `dev-workflow.mdc`: Development workflow and git practices
- `testing-standards.mdc`: Comprehensive testing requirements
- `cli-implementation.mdc`: CLI standards and patterns
- `docker-git-testing.mdc`: Docker testing architecture

## Running Tests for Specific Features

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
```

## Configuration

Centralized config in `src/config.rs`:

- Loads environment variables (`ZERV_TEST_NATIVE_GIT`, `ZERV_TEST_DOCKER`)
- Validates boolean parsing
- Single source of truth for environment configuration
