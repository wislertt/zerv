# Centralized Config Implementation - Completed

## Summary

Successfully centralized environment variable loading in `src/config.rs` and fixed Docker test behavior to fail properly when Docker is unavailable but `ZERV_TEST_DOCKER=true`.

## Changes Made

### 1. Centralized Configuration (`src/config.rs`)

**Replaced config crate with direct environment variable parsing:**

```rust
#[derive(Debug, Clone, Default)]
pub struct ZervConfig {
    pub test_native_git: bool,  // renamed from ci
    pub test_docker: bool,      // new field
}

impl ZervConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let test_native_git = Self::parse_bool_env("ZERV_TEST_NATIVE_GIT")?;
        let test_docker = Self::parse_bool_env("ZERV_TEST_DOCKER")?;

        Ok(ZervConfig { test_native_git, test_docker })
    }

    fn parse_bool_env(var_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match env::var(var_name) {
            Ok(val) => Ok(val == "true" || val == "1"),
            Err(_) => Ok(false),
        }
    }
}
```

**Added comprehensive tests:**

- `test_default_config()` - verifies defaults when no env vars set
- `test_native_git_env_var()` - tests `ZERV_TEST_NATIVE_GIT=true`
- `test_docker_tests_env_var()` - tests `ZERV_TEST_DOCKER=true`
- `test_both_env_vars()` - tests both variables together
- `test_false_values()` - tests explicit `false` values

### 2. Updated Helper Functions (`src/test_utils/mod.rs`)

**Replaced scattered environment variable access:**

```rust
pub fn should_run_docker_tests() -> bool {
    ZervConfig::load()
        .map(|config| config.should_run_docker_tests())
        .unwrap_or(false)
}
```

### 3. Fixed Docker Test Logic (`src/test_utils/git/docker.rs`)

**Fixed tests with `_without_docker` suffix to skip when Docker tests are enabled:**

```rust
#[test]
fn test_docker_git_commands_without_docker(#[case] args: &[&str]) {
    if should_run_docker_tests() {
        return; // Skip when Docker tests are enabled
    }
    // test code
}
```

### 4. Removed Dependencies

**Removed problematic `config` crate from `Cargo.toml`:**

- The config crate had boolean parsing issues
- Direct environment variable parsing is more reliable

## Environment Variable Matrix

| Scenario         | `ZERV_TEST_NATIVE_GIT` | `ZERV_TEST_DOCKER` | Git Implementation | Docker Tests | Result             |
| ---------------- | ---------------------- | ------------------ | ------------------ | ------------ | ------------------ |
| Local Easy       | `false`                | `false`            | Docker Git         | Skipped      | Coverage with gaps |
| Local Full       | `false`                | `true`             | Docker Git         | Run          | Full coverage      |
| CI Linux         | `true`                 | `true`             | Native Git         | Run          | Platform coverage  |
| CI macOS/Windows | `true`                 | `false`            | Native Git         | Skipped      | Platform coverage  |

## Key Behavior Changes

### Before

- Docker tests silently passed when Docker was unavailable
- Environment variables scattered across codebase
- Config crate had boolean parsing issues

### After

- Docker tests **fail** with clear error messages when `ZERV_TEST_DOCKER=true` but Docker unavailable
- All environment variable loading centralized in `src/config.rs`
- Comprehensive test coverage for config loading
- Proper test separation between Docker-dependent and Docker-independent tests

## Verification

**Test Results:**

- `make test_easy` (Docker closed) → ✅ Passes (Docker tests skipped)
- `make test` (Docker closed) → ❌ Fails with "Cannot connect to the Docker daemon" (as expected)
- Config tests → ✅ All 5 tests pass with proper environment variable handling

## Files Modified

1. `src/config.rs` - Centralized config with tests
2. `src/test_utils/mod.rs` - Updated helper functions
3. `src/test_utils/git/docker.rs` - Fixed Docker test logic
4. `Cargo.toml` - Removed config crate dependency

## Benefits Achieved

- **Centralized Control**: All environment variable loading in one place
- **Proper Failure Behavior**: Docker tests fail when expected, don't silently pass
- **Comprehensive Testing**: Full test coverage for config functionality
- **Simplified Dependencies**: Removed problematic config crate
- **Clear Separation**: Docker tests vs non-Docker tests properly separated
