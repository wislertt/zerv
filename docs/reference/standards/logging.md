# Logging Standards

## Rules

1. **Use `tracing` macros**: `error!`, `warn!`, `info!`, `debug!` (we don't use `trace!`)
2. **All logs go to stderr**: Never log to stdout (breaks piping)
3. **Choose appropriate levels**: error for failures, debug for detailed flow
4. **Include context**: What operation, which file/module, relevant values
5. **NEVER log in hot paths**: No logs inside tight loops
6. **Use test macros in test code**: `test_debug!`, `test_info!`, `test_error!`, `test_warn!`

## Log Level Guidelines

- `error!`: Unrecoverable errors (Git command failed, schema parse error)
- `warn!`: Recoverable issues (retry attempts, fallback to defaults)
- `info!`: High-level operations (1-2 per command max)
- `debug!`: Detailed flow (Git commands, transformations, RON dumps - everything)

### Test Code Logging

- `test_error!`: Test failures and setup errors
- `test_warn!`: Test warnings (slow operations, skipped tests)
- `test_info!`: Test progress and important milestones
- `test_debug!`: Detailed test flow and intermediate results

## Usage

```bash
# Default: error only (clean output)
zerv version

# Debug logs (shows everything you need)
zerv version -v
zerv version --verbose

# Power user - module-specific
RUST_LOG=zerv::vcs=debug zerv version

# Info level via RUST_LOG
RUST_LOG=info zerv version

# Trace via RUST_LOG (rarely needed)
RUST_LOG=trace zerv version

# Test debugging
RUST_LOG=debug cargo test test_name

# Test-only logs (using custom macros)
RUST_LOG=zerv_test=info cargo test test_name

# Test logs + specific source modules
RUST_LOG=zerv_test=info,zerv::cli::flow=debug cargo test test_name

# Show all logs (test + source)
RUST_LOG=debug cargo test test_name
```

## Examples

### Source Code Logging

```rust
// ✅ GOOD - Appropriate level and context
tracing::debug!("Loading schema: {}", schema_name);
tracing::error!("Failed to parse RON schema at line {}: {}", line, err);

// ❌ BAD - Wrong level
tracing::info!("Calling function xyz"); // Too verbose for info

// ❌ BAD - Missing context
tracing::error!("Parse failed"); // What failed? Where? Why?
```

### Test Code Logging

```rust
use crate::test_utils::logging::{test_debug, test_info, test_error, test_warn};

#[test]
fn test_version_generation() {
    test_debug!("Starting test: version generation");

    let fixture = GitRepoFixture::tagged("v1.0.0")
        .expect("Failed to create test fixture");
    test_info!("Created test fixture at: {}", fixture.path().display());

    let result = run_pipeline(&fixture);
    if let Err(e) = result {
        test_error!("Pipeline failed: {}", e);
        return;
    }

    test_debug!("Test completed successfully");
}

// ✅ GOOD - Test-specific logging with context
test_debug!("Flow pipeline output ({}): {}", format_name, output);
test_info!("Test setup completed for branch: {}", branch_name);

// ❌ BAD - Using source code logging in tests
tracing::debug!("Test message"); // Mixes with source code logs
```

### Test Log Filtering Examples

```bash
# Show only test logs (no source code noise)
RUST_LOG=zerv_test=debug cargo test

# Show test info + source debug
RUST_LOG=zerv_test=info,zerv::vcs=debug cargo test

# Show all logs when debugging
RUST_LOG=debug cargo test
```
