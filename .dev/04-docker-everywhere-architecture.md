# Multi-Platform Testing Architecture Plan

## ✅ IMPLEMENTATION COMPLETED

**Status**: All phases implemented and tested successfully across all platforms.

**CI Results**:

- ✅ **Linux CI**: All tests pass with NativeGit
- ✅ **macOS CI**: All tests pass with NativeGit
- ✅ **Windows CI**: All tests pass with NativeGit

**Architecture Achieved**:

```
Local:  Tests → DockerGit ✅ (Safe isolation)
CI:     Tests → NativeGit ✅ (Real Windows/macOS/Linux testing)
```

## Current State Analysis (After Revert)

### ✅ What's Working

- **Local Tests**: All 1132 tests pass locally (94.50% coverage)
- **DockerGit Implementation**: Clean implementation in `src/test_utils/git.rs`
- **CI Stable**: Linux-only CI working
- **Docker Usage**: Already using Docker for all Git operations in tests

### Current Architecture

- **DockerGit**: Handles all Git operations via `alpine/git:latest`
- **VCS Git Tests**: Uses inline Docker commands (needs cleanup)
- **VCS Fixtures**: Uses DockerGit properly ✅
- **Pipeline Tests**: Uses VCS fixtures properly ✅

## Target Architecture: Option A - Native Git in CI, Docker for Local Safety

### Core Principle

**Local: Docker for isolation | CI: Native Git for real platform testing**

### Benefits

1. **Local Safety**: Docker isolation never touches your git config
2. **Real Platform Testing**: CI tests actual Windows/macOS/Linux Git behavior
3. **Catches Platform Issues**: Path separators, Git behavior differences, compilation issues
4. **Simple**: Two clear environments, no complex hybrid logic

## Implementation Plan

### Phase 1: Create Native Git Test Utility

#### 1.1 Create Git Module Structure

**Purpose:** Organize Git implementations into separate files for maintainability

**File Structure:**

```
src/test_utils/git/
├── mod.rs          # GitOperations trait + public exports
├── docker.rs       # DockerGit implementation (move from git.rs)
└── native.rs       # NativeGit implementation
```

#### 1.2 Add Git Trait

**File:** `src/test_utils/git/mod.rs`

**Implementation:**

```rust
use super::TestDir;
use std::io;

mod docker;
mod native;

pub use docker::DockerGit;
pub use native::NativeGit;

/// Common Git operations trait for both Docker and Native implementations
pub trait GitOperations {
    /// Execute a git command with the given arguments
    fn execute_git(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String>;

    /// Initialize a git repository with initial commit (shared logic)
    fn init_repo(&self, test_dir: &TestDir) -> io::Result<()> {
        test_dir.create_file("README.md", "# Test Repository")?;
        self.execute_git(test_dir, &["init"])?;
        self.execute_git(test_dir, &["config", "user.name", "Test User"])?;
        self.execute_git(test_dir, &["config", "user.email", "test@example.com"])?;
        self.execute_git(test_dir, &["add", "."])?;
        self.execute_git(test_dir, &["commit", "-m", "Initial commit"])?;
        Ok(())
    }

    /// Create a git tag (shared logic)
    fn create_tag(&self, test_dir: &TestDir, tag: &str) -> io::Result<()> {
        self.execute_git(test_dir, &["tag", tag])?;
        Ok(())
    }

    /// Create a commit (shared logic)
    fn create_commit(&self, test_dir: &TestDir, message: &str) -> io::Result<()> {
        self.execute_git(test_dir, &["add", "."])?;
        self.execute_git(test_dir, &["commit", "-m", message])?;
        Ok(())
    }
}
```

#### 1.3 Move DockerGit Implementation

**File:** `src/test_utils/git/docker.rs`

**Implementation:**

```rust
use super::{GitOperations, TestDir};
use std::io;
use std::process::Command;

// Move entire DockerGit implementation from git.rs here
// Add GitOperations trait implementation:
impl GitOperations for DockerGit {
    fn execute_git(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String> {
        self.run_git_command(test_dir, args)
    }
}
```

#### 1.4 Add NativeGit Implementation

**File:** `src/test_utils/git/native.rs`

**Implementation:**

```rust
use super::{GitOperations, TestDir};
use std::io;
use std::process::Command;

/// Native Git implementation for CI testing
pub struct NativeGit;

impl NativeGit {
    pub fn new() -> Self { Self }
}

impl GitOperations for NativeGit {
    fn execute_git(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(test_dir.path())
            .output()?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "Git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
```

#### 1.5 Update Test Utils Module

**File:** `src/test_utils/mod.rs`

**Update git import:**

```rust
// Change from:
// pub mod git;
// To:
pub mod git;
pub use git::{DockerGit, NativeGit, GitOperations};
```

### Phase 2: Add Configuration Management

#### 2.1 Add Config Crate Integration

**File:** `Cargo.toml`

**Rationale:** Add `config` crate now to avoid guaranteed refactoring in Phase 4 alpha. Current simple approach would require breaking changes when `zerv.toml` support is added.

**Dependencies:**

```toml
[dependencies]
config = "0.14"
serde = { version = "1.0", features = ["derive"] }
```

#### 2.2 Create Configuration Structure

**File:** `src/config.rs`

**Implementation:**

```rust
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct ZervConfig {
    pub ci: bool,
}

impl Default for ZervConfig {
    fn default() -> Self {
        Self { ci: false }
    }
}

impl ZervConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let mut builder = Config::builder()
            .add_source(Environment::with_prefix("ZERV").separator("_"))
            .add_source(Environment::default().try_parsing(true));

        // Future: Add zerv.toml support in Phase 4
        // if Path::new("zerv.toml").exists() {
        //     builder = builder.add_source(File::with_name("zerv.toml"));
        // }

        builder.build()?.try_deserialize()
    }

    pub fn should_use_native_git(&self) -> bool {
        self.ci
    }
}
```

#### 2.3 Add CI Detection Function

**File:** `src/test_utils/mod.rs`

**Implementation:**

```rust
use crate::config::ZervConfig;

pub fn should_use_native_git() -> bool {
    ZervConfig::load()
        .map(|config| config.should_use_native_git())
        .unwrap_or(false)
}
```

#### 2.4 Update Module Structure

**File:** `src/lib.rs`

**Add:**

```rust
pub mod config;
```

#### 2.5 Update Test Setup Functions

**File:** `src/vcs/git.rs`

**Replace inline Docker commands with environment-aware setup:**

```rust
use crate::test_utils::git::{DockerGit, NativeGit, GitOperations};
use crate::test_utils::should_use_native_git;

fn get_git_impl() -> Box<dyn GitOperations> {
    if should_use_native_git() {
        Box::new(NativeGit::new())
    } else {
        Box::new(DockerGit::new())
    }
}

fn setup_git_repo() -> TestDir {
    let test_dir = TestDir::new().expect("should create temp dir");
    let git = get_git_impl();
    git.init_repo(&test_dir).expect("should init repo");
    test_dir
}

fn setup_git_repo_with_tag(tag: &str) -> TestDir {
    let test_dir = setup_git_repo();
    let git = get_git_impl();
    git.create_tag(&test_dir, tag).expect("should create tag");
    test_dir
}
```

#### 2.6 Update VCS Fixtures

**File:** `src/test_utils/vcs_fixtures.rs`

**Make fixtures CI-aware:**

```rust
fn create_vcs_data_with_tag(tag: &str, filename: &str, content: &str, commit_msg: &str) -> VcsData {
    let test_dir = TestDir::new().expect("Failed to create test dir");
    let git = get_git_impl();

    git.init_repo(&test_dir).expect("Failed to init repo");
    git.create_tag(&test_dir, tag).expect("Failed to create tag");

    // Add file and commit
    test_dir.create_file(filename, content).expect("Failed to create file");
    git.create_commit(&test_dir, commit_msg).expect("Failed to create commit");

    let git_vcs = GitVcs::new(test_dir.path()).expect("Failed to create GitVcs");
    git_vcs.get_vcs_data().expect("Failed to get VCS data")
}
```

### Phase 3: Multi-Platform CI Implementation

#### 3.1 Update CI Workflow

**File:** `.github/workflows/ci-test.yml`

**Current:** Only tests on Linux (`runs-on: ubuntu-latest`)
**Target:** Test on all platforms with native Git

```yaml
strategy:
    fail-fast: false
    matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
env:
    ZERV_CI: true # Enables native Git testing via config crate
```

**This change enables:**

- **Windows testing**: Real Windows Git behavior, CRLF line endings, Windows paths
- **macOS testing**: Real macOS Git behavior, case-insensitive filesystem
- **Linux testing**: Real Linux Git behavior (baseline)

#### 3.2 Git Availability on GitHub Actions

**All runners have native Git:**

- Ubuntu: Git pre-installed ✅
- macOS: Git pre-installed ✅
- Windows: Git pre-installed ✅

## Files Requiring Changes

### High Priority (Core Implementation)

1. **`src/test_utils/git/mod.rs`** - Create git module with GitOperations trait
2. **`src/test_utils/git/docker.rs`** - Move DockerGit from git.rs + implement GitOperations
3. **`src/test_utils/git/native.rs`** - Add NativeGit implementation
4. **`src/test_utils/mod.rs`** - Update git module import
5. **`Cargo.toml`** - Add config and serde dependencies
6. **`src/config.rs`** - Create configuration management structure
7. **`src/lib.rs`** - Add config module
8. **`src/test_utils/mod.rs`** - Add CI detection function using config
9. **`src/vcs/git.rs`** - Replace inline Docker with trait-based setup
10. **`src/test_utils/vcs_fixtures.rs`** - Make fixtures use trait-based approach

### Medium Priority (CI Setup)

11. **`.github/workflows/ci-test.yml`** - **CRITICAL**: Change from Linux-only to multi-platform matrix (Windows, macOS, Linux) with ZERV_CI=true

### Low Priority (No Changes Needed)

12. **`src/pipeline/vcs_data_to_zerv_vars.rs`** - Already uses fixtures ✅

## ✅ FINAL IMPLEMENTATION STATUS

### ✅ All Phases Complete

**Phase 1**: Git module structure with trait-based code reuse ✅
**Phase 2**: Configuration management with config crate ✅
**Phase 3**: Multi-platform CI implementation ✅

### ✅ Platform-Specific Fixes Applied

**Windows Compatibility**:

- ✅ Fixed `getuid()`/`getgid()` Unix-specific functions
- ✅ Fixed integration test commands (`cmd` vs `/bin/echo`)
- ✅ Excluded DockerGit tests (Windows uses NativeGit only)

**macOS Compatibility**:

- ✅ Excluded DockerGit tests (macOS uses NativeGit only)
- ✅ Docker not available on macOS CI runners

**Linux Compatibility**:

- ✅ Maintains DockerGit tests for local development
- ✅ Uses NativeGit in CI environment
- ✅ Handles coverage reporting (SonarCloud + Codecov)

### ✅ CI Optimizations

**Coverage Reporting**: Only uploads from Linux (eliminates redundancy)
**Test Distribution**:

- Linux: Runs all tests including DockerGit unit tests
- Windows/macOS: Runs all tests except DockerGit unit tests

## Expected Outcome

### Before (Original Linux-only)

```
Local:  Tests → DockerGit ✅ (Linux container)
CI:     Tests → DockerGit ✅ (Linux container only)
```

### After (✅ ACHIEVED - Multi-platform)

```
Local:  Tests → DockerGit ✅ (Safe isolation)
CI:     Tests → NativeGit ✅ (Real Windows/macOS/Linux testing)
```

## ✅ Benefits Realized

### Real Platform Testing

- ✅ **Windows CI**: Tests actual Windows Git behavior, CRLF line endings, Windows paths
- ✅ **macOS CI**: Tests actual macOS Git behavior, case-insensitive filesystem
- ✅ **Linux CI**: Tests actual Linux Git behavior, permissions

### Local Safety

- ✅ Docker isolation protects your personal git config
- ✅ No risk of test interference with your work
- ✅ Consistent local development experience

### Future-Proof Configuration

- ✅ **Config crate**: Prevents guaranteed refactoring in Phase 4 alpha
- ✅ **Environment variables**: `ZERV_CI=true` works immediately
- ✅ **Ready for zerv.toml**: Just uncomment file source in Phase 4
- ✅ **No breaking changes**: Configuration API stays consistent

### Code Reuse & Maintainability

- ✅ **GitOperations trait**: Shared logic for `init_repo`, `create_tag`, `create_commit`
- ✅ **Single source of truth**: Git workflows defined once, executed differently
- ✅ **Easy testing**: Both implementations use same test patterns
- ✅ **DRY principle**: No duplication of Git operation sequences
- ✅ **Polymorphism**: `get_git_impl()` eliminates conditional logic everywhere

### Simplicity

- ✅ Two clear environments, no hybrid complexity
- ✅ Professional configuration management from start
- ✅ Trait-based polymorphism eliminates conditional logic
- ✅ Minimal code changes needed

---

## 🎉 IMPLEMENTATION COMPLETE

**Multi-platform CI testing is now fully operational with real platform testing on Windows, macOS, and Linux while maintaining Docker isolation for local development safety.**ocal: Tests → DockerGit ✅ (Linux container)
CI: Tests → DockerGit ✅ (Linux container)

```

### After (Option A - Native Git in CI)

```

Local: Tests → DockerGit ✅ (Safe isolation)
CI: Tests → NativeGit ✅ (Real Windows/macOS/Linux testing)

```

## Risk Mitigation

### Git Availability

- All GitHub Actions runners have Git pre-installed
- Local development keeps Docker isolation (safe)
- NativeGit is simple - no complex platform-specific code

### Test Consistency

- Most test logic stays the same (just different Git execution)
- Environment detection is simple boolean check
- Clear separation: Docker locally, Native in CI

### Platform Issues

- CI will catch real platform-specific problems
- Temp directory isolation prevents config contamination
- Simple native Git commands, no complex setup

## Benefits of This Approach

### Real Platform Testing

- **Windows CI**: Tests actual Windows Git behavior, paths, line endings
- **macOS CI**: Tests actual macOS Git behavior, case sensitivity
- **Linux CI**: Tests actual Linux Git behavior, permissions

### Local Safety

- Docker isolation protects your personal git config
- No risk of test interference with your work
- Consistent local development experience

### Future-Proof Configuration

- **Config crate now**: Prevents guaranteed refactoring in Phase 4 alpha
- **Environment variables**: `ZERV_CI=true` works immediately
- **Ready for zerv.toml**: Just uncomment file source in Phase 4
- **No breaking changes**: Configuration API stays consistent

### Code Reuse & Maintainability

- **GitOperations trait**: Shared logic for `init_repo`, `create_tag`, `create_commit`
- **Single source of truth**: Git workflows defined once, executed differently
- **Easy testing**: Both implementations use same test patterns
- **DRY principle**: No duplication of Git operation sequences
- **Polymorphism**: `get_git_impl()` eliminates conditional logic everywhere

### Simplicity

- Two clear environments, no hybrid complexity
- Professional configuration management from start
- Trait-based polymorphism eliminates conditional logic
- Minimal code changes needed


```
