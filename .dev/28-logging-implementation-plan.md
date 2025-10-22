# Logging Implementation Plan

**Status**: Planned
**Priority**: Medium
**Estimated Effort**: 2-3 hours

## Quick Reference

| What             | How                | Example                                 |
| ---------------- | ------------------ | --------------------------------------- |
| **Default**      | Error-only logs    | `zerv version`                          |
| **Debug**        | `-v` / `--verbose` | `zerv version -v`                       |
| **Fine-grained** | `RUST_LOG` env     | `RUST_LOG=zerv::vcs=debug zerv version` |
| **Power users**  | Trace if needed    | `RUST_LOG=trace zerv version`           |

**Note**: Debug level is sufficient for all normal debugging (including multi-line RON structures). Trace rarely needed for Zerv's simple pipeline.

## Context

As Zerv grows in complexity (VCS detection, pipeline transformations, schema parsing, Docker Git operations), debugging issues becomes harder without structured logging. Currently, the codebase has ad-hoc `println!`/`eprintln!` statements scattered across 7 files with no consistent approach.

### Why Logging Now?

1. **Complex Pipeline**: Input → VCS → Parsing → Transform → Output requires visibility
2. **Git Operations**: Docker retry logic, command failures need better debugging
3. **CI/CD Debugging**: Users need to troubleshoot version generation in pipelines
4. **Flaky Test Detection**: Better visibility into test failures
5. **User Support**: Enable detailed logs for bug reports

### Industry Standard: uv's Approach

Research shows popular Rust CLI tools (uv, ripgrep) use:

- **`tracing` + `tracing-subscriber`**: Modern structured logging framework
- **`RUST_LOG` environment variable**: Fine-grained control for power users
- **`--verbose` flag**: User-friendly logging enablement
- **Logs to stderr**: Preserves stdout for piping workflows

## Goals

### Primary Goals

1. ✅ Add structured logging using `tracing` ecosystem
2. ✅ Support `RUST_LOG` environment variable for granular control
3. ✅ Add `--verbose` / `-v` CLI flag for user-friendly logging
4. ✅ Ensure logs go to **stderr** (preserve piping support)
5. ✅ Replace existing debug `println!`/`eprintln!` statements

### Non-Goals

- ❌ Log aggregation/rotation (CLI tool, not service)
- ❌ Structured JSON logging (not needed for CLI)
- ❌ Performance tracing/metrics (out of scope)

## Architecture

### Stream Separation (Critical for Piping)

```
┌─────────────────────────────────────────────────┐
│                   zerv version                   │
├─────────────────────────────────────────────────┤
│                                                  │
│  tracing logs  →  stderr  →  Terminal (visible) │
│                                                  │
│  version output → stdout  →  Pipe/File          │
│                                                  │
└─────────────────────────────────────────────────┘
```

**Why This Matters**: Piping workflows like this must work:

```bash
zerv version -v --output-format zerv | zerv version -v --source stdin --output-format pep440
```

### Logging Levels Strategy

| Level    | When to Use            | Examples                                                         |
| -------- | ---------------------- | ---------------------------------------------------------------- |
| `error!` | Unrecoverable failures | Git command failed, schema parse error                           |
| `warn!`  | Recoverable issues     | Retry attempts, fallback to defaults                             |
| `info!`  | High-level operations  | "Starting version generation" (1-2 per command)                  |
| `debug!` | Detailed flow          | Git commands, transformation steps, multi-line RON, schema loads |

**Note**: We don't use `trace!` - debug level is sufficient for Zerv's needs.

### Module-Level Logging Targets

```
zerv                         # Root
├── zerv::vcs               # VCS operations
│   └── zerv::vcs::git      # Git-specific
├── zerv::pipeline          # Data transformations
├── zerv::schema            # Schema loading/parsing
├── zerv::version           # Format parsing/conversion
├── zerv::cli               # CLI command execution
└── zerv::test_utils        # Test infrastructure
```

## Implementation Plan

### Phase 1: Foundation (Priority 1)

#### 1.1 Add Dependencies

**File**: `Cargo.toml`

```toml
[dependencies]
tracing = "^0.1"
tracing-subscriber = { version = "^0.3", features = ["env-filter"] }
```

#### 1.2 Create Logging Module

**File**: `src/logging.rs` (new)

```rust
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize logging based on --verbose flag and RUST_LOG environment variable
///
/// Verbosity levels (simple and practical):
/// - false (default): error only
/// - true (-v / --verbose): debug (sufficient for all debugging)
///
/// Priority order:
/// 1. RUST_LOG environment variable (if set) - full control
/// 2. --verbose flag - enables debug level
/// 3. Default - error level only (Rust standard)
pub fn init_logging(verbose: bool) {
    let filter = if let Ok(rust_log) = std::env::var("RUST_LOG") {
        // RUST_LOG takes precedence over --verbose
        EnvFilter::new(rust_log)
    } else if verbose {
        // -v: debug level (sufficient for everything including RON dumps)
        EnvFilter::new("zerv=debug")
    } else {
        // Default: errors only
        EnvFilter::new("error")
    };

    fmt()
        .with_writer(std::io::stderr) // Critical: logs to stderr
        .with_env_filter(filter)
        .with_target(false) // Cleaner output for CLI
        .compact() // Compact format for terminal
        .init();
}
```

**Why This Design**:

- **Simple**: Just on/off, no confusing multiple levels
- **Practical**: Debug level handles everything (Git output, RON dumps, transformations)
- **YAGNI**: Trace level rarely needed for Zerv's simple pipeline (no complex locking/parallelism)
- **Default**: Error-only (matches env_logger standard)
- **`RUST_LOG` precedence**: Power users can still use trace if needed (`RUST_LOG=trace`)
- **Explicit `std::io::stderr`**: Ensures piping workflows work correctly
- **Compact format**: Suitable for CLI use (not JSON/structured format)
- **`with_target(false)`**: Cleaner output without module prefixes by default

#### 1.3 Add --verbose Flag (Simple Boolean)

**File**: `src/cli/app.rs`

```rust
#[derive(Parser, Debug)]
#[command(name = "zerv")]
#[command(about = "Dynamic versioning from VCS", long_about = None)]
pub struct Cli {
    /// Use verbose output (enables debug-level logs to stderr).
    /// Use RUST_LOG for fine-grained control (e.g., RUST_LOG=zerv::vcs=debug)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,

    // ... rest of fields
}
```

**Key Points**:

- Simple `bool` flag: on or off, no counting needed
- `global = true`: Works with all subcommands
- Help text mentions `RUST_LOG` for power users

**Usage Examples**:

```bash
zerv version           # error only (default)
zerv version -v        # debug (shows everything you need)
zerv version --verbose # same as -v (debug)
RUST_LOG=info zerv     # info level (overrides -v flag)
RUST_LOG=trace zerv    # trace level (if ever needed)
```

#### 1.4 Initialize Logging in main()

**File**: `src/main.rs`

```rust
mod logging;

fn main() {
    let cli = Cli::parse();

    // Initialize logging before any operations
    logging::init_logging(cli.verbose);

    tracing::debug!("Zerv started with args: {:?}", cli);

    if let Err(e) = run_cli(cli) {
        tracing::error!("Command failed: {}", e);
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

**Why**:

- Initialize early to capture all logs
- Keep `eprintln!` for error messages (user-facing, even without `--verbose`)
- Log error details for debugging

**Testing Phase 1**:

```bash
# Test that logging initializes without crashing
cargo run -- version
# Expected: Clean output, no logs

# Test verbose flag works
cargo run -- version -v
# Expected: Debug logs visible on stderr

# Test RUST_LOG override
RUST_LOG=info cargo run -- version
# Expected: Info level logs
```

### Phase 2: Strategic Log Points (Priority 1)

#### 2.1 VCS Module - Git Operations

**File**: `src/vcs/git.rs`

**High-Priority Locations**:

```rust
pub fn detect_git_version() -> Result<GitVersionData, ZervError> {
    tracing::debug!("Detecting Git version in current directory");

    // ...

    match get_latest_tag() {
        Ok(tag) => {
            tracing::debug!("Found Git tag: {}", tag);
            // ...
        }
        Err(e) => {
            tracing::warn!("No Git tag found, using defaults: {}", e);
            // ...
        }
    }
}

fn run_git_command(args: &[&str]) -> Result<String, ZervError> {
    let cmd_str = args.join(" ");
    tracing::debug!("Running git command: git {}", cmd_str);

    let output = Command::new("git").args(args).output()
        .map_err(|e| {
            tracing::error!("Failed to execute git command: {}", e);
            ZervError::Io(io::Error::other(format!("Git command failed: {}", e)))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Git command failed: git {} - {}", cmd_str, stderr);
        // ...
    }

    tracing::debug!("Git command output: {}", output_str);
    Ok(output_str)
}
```

#### 2.2 Pipeline Module - Transformations

**Files**: `src/pipeline/*.rs`

```rust
pub fn vcs_data_to_zerv_vars(vcs_data: &VcsData) -> ZervVars {
    tracing::debug!("Converting VCS data to Zerv variables");
    tracing::debug!("VCS data: {:?}", vcs_data);

    // ... transformation logic

    tracing::debug!("Conversion complete: {} variables populated", var_count);
    vars
}
```

#### 2.3 Schema Module - Parsing

**Files**: `src/schema/*.rs`

```rust
pub fn load_schema(schema_name: &str) -> Result<Schema, ZervError> {
    tracing::debug!("Loading schema: {}", schema_name);

    match schema_name {
        schema_names::ZERV_STANDARD => {
            tracing::debug!("Using built-in zerv-standard schema");
            // ...
        }
        _ => {
            tracing::error!("Unknown schema name: {}", schema_name);
            return Err(ZervError::UnknownSchema(schema_name.to_string()));
        }
    }
}

pub fn parse_ron_schema(ron_str: &str) -> Result<Schema, ZervError> {
    tracing::debug!("Parsing RON schema ({} bytes)", ron_str.len());
    tracing::debug!("RON schema content: {}", ron_str);

    ron::from_str(ron_str).map_err(|e| {
        tracing::error!("Failed to parse RON schema: {}", e);
        ZervError::SchemaParseError(e.to_string())
    })
}
```

#### 2.4 Test Utils - Docker Git Operations

**File**: `src/test_utils/git/docker.rs`

```rust
impl GitOperations for DockerGit {
    fn run_command(&self, args: &[&str]) -> Result<String, Box<dyn Error>> {
        let cmd_str = args.join(" ");
        tracing::debug!("Docker Git command: git {}", cmd_str);

        // ... retry logic

        if is_transient_error(&stderr) {
            tracing::warn!("Transient Git error detected, retrying ({}/3): {}",
                          attempt, stderr);
            // ...
        }

        tracing::debug!("Docker Git output: {}", output);
        Ok(output)
    }
}
```

**Testing Phase 2**:

```bash
# Test Git operations show debug logs
cargo run -- version -v
# Expected: See "Detecting Git version", "Running git command" logs

# Test specific module logging
RUST_LOG=zerv::vcs=debug cargo run -- version
# Expected: Only VCS module logs visible

# Test piping still works with logs
cargo run -- version -v --output-format zerv | cargo run -- version -v --source stdin
# Expected: Logs visible, data flows through pipe correctly
```

### Phase 3: Cleanup Existing Debug Statements

#### 3.1 Find and Replace

**Target Files** (from earlier grep):

- `src/cli/version/zerv_draft.rs`
- `src/test_utils/git/docker.rs`
- `src/vcs/git.rs`
- `src/cli/app.rs`
- `src/cli/check.rs`

**Strategy**:

```rust
// ❌ REMOVE - Ad-hoc debugging
eprintln!("Debug: processing version");

// ✅ REPLACE WITH - Structured logging
tracing::debug!("Processing version");

// ✅ KEEP - User-facing error messages
eprintln!("Error: {}", e);

// ✅ KEEP - Actual program output
println!("{}", version);
```

**Testing Phase 3**:

```bash
# Verify no debug println!/eprintln! remain
grep -r "println!\|eprintln!" src/ --include="*.rs" | grep -v "// ✅"
# Expected: Only user-facing messages and actual output

# Test all previously working functionality
cargo test
# Expected: All tests pass
```

### Phase 4: Final Validation & Documentation

#### 4.1 Comprehensive Manual Testing

```bash
# Test 1: Default behavior (clean output)
cargo run -- version
# Expected: Clean version output, no logs

# Test 2: Verbose flag
cargo run -- version -v
cargo run -- version --verbose
# Expected: Debug logs to stderr, version to stdout

# Test 3: Multi-line RON in debug logs
cargo run -- version -v --output-format zerv
# Expected: RON structure visible in debug logs

# Test 4: RUST_LOG override
RUST_LOG=info cargo run -- version
RUST_LOG=trace cargo run -- version
# Expected: Respective log levels shown

# Test 5: Module-specific logging
RUST_LOG=zerv::vcs=debug cargo run -- version
# Expected: Only VCS module logs

# Test 6: Piping with verbose (CRITICAL)
cargo run -- version -v --output-format zerv | cargo run -- version -v --source stdin --output-format pep440
# Expected: Logs visible, data flows correctly

# Test 7: Redirect separation (CRITICAL)
cargo run -- version -v > version.txt 2> logs.txt
cat version.txt  # Should have version only
cat logs.txt     # Should have debug logs
```

#### 4.2 Automated Test Suite

```bash
# All existing tests should pass
cargo test

# Test with logging enabled (should not break tests)
RUST_LOG=debug cargo test

# Full test suite with Docker
make test

# Flaky test detection
make test_flaky
```

#### 4.3 Optional: Integration Test for Logging

**File**: `tests/integration_tests/logging.rs` (optional)

```rust
// Note: This is optional - logging is tested via manual verification
// Adding automated tests for logging output is complex and low value

#[test]
fn test_verbose_flag_doesnt_crash() {
    let output = Command::new(env!("CARGO_BIN_EXE_zerv"))
        .args(&["version", "--verbose"])
        .output()
        .expect("Failed to run zerv");

    assert!(output.status.success(), "Should succeed with --verbose");
}
```

## Success Criteria

### Must Have (Test After Each Phase)

**Phase 1 Complete When**:

- ✅ `cargo run -- version` compiles and runs without logs
- ✅ `cargo run -- version -v` shows debug logs on stderr
- ✅ `RUST_LOG=info cargo run -- version` works

**Phase 2 Complete When**:

- ✅ Git operations log: "Detecting Git version", "Running git command"
- ✅ Module-specific logging works: `RUST_LOG=zerv::vcs=debug`
- ✅ **CRITICAL**: Piping works with `-v` (logs on stderr, data on stdout)

**Phase 3 Complete When**:

- ✅ No ad-hoc `println!`/`eprintln!` for debugging remain
- ✅ All existing tests pass: `cargo test`

**Phase 4 Complete When**:

- ✅ Multi-line RON structures visible in debug logs
- ✅ All manual tests pass (piping, redirection, RUST_LOG)
- ✅ Full test suite passes: `make test`
- ✅ Help text explains verbosity: `zerv --help`

### Nice to Have

- ✅ Docker retry logs visible with `-v`
- ✅ Git command failures have detailed error context
- ✅ Schema parse errors show helpful debugging info

## Documentation Updates

### Update CLAUDE.md

Add new section after "Error Handling Standards":

````markdown
## 🔍 Logging Standards

### Rules

1. **Use `tracing` macros**: `error!`, `warn!`, `info!`, `debug!` (we don't use `trace!`)
2. **All logs go to stderr**: Never log to stdout (breaks piping)
3. **Choose appropriate levels**: error for failures, debug for detailed flow
4. **Include context**: What operation, which file/module, relevant values
5. **NEVER log in hot paths**: No logs inside tight loops

### Log Level Guidelines

- `error!`: Unrecoverable errors (Git command failed, schema parse error)
- `warn!`: Recoverable issues (retry attempts, fallback to defaults)
- `info!`: High-level operations (1-2 per command max)
- `debug!`: Detailed flow (Git commands, transformations, RON dumps - everything)

### Usage

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
````

### Examples

```rust
// ✅ GOOD - Appropriate level and context
tracing::debug!("Loading schema: {}", schema_name);
tracing::error!("Failed to parse RON schema at line {}: {}", line, err);

// ❌ BAD - Wrong level
tracing::info!("Calling function xyz"); // Too verbose for info

// ❌ BAD - Missing context
tracing::error!("Parse failed"); // What failed? Where? Why?
```

````

### Update .dev/26-zerv-cli-comprehensive-documentation.md

Add section before "Error Handling":

```markdown
## Debugging and Logging

### Enable Verbose Output

```bash
# Default: Clean output (errors only)
zerv version

# Show debug logs (everything you need)
zerv version -v
zerv version --verbose

# Works with all commands
zerv check "1.2.3" -v
````

### Verbosity Levels Explained

| Flag               | Level | What You See                              | When to Use                     |
| ------------------ | ----- | ----------------------------------------- | ------------------------------- |
| (none)             | error | Errors only                               | Normal usage, production        |
| `-v` / `--verbose` | debug | All debugging info (Git, RON, transforms) | Troubleshooting, debugging      |
| `RUST_LOG=trace`   | trace | Implementation details (rarely needed)    | Deep debugging (if ever needed) |

**Note**: Debug level is sufficient for all normal debugging. Simpler than multi-level flags.

### Fine-Grained Control (RUST_LOG)

For power users who need surgical control:

```bash
# Override to specific level
RUST_LOG=debug zerv version
RUST_LOG=trace zerv version

# Module-specific logging
RUST_LOG=zerv::vcs=debug zerv version
RUST_LOG=zerv::vcs=debug,zerv::pipeline=trace zerv version

# Specific component only
RUST_LOG=zerv::vcs::git=trace zerv version

# RUST_LOG takes precedence over -v flags
RUST_LOG=trace zerv version -v    # Uses trace, not warn
```

### Logging with Piping

Logs go to stderr, so piping still works:

```bash
# Logs visible in terminal, data flows through pipe
zerv version -v --output-format zerv | zerv version -v --source stdin

# Separate logs and output to files
zerv version -v > version.txt 2> debug.log
# version.txt: version string
# debug.log: debug logs
```

```

## Migration Strategy

### Rollout Plan

1. **Phase 1** (Core Foundation): Add dependencies, create logging module, initialize in main → Test immediately
2. **Phase 2** (Critical Paths): Add logs to VCS, pipeline, schema modules → Test piping works
3. **Phase 3** (Cleanup): Replace existing debug statements → Run full test suite
4. **Phase 4** (Final Validation): Comprehensive testing, documentation updates

**Key**: Test after each phase before moving to next. Don't batch testing at the end.

### Compatibility

- **No Breaking Changes**: Purely additive feature
- **Backward Compatible**: Existing behavior unchanged (no logs by default)
- **Opt-In**: Users must use `--verbose` or `RUST_LOG` to see logs

### Rollback Plan

If logging causes issues:
1. Remove `--verbose` flag parsing
2. Comment out `logging::init_logging()` call
3. Logs become no-ops (tracing compiles to near-zero overhead when disabled)

## Future Enhancements (Out of Scope)

- ❌ Structured JSON logs (not needed for CLI)
- ❌ Log file output (stderr redirection is sufficient)
- ❌ Multiple verbosity levels (`-vv`, `-vvv`) - can add later if requested
- ❌ Custom log formats/colors - default format is sufficient
- ❌ Performance/timing traces - out of scope

## References

- **uv logging approach**: https://docs.astral.sh/uv/reference/environment/ (RUST_LOG)
- **Rust CLI Book - Output**: https://rust-cli.github.io/book/tutorial/output.html (stderr vs stdout)
- **tracing docs**: https://docs.rs/tracing/latest/tracing/
- **tracing-subscriber EnvFilter**: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html

## Estimated Timeline

- **Phase 1** (Foundation + Testing): 30 minutes
- **Phase 2** (Strategic Logs + Testing): 1 hour
- **Phase 3** (Cleanup + Testing): 30 minutes
- **Phase 4** (Final Validation + Documentation): 30 minutes

**Total**: 2.5-3 hours

**Note**: Testing integrated into each phase, not deferred to end.

## Summary of Changes from Initial Plan

### ✅ Updated to Simplest Approach (YAGNI)

1. **Default log level**: `error` only (clean output)
2. **Single verbosity flag**: `-v` = **debug** (sufficient for everything)
3. **No `-vv` support**: Not needed for Zerv's simple pipeline (no complex locking/parallelism)
4. **Simple bool flag**: `verbose: bool` (not `u8` counting)
5. **Power users**: Can still use `RUST_LOG=trace` if ever needed

### Why This Is Better

- ✅ **YAGNI**: Debug level handles everything (RON dumps, Git output, transformations)
- ✅ **Simpler UX**: Just on/off, no "should I use -v or -vv?" confusion
- ✅ **Less to maintain**: One level instead of multiple
- ✅ **Can add later**: Easy to add `-vv` if users request it (backward compatible)
- ✅ **Cleaner by default**: Error-only output unless explicitly requested

## Questions / Decisions

- ✅ **What should `-v` mean?** → **DEBUG** (sufficient for all debugging needs)
- ✅ **Should we support `-vv` (trace)?** → **NO** (YAGNI - debug handles everything, including RON dumps)
- ✅ **What if users need trace?** → **RUST_LOG=trace** (available but rarely needed)
- ✅ **What should be the default log level?** → **error** (matches env_logger standard)
- ✅ **Should we add colors to log output?** → **Use tracing defaults** (auto-detects terminal)
- ✅ **Should tests show logs by default?** → **No**, use `RUST_LOG` when debugging tests
- ✅ **Should we use `clap-verbosity-flag` crate?** → **No**, simple `bool` flag is clearer
```
