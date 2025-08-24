# Implementation Plan - Phase 2: CLI Pipeline

## Current Status

✅ **Step 1 COMPLETE**: VCS Integration (3 days)

- `parse_version_from_tag` with auto-detection
- `vcs_data_to_zerv_vars` conversion
- Real VCS data fixtures with comprehensive testing
- 1131 tests passing, 97.38% coverage maintained

## Next Steps

### Step 2: Schema System (1-2 days) 🔄 NEXT

**Goal**: RON schema parsing and `zerv-default` preset

**Tasks**:

1. Create `src/schema/` module
2. Implement RON parsing for `ZervFormat`
3. Add `zerv-default` preset with tier-aware logic
4. Implement `create_zerv_version` function - Takes `ZervVars` + schema and produces `Zerv` object
5. Unit tests for schema parsing and version creation

**Files**:

- `src/schema/mod.rs` - Schema parsing and `create_zerv_version` function
- `src/schema/presets.rs` - Built-in schemas

### Step 3: CLI Pipeline (1-2 days)

**Goal**: `zerv version` command implementation

**Tasks**:

1. Update `src/cli/app.rs` with new args
2. Implement `run_version_pipeline` function
3. Connect VCS → Schema → Output pipeline
4. Add format validation and error handling

**Core Pipeline**:

```rust
pub fn run_version_pipeline(args: VersionArgs) -> Result<String> {
    // 1. Get VCS data
    let vcs_data = detect_vcs(current_dir())?.get_vcs_data()?;

    // 2. Convert to ZervVars
    let vars = vcs_data_to_zerv_vars(vcs_data)?;

    // 3. Create Zerv version object from vars and schema
    let zerv = create_zerv_version(vars, &args.schema, args.schema_ron.as_deref())?;

    // 4. Apply output format
    match args.output_format.as_deref() {
        Some("pep440") => Ok(PEP440::from(zerv).to_string()),
        Some("semver") => Ok(SemVer::from(zerv).to_string()),
        _ => Ok(zerv.to_string()),
    }
}
```

### Step 4: Check Command (0.5 days)

**Goal**: `zerv check <version>` validation

**Tasks**:

1. Implement `run_check_command` with auto-detection
2. Add format-specific validation
3. Unit tests for validation logic

### Step 5: Integration Testing (1 day)

**Goal**: End-to-end testing

**Tasks**:

1. Create `tests/integration/version_command.rs`
2. Test full pipeline with real Git repos
3. Error case validation
4. Output format verification

## Success Criteria

- `zerv version` generates versions from Git repository state
- `zerv check <version>` validates version strings
- `--output-format pep440|semver` works correctly
- Integration tests pass for end-to-end workflow
- All existing tests continue to pass

## Timeline

- **Total**: 5-7 days focused development
- **Milestone 1**: VCS integration ✅ COMPLETE (Day 3)
- **Milestone 2**: Basic CLI pipeline + check command (Day 5)
- **Milestone 3**: Full Phase 2 complete (Day 7)

## Dependencies

```toml
[dependencies]
ron = "0.8"  # RON schema parsing
```

## CLI Implementation Details

### Version Command Args

```rust
#[derive(Parser)]
struct VersionArgs {
    version: Option<String>,
    #[arg(long, default_value = "git")]
    source: String,
    #[arg(long, default_value = "zerv-default")]
    schema: String,
    #[arg(long)]
    schema_ron: Option<String>,
    #[arg(long)]
    output_format: Option<String>,
}
```

### Check Command Args

```rust
#[derive(Parser)]
struct CheckArgs {
    version: String,
    #[arg(long)]
    format: Option<String>,  // pep440, semver, auto-detect (default)
}
```

### Format Flag Validation

```rust
// Error if --format used with --input-format or --output-format
if args.format.is_some() && (args.input_format.is_some() || args.output_format.is_some()) {
    return Err(ZervError::ConflictingFlags(
        "Cannot use --format with --input-format or --output-format".to_string()
    ));
}
```

## Demo Examples

```bash
# Basic version generation
zerv version
# Output: 1.2.3.post5+main.abc123

# Format-specific output
zerv version --output-format pep440
# Output: 1.2.3.post5+main.abc123

zerv version --output-format semver
# Output: 1.2.3-post.5+main.abc123

# Version validation with auto-detection
zerv check "1.2.3"
# Output: ✓ Valid PEP440 version
#         ✓ Valid SemVer version

zerv check "1.2.3a1"
# Output: ✓ Valid PEP440 version

# Format conversion
zerv version --source string "1.2.3a1" --output-format semver
# Output: 1.2.3-alpha.1
```
