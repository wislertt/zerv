# Schema System Implementation Plan

## ✅ STATUS: COMPLETED

**All implementation steps have been successfully completed. The schema system is fully functional with 33 passing tests.**

## Overview

Implemented a complete schema system for zerv that supports:

1. **RON Schema Parsing**: Parse custom schemas from RON (Rust Object Notation) strings ✅
2. **Preset Schemas**: Built-in schemas for common versioning patterns ✅
3. **Tier-Aware Logic**: Different schema components based on Git state (tagged/distance/dirty) ✅
4. **Core Integration**: Bridge `ZervVars` to `Zerv` objects via `create_zerv_version` function ✅

## Architecture

```
ZervVars → create_zerv_version() → Zerv
                ↓
    Schema Selection Logic:
    - schema_name → Preset Schema
    - schema_ron → Custom RON Schema
    - Neither → Default (zerv-standard)
```

## ✅ COMPLETED IMPLEMENTATION

### 1. RON Schema Parser (`src/schema/parser.rs`)

**Features:**

- `SchemaConfig` struct for RON deserialization
- `ComponentConfig` enum with tagged union support
- Automatic conversion to `ZervSchema` via `From` traits
- Comprehensive error handling with `ZervError::SchemaParseError`

```rust
#[derive(Debug, Deserialize)]
pub struct SchemaConfig {
    pub core: Vec<ComponentConfig>,
    pub extra_core: Vec<ComponentConfig>,
    pub build: Vec<ComponentConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ComponentConfig {
    String { value: String },
    Integer { value: u64 },
    VarField { field: String },
    VarTimestamp { pattern: String },
}

pub fn parse_ron_schema(ron_str: &str) -> Result<ZervSchema, ZervError>
```

### 2. Preset Schemas

#### Standard Schema (`src/schema/presets/standard.rs`)

**State-Based Versioning Tiers:**

- **Tier 1** (Tagged, clean): `major.minor.patch`
- **Tier 2** (Distance, clean): `major.minor.patch.post<distance>+branch.<commit>`
- **Tier 3** (Dirty): `major.minor.patch.dev<timestamp>+branch.<distance>.<commit>`

**Functions:**

- `zerv_standard_tier_1()` → Basic semantic version
- `zerv_standard_tier_2()` → Adds post-release and build metadata
- `zerv_standard_tier_3()` → Adds development identifier
- `get_standard_schema(vars)` → Tier-aware selection

#### CalVer Schema (`src/schema/presets/calver.rs`)

**Calendar-Based Versioning Tiers:**

- **Tier 1** (Tagged, clean): `YYYY.MM.DD.patch`
- **Tier 2** (Distance, clean): `YYYY.MM.DD.patch.post<distance>+branch.<commit>`
- **Tier 3** (Dirty): `YYYY.MM.DD.patch.dev<timestamp>+branch.<distance>.<commit>`

**Functions:**

- `zerv_calver_tier_1()` → Calendar version with patch
- `zerv_calver_tier_2()` → Adds post-release and build metadata
- `zerv_calver_tier_3()` → Adds development identifier
- `get_calver_schema(vars)` → Tier-aware selection

#### Preset Dispatcher (`src/schema/presets/mod.rs`)

```rust
pub fn get_preset_schema(name: &str, vars: &ZervVars) -> Option<ZervSchema> {
    match name {
        "zerv-standard" => Some(get_standard_schema(vars)),
        "zerv-calver" => Some(get_calver_schema(vars)),
        _ => None,
    }
}
```

### 3. Main Schema Module (`src/schema/mod.rs`)

**Core Function:**

```rust
pub fn create_zerv_version(
    vars: ZervVars,
    schema_name: Option<&str>,
    schema_ron: Option<&str>,
) -> Result<Zerv, ZervError>
```

**Logic:**

- Validates mutually exclusive schema parameters
- Handles custom RON schemas via `parse_ron_schema`
- Resolves preset schemas via `get_preset_schema`
- Defaults to "zerv-standard" when no schema specified
- Returns `Zerv { schema, vars }` object

### 4. Error Handling (`src/error.rs`)

**Added Error Variants:**

```rust
SchemaParseError(String),    // RON parsing failures
UnknownSchema(String),       // Invalid preset names
ConflictingSchemas(String),  // Both schema_name and schema_ron provided
```

**Standards Compliance:**

- Uses `ZervError` for all custom errors
- Implements `Display`, `Error`, and `PartialEq` traits
- Follows project error handling patterns

### 5. Integration (`src/lib.rs`)

```rust
pub mod schema;
```

### 6. Dependencies (`Cargo.toml`)

```toml
[dependencies]
ron = "^0.8"                                    # RON parsing
serde = { version = "^1.0", features = ["derive"] }  # Serialization

[dev-dependencies]
rstest = "^0.26.0"  # Parameterized testing
```

## ✅ COMPREHENSIVE TEST COVERAGE

**33 Tests Passing** across all modules:

### Parser Tests (4 tests)

- ✅ Simple schema parsing
- ✅ Complex schema with all component types
- ✅ Invalid RON syntax error handling
- ✅ Component configuration conversion

### Standard Schema Tests (8 tests)

- ✅ Tier determination logic (4 parameterized cases)
- ✅ Schema generation for each tier (3 tests)
- ✅ Integration with `get_standard_schema`

### CalVer Schema Tests (8 tests)

- ✅ Tier determination logic (4 parameterized cases)
- ✅ Schema generation for each tier (3 tests)
- ✅ Integration with `get_calver_schema`

### Preset Dispatcher Tests (3 tests)

- ✅ Standard schema selection
- ✅ CalVer schema selection
- ✅ Unknown schema handling

### Main Module Tests (9 tests)

- ✅ Preset schema integration (6 parameterized cases)
- ✅ Default schema behavior
- ✅ Custom RON schema functionality
- ✅ Error handling (conflicting, unknown, parse errors)

### Core Integration Test (1 test)

- ✅ Empty schema edge case

## Key Architecture Decisions

1. **Functions over Constants**: Schema definitions use functions instead of constants due to Rust's allocation restrictions in const contexts

2. **Correct Field Names**: Uses proper `ZervVars` field names:
    - `current_branch` (not `branch`)
    - `current_commit_hash` (not `commit`)

3. **Tier-Based Logic**: 3-tier system based on Git repository state:
    - **Tier 1**: Clean, tagged state (minimal components)
    - **Tier 2**: Clean with distance (adds post-release metadata)
    - **Tier 3**: Dirty state (adds development identifiers)

4. **Error Standards Compliance**:
    - Uses `ZervError` enum for all custom errors
    - Uses `io::Error::other()` instead of deprecated patterns
    - Includes context in error messages

5. **Comprehensive Testing**:
    - Uses `rstest` for parameterized testing
    - Covers all code paths and error conditions
    - Tests integration between modules

## Usage Examples

### Default Schema

```rust
let vars = ZervVars { major: Some(1), minor: Some(2), patch: Some(3), ..Default::default() };
let zerv = create_zerv_version(vars, None, None)?;  // Uses zerv-standard
```

### Preset Schema

```rust
let zerv = create_zerv_version(vars, Some("zerv-calver"), None)?;
```

### Custom RON Schema

```rust
let ron_schema = r#"
    SchemaConfig(
        core: [(type: "VarField", field: "major")],
        extra_core: [],
        build: [(type: "String", value: "custom")]
    )
"#;
let zerv = create_zerv_version(vars, None, Some(ron_schema))?;
```

## Next Steps

The schema system is complete and ready for CLI integration. Next implementation phases can focus on:

1. **CLI Commands**: Integrate schema system with `zerv version` and `zerv check` commands
2. **Output Formats**: Connect schemas to PEP440/SemVer output formatters
3. **Template System**: Add custom template support for advanced use cases

**All success criteria met. Implementation ready for production use.**
