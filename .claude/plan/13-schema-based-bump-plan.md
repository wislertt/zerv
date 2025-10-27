# Schema-Based Bump Functionality - Future Plan

## Overview

This document outlines a potential future enhancement to add schema-based bump functionality to complement the existing ZervVars field bumps. This would provide a higher-level, schema-aware bumping system.

## Current State

**Existing ZervVars Field Bumps** (Task 3.3):

- `bump_major(increment)` → `vars.major = Some(vars.major.unwrap_or(0) + increment)`
- `bump_minor(increment)` → `vars.minor = Some(vars.minor.unwrap_or(0) + increment)`
- `bump_patch(increment)` → `vars.patch = Some(vars.patch.unwrap_or(0) + increment)`
- `bump_distance(increment)` → `vars.distance = Some(vars.distance.unwrap_or(0) + increment)`
- `bump_post(increment)` → `vars.post = Some(vars.post.unwrap_or(0) + increment)`
- `bump_dev(increment)` → `vars.dev = Some(vars.dev.unwrap_or(0) + increment)`
- `bump_epoch(increment)` → `vars.epoch = Some(vars.epoch.unwrap_or(0) + increment)`
- `bump_pre_release(increment)` → `vars.pre_release.number = Some(vars.pre_release.number.unwrap_or(0) + increment)`

## Proposed Schema-Based Bumps

### Concept

Schema-based bumps work by:

1. **Input**: Schema part (core, extra_core, build), component index, increment
2. **Resolution**: Look up the component at the given index in the schema
3. **Bump Logic**: Determine bump behavior based on component type

### API Design

```rust
// High-level schema-based bump
zerv.bump_by_schema(SchemaPart::Core, 0, 1)        // Bump first component of core schema
zerv.bump_by_schema(SchemaPart::ExtraCore, 2, 3)   // Bump third component of extra_core schema
zerv.bump_by_schema(SchemaPart::Build, 1, 5)       // Bump second component of build schema
```

### Component Type Resolution

| Component Type         | Bump Behavior                    | Example             |
| ---------------------- | -------------------------------- | ------------------- |
| `VarField("major")`    | Call `bump_major(increment)`     | `major` field       |
| `VarField("minor")`    | Call `bump_minor(increment)`     | `minor` field       |
| `VarField("distance")` | Call `bump_distance(increment)`  | `distance` field    |
| `VarTimestamp(_)`      | Call `bump_timestamp(increment)` | Timestamp variables |
| `Integer(_)`           | Bump integer directly in schema  | `int(42)`           |
| `String(_)`            | **Error** - Cannot bump strings  | `str("stable")`     |

## Implementation Structure

### File Organization

```
src/version/zerv/bump/
├── mod.rs                    # BumpType enum and apply_bumps coordinator method
├── vars_primary.rs          # ZervVars field bumps (major, minor, patch)
├── vars_secondary.rs        # ZervVars field bumps (distance, post, dev, etc.)
├── schema.rs                # Schema-based bumps (works for any schema part)
└── tests.rs                 # Tests
```

### Core Implementation

```rust
// src/version/zerv/bump/schema.rs
use crate::version::zerv::schema::SchemaPart;

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaPart {
    Core,
    ExtraCore,
    Build,
}

impl Zerv {
    /// Bump a component by schema part and index
    pub fn bump_by_schema(&mut self, part: SchemaPart, index: usize, increment: u64) -> Result<(), ZervError> {
        let components = match part {
            SchemaPart::Core => &self.schema.core,
            SchemaPart::ExtraCore => &self.schema.extra_core,
            SchemaPart::Build => &self.schema.build,
        };

        let component = components.get(index)
            .ok_or_else(|| ZervError::InvalidIndex(index))?;

        self.bump_component(component, increment)
    }

    /// Bump a specific component based on its type
    fn bump_component(&mut self, component: &Component, increment: u64) -> Result<(), ZervError> {
        match component {
            Component::VarField(field) => self.bump_by_field_name(field, increment),
            Component::VarTimestamp(pattern) => self.bump_timestamp_by_pattern(pattern, increment),
            Component::Integer(current) => self.bump_schema_integer(*current, increment),
            Component::String(_) => Err(ZervError::CannotBumpString),
        }
    }

    /// Bump by field name (delegates to existing ZervVars bumps)
    fn bump_by_field_name(&mut self, field: &str, increment: u64) -> Result<(), ZervError> {
        match field {
            "major" => self.bump_major(increment),
            "minor" => self.bump_minor(increment),
            "patch" => self.bump_patch(increment),
            "distance" => self.bump_distance(increment),
            "post" => self.bump_post(increment),
            "dev" => self.bump_dev(increment),
            "epoch" => self.bump_epoch(increment),
            "pre_release" => self.bump_pre_release(increment),
            _ => Err(ZervError::UnknownField(field.to_string())),
        }
    }
}
```

## Benefits

### 1. **Schema-Aware Bumping**

- Works with any schema configuration
- Automatically adapts to schema changes
- No hardcoded field assumptions

### 2. **Flexible Component Access**

- Bump any component by index
- Works across all schema parts (core, extra_core, build)
- Type-safe component resolution

### 3. **Higher-Level API**

- More semantic than individual field bumps
- Schema-driven rather than field-driven
- Easier to use for complex schemas

### 4. **Extensible Design**

- New schema parts automatically work
- New component types can be added
- Backwards compatible with existing ZervVars bumps

## Use Cases

### CLI Integration

```bash
# Bump first component of core schema
zerv version --bump-schema-core 0 1

# Bump third component of extra_core schema
zerv version --bump-schema-extra-core 2 3

# Bump second component of build schema
zerv version --bump-schema-build 1 5
```

### Programmatic Usage

```rust
// Bump major version (first component of core schema)
zerv.bump_by_schema(SchemaPart::Core, 0, 1);

// Bump distance (third component of extra_core schema)
zerv.bump_by_schema(SchemaPart::ExtraCore, 2, 1);

// Bump timestamp (first component of build schema)
zerv.bump_by_schema(SchemaPart::Build, 0, 1);
```

## Error Handling

### New Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ZervError {
    // ... existing errors ...

    #[error("Invalid component index: {index}")]
    InvalidIndex(usize),

    #[error("Cannot bump string components")]
    CannotBumpString,

    #[error("Unknown field: {field}")]
    UnknownField(String),
}
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_bump_by_schema_core_major() {
    let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
    zerv.bump_by_schema(SchemaPart::Core, 0, 1).unwrap();
    assert_eq!(zerv.vars.major, Some(2));
}

#[test]
fn test_bump_by_schema_invalid_index() {
    let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
    let result = zerv.bump_by_schema(SchemaPart::Core, 10, 1);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ZervError::InvalidIndex(10)));
}

#[test]
fn test_bump_by_schema_string_component() {
    let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
    let result = zerv.bump_by_schema(SchemaPart::Build, 0, 1); // Assuming first component is string
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ZervError::CannotBumpString));
}
```

## Dependencies

### Prerequisites

- ✅ ZervVars field bumps (Task 3.3)
- ✅ Schema system with Component enum
- ✅ Error handling system

### Future Considerations

- Schema validation for component types
- Performance optimization for large schemas
- CLI argument parsing for schema bumps
- Integration with existing bump pipeline

## Priority

**Status**: Future Enhancement
**Priority**: Medium
**Estimated Time**: 4-6 hours
**Dependencies**: Task 3.3 (ZervVars field bumps) must be completed first

## Conclusion

Schema-based bumps provide a powerful, flexible way to bump version components based on schema configuration rather than hardcoded field names. This complements the existing ZervVars field bumps and provides a higher-level API for schema-aware version management.

The implementation is straightforward and leverages existing bump functionality, making it a natural extension to the current bump system.
