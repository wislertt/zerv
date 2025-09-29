# Pipeline Redundancy Elimination Plan

## Problem Statement

The current `git_pipeline.rs` and `stdin_pipeline.rs` files contain significant redundancy in their TODO comments and implementation patterns:

### Current Redundant Patterns

1. **Identical TODO Comments**: Both files have the same TODO about moving logic to main pipeline
2. **Duplicate Override Application**: Both apply overrides to ZervVars before creating Zerv objects
3. **Duplicate Schema Resolution**: Both resolve schema and create Zerv objects with similar logic
4. **Similar Error Handling**: Both handle the same types of errors in similar ways

### Current Architecture Issues

```rust
// git_pipeline.rs - Lines 35-47
// TODO: try to move this logic to main pipeline (unify with other sources)
vars.apply_overrides(args)?;
let (schema_name, schema_ron) = args.resolve_schema();
create_zerv_version(vars, schema_name, schema_ron)

// stdin_pipeline.rs - Lines 12-23
// TODO: try to move this logic to main pipeline (unify with other sources)
zerv_from_stdin.vars.apply_overrides(args)?;
let (schema_name, schema_ron) = args.resolve_schema();
if schema_name.is_some() || schema_ron.is_some() {
    zerv_from_stdin = crate::schema::create_zerv_version(zerv_from_stdin.vars, schema_name, schema_ron)?;
}
```

## Solution Architecture

### Phase 1: Create Intermediate ZervDraft Structure

**Goal**: Each source should return a ZervDraft structure (Zerv-like but with optional schema), then convert to full Zerv based on args.

#### 1.1 Create `ZervDraft` Structure

```rust
// src/cli/version/common.rs (new file)
#[derive(Debug, Clone)]
pub struct ZervDraft {
    pub vars: ZervVars,
    pub schema: Option<SchemaConfig>,  // Some for stdin, None for git
}

impl ZervDraft {
    pub fn new(vars: ZervVars, schema: Option<SchemaConfig>) -> Self {
        Self { vars, schema }
    }

    pub fn apply_overrides(mut self, args: &VersionArgs) -> Result<Self, ZervError> {
        self.vars.apply_overrides(args)?;
        Ok(self)
    }

    pub fn to_zerv(self, args: &VersionArgs) -> Result<Zerv, ZervError> {
        let (schema_name, schema_ron) = args.resolve_schema();
        self.create_zerv_version(schema_name, schema_ron)
    }

    pub fn create_zerv_version(self, schema_name: Option<&str>, schema_ron: Option<&str>) -> Result<Zerv, ZervError> {
        // Move the logic from crate::schema::create_zerv_version here
        let schema = match (schema_name, schema_ron) {
            // Custom RON schema
            (None, Some(ron_str)) => crate::schema::parse_ron_schema(ron_str)?,

            // Built-in schema
            (Some(name), None) => {
                if let Some(schema) = crate::schema::get_preset_schema(name, &self.vars) {
                    schema
                } else {
                    return Err(ZervError::UnknownSchema(name.to_string()));
                }
            }

            // Error cases
            (Some(_), Some(_)) => {
                return Err(ZervError::ConflictingSchemas(
                    "Cannot specify both schema_name and schema_ron".to_string(),
                ));
            }
            (None, None) => {
                // If no new schema requested, use existing schema from stdin source
                if let Some(existing_schema) = self.schema {
                    existing_schema
                } else {
                    return Err(ZervError::MissingSchema(
                        "Either schema_name or schema_ron must be provided".to_string(),
                    ));
                }
            }
        };

        Zerv::new(schema, self.vars)
    }
}
```

#### 1.2 Update Source Pipelines

**Git Pipeline** (no schema initially):

```rust
// src/cli/version/git_pipeline.rs
pub fn process_git_source(work_dir: &Path, args: &VersionArgs) -> Result<ZervDraft, ZervError> {
    // Get git VCS data
    let vcs_data = crate::vcs::detect_vcs_with_limit(work_dir, max_depth)?.get_vcs_data()?;

    // Parse git tag with input format validation
    if let Some(ref tag_version) = vcs_data.tag_version {
        let _parsed_version = InputFormatHandler::parse_version_string(tag_version, &args.input_format)?;
    }

    // Convert VCS data to ZervVars
    let vars = vcs_data_to_zerv_vars(vcs_data)?;

    // Return ZervDraft without schema (git source)
    Ok(ZervDraft::new(vars, None))
}
```

**Stdin Pipeline** (has schema initially):

```rust
// src/cli/version/stdin_pipeline.rs
pub fn process_stdin_source(args: &VersionArgs) -> Result<ZervDraft, ZervError> {
    // Parse stdin as Zerv RON (includes schema)
    let zerv_from_stdin = InputFormatHandler::parse_stdin_to_zerv()?;

    // Return ZervDraft with existing schema (stdin source)
    Ok(ZervDraft::new(zerv_from_stdin.vars, Some(zerv_from_stdin.schema)))
}
```

### Phase 2: Create Unified Pipeline Logic

#### 2.1 Update Main Pipeline

```rust
// src/cli/version/pipeline.rs
pub fn run_version_pipeline(mut args: VersionArgs) -> Result<String, ZervError> {
    // 0. Early validation
    args.validate()?;

    // 1. Determine working directory
    let work_dir = match args.directory.as_deref() {
        Some(dir) => std::path::PathBuf::from(dir),
        None => current_dir()?,
    };

    // 2. Get ZervDraft from source (no schema applied yet)
    let zerv_draft = match args.source.as_str() {
        sources::GIT => super::git_pipeline::process_git_source(&work_dir, &args)?,
        sources::STDIN => super::stdin_pipeline::process_stdin_source(&args)?,
        source => return Err(ZervError::UnknownSource(source.to_string())),
    };

    // 3. Apply overrides and convert to Zerv (unified logic)
    let zerv_object = zerv_draft
        .apply_overrides(&args)?
        .to_zerv(&args)?;

    // 4. Apply output formatting
    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.output_format,
        args.output_prefix.as_deref(),
        args.output_template.as_deref(),
    )?;

    Ok(output)
}
```

### Phase 3: Handle Schema Differences

#### 3.1 Schema Application Logic

The key difference between sources:

- **Git Source**: Always needs schema (no schema initially)
- **Stdin Source**: May have schema already, only apply new schema if requested

```rust
impl ZervDraft {
    pub fn to_zerv(self, args: &VersionArgs) -> Result<Zerv, ZervError> {
        let (schema_name, schema_ron) = args.resolve_schema();
        self.create_zerv_version(schema_name, schema_ron)
    }

    pub fn create_zerv_version(self, schema_name: Option<&str>, schema_ron: Option<&str>) -> Result<Zerv, ZervError> {
        // Move the logic from crate::schema::create_zerv_version here
        let schema = match (schema_name, schema_ron) {
            // Custom RON schema
            (None, Some(ron_str)) => crate::schema::parse_ron_schema(ron_str)?,

            // Built-in schema
            (Some(name), None) => {
                if let Some(schema) = crate::schema::get_preset_schema(name, &self.vars) {
                    schema
                } else {
                    return Err(ZervError::UnknownSchema(name.to_string()));
                }
            }

            // Error cases
            (Some(_), Some(_)) => {
                return Err(ZervError::ConflictingSchemas(
                    "Cannot specify both schema_name and schema_ron".to_string(),
                ));
            }
            (None, None) => {
                // If no new schema requested, use existing schema from stdin source
                if let Some(existing_schema) = self.schema {
                    existing_schema
                } else {
                    return Err(ZervError::MissingSchema(
                        "Either schema_name or schema_ron must be provided".to_string(),
                    ));
                }
            }
        };

        Zerv::new(schema, self.vars)
    }
}
```

## Implementation Steps

### Step 1: Create Common Structures

1. Create `src/cli/version/common.rs` with `ZervDraft`
2. Add necessary imports and dependencies
3. Move `create_zerv_version` logic from `src/schema/mod.rs` to `ZervDraft::create_zerv_version`
4. Delete the old `create_zerv_version` function from `src/schema/mod.rs`

### Step 2: Update Git Pipeline

1. Modify `process_git_source` to return `ZervDraft`
2. Remove schema application logic
3. Remove TODO comments

### Step 3: Update Stdin Pipeline

1. Modify `process_stdin_source` to return `ZervDraft` or `StdinSourceResult`
2. Remove schema application logic
3. Remove TODO comments

### Step 4: Update Main Pipeline

1. Add unified override application and schema conversion
2. Handle both source types consistently
3. Remove redundant logic from source pipelines

### Step 5: Update Tests

1. Update existing tests to work with new structure
2. Add tests for unified pipeline logic
3. Ensure all test cases still pass

### Step 6: Cleanup

1. Remove TODO comments
2. Update documentation
3. Verify no functionality is lost

## Benefits

1. **Eliminates Redundancy**: Single place for override application and schema conversion
2. **Improves Maintainability**: Changes to pipeline logic only need to be made in one place
3. **Better Separation of Concerns**: Sources focus on data extraction, main pipeline handles processing
4. **Easier Testing**: Can test override application and schema conversion independently
5. **Future-Proof**: Easy to add new sources without duplicating pipeline logic

## Migration Strategy

1. **Backward Compatibility**: Ensure existing functionality works during transition
2. **Incremental Changes**: Implement changes in phases to minimize risk
3. **Comprehensive Testing**: Run full test suite after each phase
4. **Rollback Plan**: Keep old code commented out until new code is verified

## Files to Modify

- `src/cli/version/common.rs` (new)
- `src/cli/version/git_pipeline.rs` (refactor)
- `src/cli/version/stdin_pipeline.rs` (refactor)
- `src/cli/version/pipeline.rs` (enhance)
- `src/cli/version/mod.rs` (add common module)

## Success Criteria

- [ ] All TODO comments about moving logic to main pipeline are resolved
- [ ] No duplicate override application logic
- [ ] No duplicate schema resolution logic
- [ ] All existing tests pass
- [ ] New unified pipeline logic is well-tested
- [ ] Code is more maintainable and easier to extend

## Future Cleanup Tasks

### ComponentConfig/Component Consolidation

**Issue**: `ComponentConfig` and `Component` are nearly identical with only minor structural differences:

- `ComponentConfig`: Named fields (`{ value: String }`) - better for serialization
- `Component`: Tuple variants (`String(String)`) - more concise in Rust code

**Solution**: Consolidate to single `Component` type with serde attributes:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "String")]
    String { value: String },
    #[serde(rename = "Integer")]
    Integer { value: u64 },
    // etc.
}
```

**Benefits**:

- Eliminates code duplication
- Reduces maintenance overhead
- Simplifies conversion logic
- Maintains same serialization behavior

**Files to Update**:

- `src/schema/parser.rs` - Remove `ComponentConfig` and conversion logic
- `src/version/zerv/schema.rs` - Update `Component` with serde attributes
- All test files using `ComponentConfig`
