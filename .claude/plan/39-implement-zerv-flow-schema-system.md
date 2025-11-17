# Implement Zerv Flow Schema System

**Status**: Planned
**Priority**: High
**Context**: Refactor zerv flow CLI arguments to use flexible schema system, consolidating multiple context/control flags into a single `--schema` argument that leverages the existing schema system from zerv version.

## Current State Analysis

### Existing Flow Arguments (src/cli/flow/args/main.rs)

```rust
// Currently implemented context flags to be replaced
--dev-ts / --no-dev-ts              // Controls dev timestamp inclusion ✅ IMPLEMENTED

// Context flags from design doc that were never implemented
--build-context / --no-build-context // Controls build context inclusion ❌ NOT IMPLEMENTED
--with-pre-release                  // Include prerelease/post only ❌ NOT IMPLEMENTED
--base-only                         // Base version only ❌ NOT IMPLEMENTED

// Context flags to be replaced (that are actually implemented)
--no-pre-release                    // Disable pre-release entirely ✅ IMPLEMENTED

// Existing arguments to keep
--pre-release-label <LABEL>         // Pre-release label (alpha, beta, rc)
--pre-release-num <NUM>             // Pre-release number
--hash-branch-len <LEN>             // Hash length for bumped branch
--post-mode <MODE>                  // Post calculation mode (commit, tag)
```

### Implementation Scope

**Current State**: Three context flags are implemented in the codebase:

- `--dev-ts`/`--no-dev-ts` (controls dev timestamp inclusion)
- `--no-pre-release` (disables pre-release entirely)

**Goal**: Replace all implemented context control logic with schema system and implement the missing context control through schemas instead of individual flags.

### Target Schema Integration

- **Leverage**: Existing `ZervSchemaPreset` system from `src/schema/presets.rs`
- **Restrict**: Only standard schema family (no calver variants)
- **Validate**: Schema selection and provide clear error messages

## Implementation Plan

### Phase 1: Update Flow Arguments Structure

#### Step 1: Remove Implemented Context Flags

**Target**: `src/cli/flow/args/main.rs:100-118`

**Actions:**

- Remove `no_pre_release: bool` field and argument (lines 101-103)
- Remove `dev_ts: bool` field and argument (lines 111-113)
- Remove `no_dev_ts: bool` field and argument (lines 116-118)
- Remove related clap argument definitions
- Update help text and examples
- Update default implementation (lines 129, 131-132)

**Note**: Other context flags (`--build-context`, `--with-pre-release`, `--base-only`) were never implemented and don't need removal.

**Schema Replacement**: `--no-pre-release` functionality will be replaced by `--schema standard-base`.

#### Step 2: Add Schema Argument

**Target**: `src/cli/flow/args/main.rs:73-119`

**Actions:**

- Add `schema: Option<String>` field to `FlowArgs`
- Add clap argument definition with validation for standard schemas only
- Update help text to reflect new schema system
- Update command examples in help text

#### Step 3: Update Argument Help Documentation

**Target**: `src/cli/flow/args/main.rs:17-71`

**Actions:**

- Remove references to `--dev-ts`, `--no-dev-ts` in help text
- Add schema system documentation
- Update examples to use `--schema` instead of individual flags
- Document schema validation behavior

### Phase 2: Update Validation Logic

#### Step 4: Add Schema Validation Function

**Target**: `src/cli/flow/args/main.rs:149-166`

**Actions:**

- Add `validate_schema()` method to `FlowArgs`
- Implement standard schema restriction logic
- Add clear error messages for invalid schemas
- Use existing `ZervSchemaPreset::from_str()` for parsing

#### Step 5: Remove Context Flag Validation Methods

**Target**: `src/cli/flow/args/main.rs:157-166`

**Actions:**

- Remove `validate_dev_ts()` method (lines 209-211)
- Remove `validate_pre_release_label()` method's no_pre_release logic (lines 168-177)
- Remove `validate_pre_release_num()` method's no_pre_release logic (lines 179-186)
- Remove opposing flag resolution logic for dev_ts (lines 160-164)
- Update main validation to call `validate_schema()`

#### Step 6: Update Default Implementation

**Target**: `src/cli/flow/args/main.rs:121-135`

**Actions:**

- Remove `dev_ts` and `no_dev_ts` from defaults
- Set default schema to `None` (will use `standard` in pipeline)

### Phase 3: Update Bump Configuration Generation

#### Step 7: Remove Context-Dependent Bump Logic

**Target**: `src/cli/flow/args/main.rs:213-266`

**Actions:**

- Remove `bump_dev()` method entirely (lines 258-266)
- Update `bump_pre_release_label()` to remove no_pre_release logic (lines 213-218)
- Update `bump_pre_release_num()` to remove no_pre_release logic (lines 220-241)
- Schema system now controls component inclusion

#### Step 8: Update Template Helper Functions

**Target**: `src/cli/flow/args/main.rs:138-147`

**Actions:**

- Review and possibly simplify template building logic
- Ensure compatibility with schema-based component control

### Phase 4: Update Pipeline Integration

#### Step 9: Pass Schema to Version Pipeline

**Target**: `src/cli/flow/pipeline.rs:20-38`

**Actions:**

- Add schema field to `VersionArgs` construction
- Pass `args.schema` to version pipeline
- Remove explicit dev bumping configuration

#### Step 10: Test Pipeline Integration

**Target**: `src/cli/flow/pipeline.rs`

**Actions:**

- Ensure pipeline correctly uses schema from flow args
- Verify output format conversion still works
- Test error propagation from schema validation

### Phase 5: Update Tests

#### Step 11: Update Unit Tests for Arguments

**Target**: `src/cli/flow/args/main.rs:269-841`

**Actions:**

- Remove tests for deprecated flags (`dev_ts`, `no_dev_ts`)
- Add tests for schema validation (valid/invalid schemas)
- Update default value tests
- Update validation conflict tests
- Add integration tests for schema + manual overrides

#### Step 12: Update Pipeline Tests

**Target**: `src/cli/flow/pipeline.rs:52-145`

**Actions:**

- Update existing test expectations to use schema-based behavior
- Add tests for different schema variants
- Add tests for schema validation errors
- Update fixture-based tests to match new schema outputs

### Phase 6: Update CLI Documentation

#### Step 13: Update Help Text and Examples

**Target**: `src/cli/flow/args/main.rs:17-71`

**Actions:**

- Update command help text to reflect schema system
- Update examples section with schema usage
- Document error cases and validation behavior

#### Step 14: Integration with Existing Documentation

**Target**: External documentation files

**Actions:**

- Update any external CLI documentation
- Ensure consistency with design document (#33)
- Update man pages if they exist

## Implementation Details

### Schema Validation Logic

```rust
fn validate_schema(&self) -> Result<(), ZervError> {
    if let Some(schema_name) = &self.schema {
        // First, validate it's a known schema
        let _parsed = ZervSchemaPreset::from_str(schema_name)
            .map_err(|_| ZervError::InvalidArgument(
                format!("Unknown schema variant: '{}'", schema_name)
            ))?;

        // Restrict to standard schemas only (check prefix)
        if schema_name.starts_with("standard") {
            Ok(())
        } else {
            Err(ZervError::InvalidArgument(
                format!("zerv flow only supports standard schema variants, got: '{}'", schema_name)
            ))
        }
    } else {
        Ok(())
    }
}
```

### Updated FlowArgs Structure

```rust
pub struct FlowArgs {
    #[command(flatten)]
    pub input: InputConfig,

    #[command(flatten)]
    pub output: OutputConfig,

    // NEW: Schema argument replaces multiple context flags
    #[arg(long, help = "Schema variant for output components [default: standard] [possible values: standard, standard-no-context, standard-context, standard-base, standard-base-prerelease, standard-base-prerelease-post, standard-base-prerelease-post-dev]")]
    pub schema: Option<String>,

    // Existing arguments to keep
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(pre_release_labels::VALID_LABELS))]
    pub pre_release_label: Option<String>,

    #[arg(long, value_parser = clap::value_parser!(u32))]
    pub pre_release_num: Option<u32>,

    #[arg(long = "hash-branch-len", value_parser = clap::value_parser!(u32), default_value = "5")]
    pub hash_branch_len: u32,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub no_pre_release: bool,

    #[arg(long = "post-mode", value_parser = clap::builder::PossibleValuesParser::new(post_modes::VALID_MODES), default_value = post_modes::COMMIT)]
    pub post_mode: String,

    // REMOVED: dev_ts, no_dev_ts, and no_pre_release fields
}
```

### Pipeline Integration Changes

```rust
// In run_flow_pipeline()
let version_args = VersionArgs {
    input: args.input.clone(),
    output: OutputConfig {
        output_format: "zerv".to_string(),
        output_template: None,
        output_prefix: None,
    },
    main: MainConfig {
        schema: args.schema,  // NEW: Pass schema through
        schema_ron: None,
    },
    overrides: Default::default(),
    bumps: BumpsConfig {
        bump_pre_release_label: args.bump_pre_release_label(),
        bump_pre_release_num: args.bump_pre_release_num(),
        bump_patch: args.bump_patch(),
        bump_post: args.bump_post(),
        // REMOVED: bump_dev: args.bump_dev(), // Now controlled by schema
        // REMOVED: no_pre_release logic in bump methods // Now controlled by schema
        ..Default::default()
    },
};
```

## Migration Guide

### Before (Implemented flags that will be removed)

```bash
# Include dev timestamp
zerv flow --dev-ts

# Exclude dev timestamp
zerv flow --no-dev-ts

# Disable pre-release entirely
zerv flow --no-pre-release
```

### After (Schema system)

```bash
# Default smart context (equivalent to old --dev-ts behavior)
zerv flow
zerv flow --schema standard

# Never include context (equivalent to old --no-dev-ts + --no-build-context)
zerv flow --schema standard-no-context

# Always include context
zerv flow --schema standard-context

# Base only (replaces --no-pre-release)
zerv flow --schema standard-base

# Pre-release + post (no context)
zerv flow --schema standard-base-prerelease-post
```

## Testing Strategy

### Unit Tests

- Schema validation with valid/invalid inputs
- Default behavior when no schema specified
- Conflict detection between schema and manual overrides
- Template generation compatibility

### Integration Tests

- End-to-end pipeline with different schemas
- Error propagation from schema validation
- Output format consistency
- Repository state behavior (clean/dirty/distance)

### Regression Tests

- Ensure existing functionality not broken
- Compare old flag behavior with equivalent schema variants
- Performance impact assessment

## Success Criteria

1. ✅ **Schema argument successfully replaces all context flags**
2. ✅ **Only standard schema variants accepted**
3. ✅ **Clear error messages for invalid schemas**
4. ✅ **All existing tests updated and passing**
5. ✅ **New tests cover schema validation and pipeline integration**
6. ✅ **Backward compatibility through clear migration path**
7. ✅ **Documentation updated with new examples**
8. ✅ **No performance regression**

## Risk Mitigation

### Breaking Changes

- **Risk**: Users relying on old flags will have breaking changes
- **Mitigation**: Clear error messages guide users to equivalent schema options

### Complexity

- **Risk**: Schema system adds complexity compared to simple flags
- **Mitigation**: Smart defaults and comprehensive documentation

### Testing Coverage

- **Risk**: Missing edge cases in schema validation
- **Mitigation**: Comprehensive test suite with both positive and negative cases

---

**Next Steps**: Begin Phase 1 implementation by updating the FlowArgs structure and removing deprecated flags.
