# Step 2: Schema System Implementation

## Overview

Implement RON schema parsing, `zerv-standard` and `zerv-calver` presets, and `create_zerv_version` function that bridges `ZervVars` to `Zerv` objects.

## File Structure

```
src/schema/
├── mod.rs              # Main module with create_zerv_version function
├── parser.rs           # RON schema parsing
└── presets/
    ├── mod.rs          # Preset dispatcher
    ├── standard.rs     # zerv-standard (tier-based major.minor.patch)
    └── calver.rs       # zerv-calver (calendar versioning)
```

## 1. Schema Parser (`src/schema/parser.rs`)

```rust
use crate::version::zerv::{Component, ZervSchema};
use crate::error::ZervError;
use ron::de::from_str;
use serde::Deserialize;

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

impl From<ComponentConfig> for Component {
    fn from(config: ComponentConfig) -> Self {
        match config {
            ComponentConfig::String { value } => Component::String(value),
            ComponentConfig::Integer { value } => Component::Integer(value),
            ComponentConfig::VarField { field } => Component::VarField(field),
            ComponentConfig::VarTimestamp { pattern } => Component::VarTimestamp(pattern),
        }
    }
}

impl From<SchemaConfig> for ZervSchema {
    fn from(config: SchemaConfig) -> Self {
        ZervSchema {
            core: config.core.into_iter().map(Component::from).collect(),
            extra_core: config.extra_core.into_iter().map(Component::from).collect(),
            build: config.build.into_iter().map(Component::from).collect(),
        }
    }
}

pub fn parse_ron_schema(ron_str: &str) -> Result<ZervSchema, ZervError> {
    let config: SchemaConfig = from_str(ron_str)
        .map_err(|e| ZervError::SchemaParseError(format!("Invalid RON schema: {}", e)))?;
    Ok(config.into())
}
```

## 2. Built-in Presets

### Preset Dispatcher (`src/schema/presets/mod.rs`)

```rust
mod standard;
mod calver;

pub use standard::{get_standard_schema, ZERV_STANDARD_TIER_1, ZERV_STANDARD_TIER_2, ZERV_STANDARD_TIER_3};
pub use calver::{get_calver_schema, ZERV_CALVER_TIER_1, ZERV_CALVER_TIER_2, ZERV_CALVER_TIER_3};

use crate::version::zerv::{ZervSchema, ZervVars};

pub fn get_preset_schema(name: &str, vars: &ZervVars) -> Option<ZervSchema> {
    match name {
        "zerv-standard" => Some(get_standard_schema(vars)),
        "zerv-calver" => Some(get_calver_schema(vars)),
        _ => None,
    }
}
```

### Standard Preset (`src/schema/presets/standard.rs`)

```rust
use crate::version::zerv::{Component, ZervSchema, ZervVars};

// Tier 1: Tagged, clean - major.minor.patch
pub const ZERV_STANDARD_TIER_1: ZervSchema = ZervSchema {
    core: vec![
        Component::VarField("major".to_string()),
        Component::VarField("minor".to_string()),
        Component::VarField("patch".to_string()),
    ],
    extra_core: vec![],
    build: vec![],
};

// Tier 2: Distance, clean - major.minor.patch.post<distance>+branch.<commit>
pub const ZERV_STANDARD_TIER_2: ZervSchema = ZervSchema {
    core: vec![
        Component::VarField("major".to_string()),
        Component::VarField("minor".to_string()),
        Component::VarField("patch".to_string()),
    ],
    extra_core: vec![
        Component::VarField("epoch".to_string()),
        Component::VarField("pre_release".to_string()),
        Component::VarField("post".to_string()),
    ],
    build: vec![
        Component::VarField("branch".to_string()),
        Component::VarField("commit".to_string()),
    ],
};

// Tier 3: Dirty - major.minor.patch.dev<timestamp>+branch.<distance>.<commit>
pub const ZERV_STANDARD_TIER_3: ZervSchema = ZervSchema {
    core: vec![
        Component::VarField("major".to_string()),
        Component::VarField("minor".to_string()),
        Component::VarField("patch".to_string()),
    ],
    extra_core: vec![
        Component::VarField("epoch".to_string()),
        Component::VarField("pre_release".to_string()),
        Component::VarField("post".to_string()),
        Component::VarField("dev".to_string()),
    ],
    build: vec![
        Component::VarField("branch".to_string()),
        Component::VarField("distance".to_string()),
        Component::VarField("commit".to_string()),
    ],
};

pub fn get_standard_schema(vars: &ZervVars) -> ZervSchema {
    let tier = determine_tier(vars);
    match tier {
        1 => ZERV_STANDARD_TIER_1.clone(),
        2 => ZERV_STANDARD_TIER_2.clone(),
        3 => ZERV_STANDARD_TIER_3.clone(),
        _ => unreachable!("Invalid tier"),
    }
}

fn determine_tier(vars: &ZervVars) -> u8 {
    if vars.dirty.unwrap_or(false) {
        3 // Dirty
    } else if vars.distance.unwrap_or(0) > 0 {
        2 // Distance, clean
    } else {
        1 // Tagged, clean
    }
}
```

### CalVer Preset (`src/schema/presets/calver.rs`)

```rust
use crate::version::zerv::{Component, ZervSchema, ZervVars};

// Tier 1: Tagged, clean - YYYY-MM-DD-PATCH
pub const ZERV_CALVER_TIER_1: ZervSchema = ZervSchema {
    core: vec![
        Component::VarTimestamp("YYYY".to_string()),
        Component::VarTimestamp("MM".to_string()),
        Component::VarTimestamp("DD".to_string()),
        Component::VarField("patch".to_string()),
    ],
    extra_core: vec![],
    build: vec![],
};

// Tier 2: Distance, clean - YYYY-MM-DD-PATCH.post<distance>+branch.<commit>
pub const ZERV_CALVER_TIER_2: ZervSchema = ZervSchema {
    core: vec![
        Component::VarTimestamp("YYYY".to_string()),
        Component::VarTimestamp("MM".to_string()),
        Component::VarTimestamp("DD".to_string()),
        Component::VarField("patch".to_string()),
    ],
    extra_core: vec![
        Component::VarField("epoch".to_string()),
        Component::VarField("pre_release".to_string()),
        Component::VarField("post".to_string()),
    ],
    build: vec![
        Component::VarField("branch".to_string()),
        Component::VarField("commit".to_string()),
    ],
};

// Tier 3: Dirty - YYYY-MM-DD-PATCH.dev<timestamp>+branch.<distance>.<commit>
pub const ZERV_CALVER_TIER_3: ZervSchema = ZervSchema {
    core: vec![
        Component::VarTimestamp("YYYY".to_string()),
        Component::VarTimestamp("MM".to_string()),
        Component::VarTimestamp("DD".to_string()),
        Component::VarField("patch".to_string()),
    ],
    extra_core: vec![
        Component::VarField("epoch".to_string()),
        Component::VarField("pre_release".to_string()),
        Component::VarField("post".to_string()),
        Component::VarField("dev".to_string()),
    ],
    build: vec![
        Component::VarField("branch".to_string()),
        Component::VarField("distance".to_string()),
        Component::VarField("commit".to_string()),
    ],
};

pub fn get_calver_schema(vars: &ZervVars) -> ZervSchema {
    let tier = determine_tier(vars);
    match tier {
        1 => ZERV_CALVER_TIER_1.clone(),
        2 => ZERV_CALVER_TIER_2.clone(),
        3 => ZERV_CALVER_TIER_3.clone(),
        _ => unreachable!("Invalid tier"),
    }
}

fn determine_tier(vars: &ZervVars) -> u8 {
    if vars.dirty.unwrap_or(false) {
        3 // Dirty
    } else if vars.distance.unwrap_or(0) > 0 {
        2 // Distance, clean
    } else {
        1 // Tagged, clean
    }
}
```

## 3. Main Schema Module (`src/schema/mod.rs`)

```rust
mod parser;
mod presets;

pub use parser::{parse_ron_schema, SchemaConfig, ComponentConfig};
pub use presets::{
    get_preset_schema, get_standard_schema, get_calver_schema,
    ZERV_STANDARD_TIER_1, ZERV_STANDARD_TIER_2, ZERV_STANDARD_TIER_3,
    ZERV_CALVER_TIER_1, ZERV_CALVER_TIER_2, ZERV_CALVER_TIER_3
};

use crate::version::zerv::{Zerv, ZervVars};
use crate::error::ZervError;

pub fn create_zerv_version(
    vars: ZervVars,
    schema_name: Option<&str>,
    schema_ron: Option<&str>,
) -> Result<Zerv, ZervError> {
    let schema = match (schema_name, schema_ron) {
        // Error if both are provided
        (Some(_), Some(_)) => {
            return Err(ZervError::ConflictingSchemas(
                "Cannot specify both schema_name and schema_ron".to_string()
            ));
        }

        // Custom RON schema
        (None, Some(ron_str)) => parse_ron_schema(ron_str)?,

        // Built-in schema
        (Some(name), None) => {
            if let Some(schema) = get_preset_schema(name, &vars) {
                schema
            } else {
                return Err(ZervError::UnknownSchema(name.to_string()));
            }
        }

        // Neither provided - use default
        (None, None) => get_preset_schema("zerv-standard", &vars).unwrap(),
    };

    Ok(Zerv { schema, vars })
}
```

## 4. Error Types (Add to `src/error.rs`)

```rust
// Add these variants to ZervError enum
SchemaParseError(String),
UnknownSchema(String),
ConflictingSchemas(String),
```

**Note**: All error handling must follow project error standards:

- Use `ZervError` for all custom errors
- Use `io::Error::other()` instead of `io::Error::new(io::ErrorKind::Other, ...)`
- Include context in error messages for debugging

## 5. Integration (Update `src/lib.rs`)

```rust
pub mod schema;
```

## 6. Test Cases

### Unit Tests (`src/schema/mod.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::ZervVars;

    use rstest::rstest;

    #[rstest]
    #[case(
        "zerv-standard",
        ZervVars {
            major: Some(1), minor: Some(2), patch: Some(3),
            dirty: Some(false), distance: Some(0),
            ..Default::default()
        },
        &ZERV_STANDARD_TIER_1
    )]
    #[case(
        "zerv-standard",
        ZervVars {
            major: Some(1), minor: Some(2), patch: Some(3),
            dirty: Some(false), distance: Some(5), post: Some(5),
            branch: Some("main".to_string()), commit: Some("abc123".to_string()),
            ..Default::default()
        },
        &ZERV_STANDARD_TIER_2
    )]
    #[case(
        "zerv-standard",
        ZervVars {
            major: Some(1), minor: Some(2), patch: Some(3),
            dirty: Some(true), dev: Some(1234567890),
            branch: Some("feature".to_string()), commit: Some("def456".to_string()),
            ..Default::default()
        },
        &ZERV_STANDARD_TIER_3
    )]
    #[case(
        "zerv-calver",
        ZervVars {
            patch: Some(1), dirty: Some(false), distance: Some(0),
            tag_timestamp: Some(1710547200),
            ..Default::default()
        },
        &ZERV_CALVER_TIER_1
    )]
    #[case(
        "zerv-calver",
        ZervVars {
            patch: Some(1), dirty: Some(false), distance: Some(5), post: Some(5),
            branch: Some("main".to_string()), commit: Some("abc123".to_string()),
            tag_timestamp: Some(1710547200),
            ..Default::default()
        },
        &ZERV_CALVER_TIER_2
    )]
    #[case(
        "zerv-calver",
        ZervVars {
            patch: Some(1), dirty: Some(true), dev: Some(1234567890),
            branch: Some("feature".to_string()), commit: Some("def456".to_string()),
            tag_timestamp: Some(1710547200),
            ..Default::default()
        },
        &ZERV_CALVER_TIER_3
    )]
    fn test_preset_schemas(
        #[case] schema_name: &str,
        #[case] vars: ZervVars,
        #[case] expected_schema: &ZervSchema,
    ) {
        let zerv = create_zerv_version(vars, Some(schema_name), None).unwrap();
        assert_eq!(zerv.schema, *expected_schema);
    }

    #[test]
    fn test_default_schema() {
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };

        let zerv = create_zerv_version(vars, None, None).unwrap();
        assert_eq!(zerv.schema.core.len(), 3); // Uses zerv-standard by default
    }

    #[test]
    fn test_custom_ron_schema() {
        let vars = ZervVars::default();
        let ron_schema = r#"
            SchemaConfig(
                core: [
                    (type: "VarField", field: "major"),
                    (type: "VarField", field: "minor"),
                ],
                extra_core: [],
                build: [(type: "String", value: "custom")]
            )
        "#;

        let zerv = create_zerv_version(vars, None, Some(ron_schema)).unwrap();
        assert_eq!(zerv.schema.core.len(), 2);
        assert_eq!(zerv.schema.build.len(), 1);
    }

    #[test]
    fn test_conflicting_schemas_error() {
        let vars = ZervVars::default();
        let ron_schema = "SchemaConfig(core: [], extra_core: [], build: [])";
        let result = create_zerv_version(vars, Some("zerv-standard"), Some(ron_schema));
        assert!(matches!(result, Err(ZervError::ConflictingSchemas(_))));
    }

    #[test]
    fn test_unknown_schema_error() {
        let vars = ZervVars::default();
        let result = create_zerv_version(vars, Some("unknown"), None);
        assert!(matches!(result, Err(ZervError::UnknownSchema(_))));
    }

    #[test]
    fn test_invalid_ron_schema_error() {
        let vars = ZervVars::default();
        let invalid_ron = "invalid ron syntax";
        let result = create_zerv_version(vars, None, Some(invalid_ron));
        assert!(matches!(result, Err(ZervError::SchemaParseError(_))));
    }
}
```

### Integration Tests (`src/schema/presets/standard.rs` and `src/schema/presets/calver.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::ZervVars;
    use rstest::rstest;

    #[rstest]
    #[case(ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, 1)]
    #[case(ZervVars { dirty: Some(false), distance: Some(5), ..Default::default() }, 2)]
    #[case(ZervVars { dirty: Some(true), distance: Some(0), ..Default::default() }, 3)]
    #[case(ZervVars { dirty: Some(true), distance: Some(10), ..Default::default() }, 3)]
    fn test_tier_determination(#[case] vars: ZervVars, #[case] expected_tier: u8) {
        assert_eq!(determine_tier(&vars), expected_tier);
    }
}
```

## 7. Dependencies (Add to `Cargo.toml`)

```toml
[dependencies]
ron = "0.8"
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
rstest = "0.18"
```

## Implementation Order

1. **Add error types** to `src/error.rs`
2. **Create parser module** with RON parsing
3. **Create presets module** with tier-aware logic
4. **Create main module** with `create_zerv_version`
5. **Add integration** to `src/lib.rs`
6. **Write and run tests** to verify functionality

## Success Criteria

- [ ] `create_zerv_version` function works with `zerv-default` schema
- [ ] RON schema parsing handles custom schemas
- [ ] Tier-aware logic correctly determines schema based on VCS state
- [ ] All unit tests pass
- [ ] Integration with existing `ZervVars` and `Zerv` types works
- [ ] Error handling for invalid schemas and unknown schema names follows project error standards (use `ZervError` and `io::Error::other()` instead of `io::Error::new(io::ErrorKind::Other, ...)`)
