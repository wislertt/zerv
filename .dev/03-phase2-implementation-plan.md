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

### Step 1: VCS Integration (2-3 days)

1. Create `src/pipeline/vcs_integration.rs`
2. Implement `vcs_data_to_zerv_vars`
3. Add tag parsing logic
4. Unit tests for conversion

### Step 2: Schema System (1-2 days)

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

    // 2. Convert to ZervVars
    let base_version = parse_version_from_tag(&vcs_data.tag_version);
    let mut vars = vcs_data_to_zerv_vars(vcs_data, base_version);

    // 3. Apply schema
    let schema = get_schema(&args.schema, &args.schema_ron)?;
    let zerv = Zerv::new(schema, vars);

    // 4. Output format
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
- **Milestone 1**: VCS integration working (Day 3)
- **Milestone 2**: Basic CLI pipeline + check command (Day 5)
- **Milestone 3**: Full Phase 2 complete (Day 7)

**Note**: `zerv check` adds minimal complexity since we already have complete PEP440/SemVer parsers.

This plan leverages the solid Phase 1 foundation to rapidly implement the core pipeline functionality needed for a functional alpha release.
