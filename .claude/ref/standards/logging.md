# Logging Standards

## Rules

1. **Use `tracing` macros**: `error!`, `warn!`, `info!`, `debug!` (we don't use `trace!`)
2. **All logs go to stderr**: Never log to stdout (breaks piping)
3. **Choose appropriate levels**: error for failures, debug for detailed flow
4. **Include context**: What operation, which file/module, relevant values
5. **NEVER log in hot paths**: No logs inside tight loops

## Log Level Guidelines

- `error!`: Unrecoverable errors (Git command failed, schema parse error)
- `warn!`: Recoverable issues (retry attempts, fallback to defaults)
- `info!`: High-level operations (1-2 per command max)
- `debug!`: Detailed flow (Git commands, transformations, RON dumps - everything)

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
```

## Examples

```rust
// ✅ GOOD - Appropriate level and context
tracing::debug!("Loading schema: {}", schema_name);
tracing::error!("Failed to parse RON schema at line {}: {}", line, err);

// ❌ BAD - Wrong level
tracing::info!("Calling function xyz"); // Too verbose for info

// ❌ BAD - Missing context
tracing::error!("Parse failed"); // What failed? Where? Why?
```
