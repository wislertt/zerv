# Implementation Plan - Phase 2: CLI Pipeline

## Current Status

✅ **Step 1 COMPLETE**: VCS Integration (3 days)

- `parse_version_from_tag` with auto-detection
- `vcs_data_to_zerv_vars` conversion
- Real VCS data fixtures with comprehensive testing
- 1131 tests passing, 97.38% coverage maintained

✅ **Step 2 COMPLETE**: Schema System (1-2 days)

- `src/schema/` module with RON parsing
- `create_zerv_version` function implemented
- `zerv-standard` and `zerv-calver` presets with tier-aware logic
- 29 comprehensive unit tests added
- 1198 tests passing, schema system fully functional

✅ **Step 3 COMPLETE**: CLI Pipeline (1-2 days)

- `zerv version` command implemented with full pipeline
- `zerv check <version>` command with auto-detection
- `--output-format pep440|semver` working correctly
- CLI args structure with clap subcommands
- 12 new CLI tests + updated integration tests
- 1206 tests passing, CLI fully functional

✅ **Step 4 COMPLETE**: Check Command (0.5 days)

- `zerv check <version>` validation implemented
- Auto-detection of PEP440/SemVer formats
- Format-specific validation with `--format` flag
- Comprehensive error handling and user feedback
- Integrated as part of Step 3 CLI implementation

## Next Steps

### Step 5: Integration Testing (1 day) 🔄 NEXT

**Goal**: End-to-end testing

**Tasks**:

1. ✅ Integration tests updated for new CLI structure
2. ✅ Test full pipeline with real Git repos
3. ✅ Error case validation
4. ✅ Output format verification

**Status**: Most integration testing already complete. Additional comprehensive testing may be added if needed.

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
ron = "0.11.0"  # RON schema parsing ✅ ADDED
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
