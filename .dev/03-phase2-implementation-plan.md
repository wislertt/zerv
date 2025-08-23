# Phase 2 Implementation Plan: Pipeline CLI Interface

## Goal

Implement `zerv version --source git` with pipeline architecture connecting VCS data to version generation.

## Implementation Scope

### Core Requirements

1. **VCS-Version Integration**: Connect `VcsData` to `ZervVars`
2. **CLI Pipeline**: Implement `zerv version` command
3. **CLI Validation**: Implement `zerv check <version>` command
4. **Schema System**: Add `zerv-default` preset and RON parsing
5. **Tag Parsing**: Extract version from Git tags

### Success Criteria

- `zerv version` generates versions from Git repository state
- `zerv check <version>` validates version strings
- `--source git` works (default behavior)
- `--schema zerv-default` provides tier-aware schema
- `--output-format pep440|semver` works
- Integration tests pass for end-to-end workflow

## Implementation Tasks

### Task 1: VCS Data Integration (`src/pipeline/`)

**Create**: `src/pipeline/mod.rs`, `src/pipeline/vcs_integration.rs`

**Functions**:

```rust
// Convert VcsData to ZervVars
fn vcs_data_to_zerv_vars(vcs_data: VcsData, base_version: Option<String>) -> ZervVars

// Parse version from tag (e.g., "v1.2.3" -> "1.2.3")
fn parse_version_from_tag(tag: &str) -> Option<String>

// Determine tier based on VCS state
fn determine_version_tier(vcs_data: &VcsData) -> VersionTier
```

**VersionTier**:

```rust
enum VersionTier {
    Tagged,    // On tag, clean
    Distance,  // Distance from tag, clean
    Dirty,     // Any dirty state
}
```

### Task 2: Schema System (`src/schema/`)

**Create**: `src/schema/mod.rs`, `src/schema/presets.rs`

**Functions**:

```rust
// Parse RON schema string to ZervFormat
fn parse_ron_schema(ron_str: &str) -> Result<ZervFormat>

// Get zerv-default preset schema
fn get_default_schema() -> ZervFormat

// Populate ZervVars based on tier and schema
fn populate_vars_by_tier(vars: &mut ZervVars, tier: VersionTier, schema: &ZervFormat)
```

**Default Schema**:

```rust
// Tier 1: major.minor.patch
// Tier 2: major.minor.patch.post<distance>+branch.<commit>
// Tier 3: major.minor.patch.dev<timestamp>+branch.<commit>
```

### Task 3: Version Command (`src/cli/`)

**Update**: `src/cli/app.rs`, `src/cli/commands.rs`

**Version Command Args**:

```rust
#[derive(Parser)]
struct VersionArgs {
    /// Version string (only used with --source string)
    version: Option<String>,

    #[arg(long, default_value = "git")]
    source: String,

    #[arg(long, default_value = "zerv-default")]
    schema: String,

    #[arg(long)]
    schema_ron: Option<String>,

    #[arg(long)]
    format: Option<String>,         // shorthand: input=output format

    #[arg(long)]
    input_format: Option<String>,   // auto-detect if not provided

    #[arg(long)]
    output_format: Option<String>,  // default to zerv format
}
```

**Format Flag Validation**:

```rust
// Error if --format used with --input-format or --output-format
if args.format.is_some() && (args.input_format.is_some() || args.output_format.is_some()) {
    return Err(ZervError::ConflictingFlags(
        "Cannot use --format with --input-format or --output-format".to_string()
    ));
}

// Resolve actual formats
let input_fmt = args.input_format.or(args.format.clone());
let output_fmt = args.output_format.or(args.format);
```

**Version Pipeline Function**:

```rust
fn run_version_pipeline(args: VersionArgs) -> Result<String>
```

**Usage Examples**:

```bash
# Basic version generation from Git
# (assuming latest tag: v1.2.3, 5 commits ahead, on main branch)
zerv version
# Output: 1.2.3.post5+main.abc123

# Single format (shorthand - most common)
zerv version --format pep440
# Output: 1.2.3.post5+main.abc123

zerv version --format semver
# Output: 1.2.3-post.5+main.abc123

# Explicit output format only
zerv version --output-format pep440
# Output: 1.2.3.post5+main.abc123

zerv version --output-format semver
# Output: 1.2.3-post.5+main.abc123

# Convert version formats (string source)
zerv version --source string "1.2.3a1" --output-format semver
# Auto-detects input as PEP440, converts to SemVer
# Output: 1.2.3-alpha.1

# Explicit input format (when ambiguous)
zerv version --source string "1.2.3" --input-format pep440 --output-format semver
# Output: 1.2.3

# Error case - conflicting flags
zerv version --format pep440 --output-format semver
# Error: Cannot use --format with --input-format or --output-format
```

### Task 4: Check Command (`src/cli/`)

**Check Command Args**:

```rust
#[derive(Parser)]
struct CheckArgs {
    /// Version string to validate
    version: String,

    #[arg(long)]
    format: Option<String>,  // pep440, semver, auto-detect (default)
}
```

**Check Command Function**:

```rust
fn run_check_command(args: CheckArgs) -> Result<()>
```

**Implementation with Auto-Detection**:

```rust
fn run_check_command(args: CheckArgs) -> Result<()> {
    match args.format.as_deref() {
        Some("pep440") => {
            PEP440::parse(&args.version)?;
            println!("✓ Valid PEP440 version");
        }
        Some("semver") => {
            SemVer::parse(&args.version)?;
            println!("✓ Valid SemVer version");
        }
        None => {
            // Auto-detect format
            let pep440_valid = PEP440::parse(&args.version).is_ok();
            let semver_valid = SemVer::parse(&args.version).is_ok();

            match (pep440_valid, semver_valid) {
                (true, false) => println!("✓ Valid PEP440 version"),
                (false, true) => println!("✓ Valid SemVer version"),
                (true, true) => {
                    println!("✓ Valid PEP440 version");
                    println!("✓ Valid SemVer version");
                }
                (false, false) => return Err(ZervError::InvalidVersion(args.version)),
            }
        }
        Some(format) => return Err(ZervError::UnknownFormat(format.to_string())),
    }
    Ok(())
}
```

**Usage Examples**:

```bash
# Auto-detect format (most common)
zerv check "1.2.3"
# Output: ✓ Valid PEP440 version
#         ✓ Valid SemVer version

zerv check "1.2.3a1"
# Output: ✓ Valid PEP440 version

zerv check "1.2.3-alpha.1"
# Output: ✓ Valid SemVer version

# Explicit format validation
zerv check "1.2.3" --format pep440
# Output: ✓ Valid PEP440 version

zerv check "1.2.3" --format semver
# Output: ✓ Valid SemVer version

# Invalid version
zerv check "invalid"
# Output: Error: Invalid version: invalid
# Exit code: 1
```

### Task 5: Integration Tests

**Create**: `tests/integration/version_command.rs`

**Test Cases**:

**Version Command**:

- `zerv version` → `1.2.3.post5+main.abc123`
- `zerv version --output-format pep440` → `1.2.3.post5+main.abc123`
- `zerv version --output-format semver` → `1.2.3-post.5+main.abc123`
- `zerv version --source string "1.2.3a1" --output-format semver` → `1.2.3-alpha.1`
- Custom RON schema parsing
- Error cases (no git repo, invalid schema)

**Check Command**:

- `zerv check "1.2.3"` → `✓ Valid version (both PEP440 and SemVer): 1.2.3`
- `zerv check "1.2.3a1"` → `✓ Valid PEP440 version: 1.2.3a1`
- `zerv check "1.2.3-alpha.1"` → `✓ Valid SemVer version: 1.2.3-alpha.1`
- `zerv check "1.2.3" --format pep440` → `✓ Valid PEP440 version: 1.2.3`
- `zerv check "invalid"` → `Error: Invalid version: invalid` (exit 1)
- Error cases (unknown format)

## File Structure

```
src/
├── pipeline/
│   ├── mod.rs              # Pipeline orchestration
│   └── vcs_integration.rs  # VCS data conversion
├── schema/
│   ├── mod.rs              # Schema parsing
│   └── presets.rs          # Built-in schemas
├── cli/
│   ├── mod.rs              # Re-exports
│   ├── app.rs              # Clap app definition
│   └── commands.rs         # Command implementations
└── lib.rs                  # Add new modules

tests/integration/
└── version_command.rs      # End-to-end tests
```

## Implementation Order

### Step 1: VCS Integration ✅ COMPLETE (3 days)

**Key Achievements:**

- **Type-safe pipeline functions**: `parse_version_from_tag` and `vcs_data_to_zerv_vars` with clean separation of concerns
- **Auto-detection logic**: SemVer-first parsing with PEP440 fallback for maximum compatibility
- **Real VCS data testing**: Docker-based fixtures with `OnceLock` session-like caching for edge case discovery
- **Reusable test infrastructure**: VCS fixtures moved to `test_utils` for cross-module usage
- **Comprehensive test coverage**: 38 parameterized test cases covering edge cases and unnormalized forms
- **Production ready**: All linting issues resolved, 1131 tests passing, 97.38% coverage maintained

**Files Created/Modified:**

- `src/pipeline/mod.rs` - Module orchestration with re-exports
- `src/pipeline/parse_version_from_tag.rs` - Version parsing with auto-detection
- `src/pipeline/vcs_data_to_zerv_vars.rs` - VCS to ZervVars conversion
- `src/test_utils/vcs_fixtures.rs` - Reusable real VCS data fixtures
- `src/version/version_object.rs` - Enhanced with unified format handling

1. ✅ Create `src/pipeline/` module with focused functions
2. ✅ Implement `parse_version_from_tag` - extracts base version from tag string with auto-detection (SemVer first, then PEP440)
3. ✅ Implement `vcs_data_to_zerv_vars` - converts VcsData to ZervVars using elegant `VersionObject::into()` pattern
4. ✅ Comprehensive unit tests with real VCS data fixtures using `OnceLock` session-like caching
5. ✅ Moved VCS fixtures to `test_utils` for reusability across codebase
6. ✅ All 1131 tests pass with 97.38% coverage maintained

### Step 2: Schema System (1-2 days) 🔄 NEXT

1. Create `src/schema/` module
2. Implement RON parsing
3. Add `zerv-default` preset
4. Unit tests for schema parsing

### Step 3: CLI Pipeline (1-2 days)

1. Update `src/cli/app.rs` with new args
2. Implement `run_version_pipeline`
3. Connect VCS → Schema → Output
4. Basic error handling

### Step 4: Check Command (0.5 days)

1. Implement `run_check_command`
2. Add format auto-detection
3. Unit tests for validation

### Step 5: Integration Testing (1 day)

1. End-to-end tests for both commands
2. Error case validation
3. Output format verification

## Minimal Implementation

**Core Pipeline**:

```rust
// src/pipeline/mod.rs
pub fn run_version_pipeline(args: VersionArgs) -> Result<String> {
    // 1. Get VCS data
    let vcs = detect_vcs(&std::env::current_dir()?)?;
    let vcs_data = vcs.get_vcs_data()?;

    // 2. Resolve version source (VCS tag or CLI string)
    let vcs_data = resolve_version_source(vcs_data, args.version)?;

    // 3. Convert to ZervVars
    let vars = vcs_data_to_zerv_vars(vcs_data)?;

    // 4. Apply schema
    let schema = get_schema(&args.schema, &args.schema_ron)?;
    let zerv = Zerv::new(schema, vars);

    // 5. Output format
    match args.output_format.as_deref() {
        Some("pep440") => Ok(PEP440::from_zerv(&zerv)?.to_string()),
        Some("semver") => Ok(SemVer::from_zerv(&zerv)?.to_string()),
        _ => Ok(zerv.to_string()),
    }
}
```

## Dependencies

**Add to Cargo.toml**:

```toml
[dependencies]
ron = "0.8"  # RON schema parsing
```

## Testing Strategy

**Unit Tests**:

- VCS data conversion functions
- Schema parsing and validation
- Tag parsing edge cases

**Integration Tests**:

- Full pipeline with real Git repos
- Docker-based testing for isolation
- Error handling validation

**Success Metrics**:

- All tests pass
- `make test` includes new integration tests
- Code coverage maintained >95%

## Risk Mitigation

**Potential Issues**:

1. **RON parsing complexity** → Start with simple schemas, expand gradually
2. **VCS data mapping** → Use existing `ZervVars` structure, minimal changes
3. **CLI argument conflicts** → Clear validation with helpful error messages

**Fallback Strategy**:

- Implement basic Git source first
- Add schema system incrementally
- Defer complex features to Phase 3

## CLI Demo Examples

**Typical Workflow**:

```bash
# Check current repository version
$ zerv version
1.2.3.post5+main.abc123

# Generate PEP440 version for Python package
$ zerv version --output-format pep440
1.2.3.post5+main.abc123

# Generate SemVer version for Node.js package
$ zerv version --output-format semver
1.2.3-post.5+main.abc123

# Validate version strings
$ zerv check "1.2.3a1"
✓ Valid PEP440 version: 1.2.3a1

$ zerv check "1.2.3-alpha.1"
✓ Valid SemVer version: 1.2.3-alpha.1

$ zerv check "invalid"
Error: Invalid version: invalid
```

**Advanced Usage**:

```bash
# Convert between formats
$ zerv version --source string "1.2.3a1" --output-format semver
1.2.3-alpha.1

# Custom schema (CalVer example)
$ zerv version --schema-ron '(core: [VarTimestamp("YYYY"), VarTimestamp("MM"), VarField("patch")], extra_core: [], build: [])'
2024.01.5

# Explicit format validation
$ zerv check "1.2.3" --format pep440
✓ Valid PEP440 version: 1.2.3
```

## Timeline Estimate

- **Total**: 5-7 days focused development
- **Milestone 1**: VCS integration working ✅ COMPLETE (Day 3)
- **Milestone 2**: Basic CLI pipeline + check command (Day 5)
- **Milestone 3**: Full Phase 2 complete (Day 7)

## Progress Summary

**✅ COMPLETED (Day 3):**

- Step 1: VCS Integration with comprehensive testing and real data fixtures
- Enhanced type safety with `VersionObject` enum and unified format handling
- Production-ready code with full lint compliance and test coverage

**🔄 NEXT STEPS:**

- Step 2: Schema System (RON parsing, presets)
- Step 3: CLI Pipeline (version command implementation)
- Step 4: Check Command (validation with auto-detection)
- Step 5: Integration Tests (end-to-end validation)

**Note**: `zerv check` adds minimal complexity since we already have complete PEP440/SemVer parsers.

This plan leverages the solid Phase 1 foundation to rapidly implement the core pipeline functionality needed for a functional alpha release.
