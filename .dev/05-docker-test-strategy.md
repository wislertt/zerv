# Docker Test Environment Strategy - Implementation Plan

## Current State

**Makefile:**

- `test_easy`: `cargo test`
- `test`: `cargo tarpaulin [...] -- --include-ignored`

**CI:**

- Uses `ZERV_CI=true` environment variable
- Runs `make test` on all platforms
- Docker tests fail gracefully on macOS/Windows

**Tests:**

- 15 tests have `#[ignore = "docker"]` attribute
- Tests run via `--include-ignored` flag

## Goal

Replace `#[ignore = "docker"]` with runtime environment variable control:

- `ZERV_TEST_NATIVE_GIT`: controls Git implementation (rename from `ZERV_CI`)
- `ZERV_TEST_DOCKER`: controls Docker test execution

## Implementation Steps

### 1. Update `src/config.rs`

```rust
pub struct ZervConfig {
    pub test_native_git: bool,  // was: ci
    pub test_docker: bool,      // NEW
}

pub fn should_use_native_git(&self) -> bool {
    self.test_native_git  // was: self.ci
}

pub fn should_run_docker_tests(&self) -> bool {
    self.test_docker
}
```

### 2. Update `src/test_utils/mod.rs`

```rust
pub fn should_run_docker_tests() -> bool {
    ZervConfig::load()
        .map(|config| config.should_run_docker_tests())
        .unwrap_or(false)
}
```

### 3. Update Makefile

```makefile
_test:
	RUST_BACKTRACE=1 cargo tarpaulin \
		--features test-utils \
		--out Xml --out Html --out Lcov \
		--output-dir coverage \
		--include-tests --exclude-files src/main.rs

test_easy:
	ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=false $(MAKE) _test

test:
	ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true $(MAKE) _test
```

### 4. Update `.github/workflows/ci-test.yml`

```yaml
env:
    ZERV_TEST_NATIVE_GIT: true
    ZERV_TEST_DOCKER: ${{ matrix.os == 'ubuntu-latest' && 'true' || 'false' }}
steps:
    - run: make _test
```

### 5. Update Docker Tests

Replace `#[ignore = "docker"]` with runtime checks in 15 test files:

```rust
#[test]
fn test_docker_functionality() {
    if !should_run_docker_tests() {
        return;
    }
    // test code
}
```

## Result

**Local Development:**

- `make test_easy` → Docker Git, Docker tests skipped, coverage with gaps
- `make test` → Docker Git, Docker tests run, full coverage

**CI:**

- Linux: `make _test` → Native Git, Docker tests, coverage
- macOS/Windows: `make _test` → Native Git, no Docker tests, coverage

**Benefits:**

- No environment variable conflicts between local and CI
- Explicit control over Docker test execution
- Same commands work across all environments
