# Component::VarField Enum Refactoring Plan

## Problem

Currently `Component::VarField(String)` uses strings for field names:

```rust
Component::VarField("major".to_string())
Component::VarField("minor".to_string())
Component::VarField("branch".to_string())
```

This causes:

- **No compile-time validation** - typos only caught at runtime
- **String matching everywhere** - error-prone and slow
- **Hard to discover** - what fields are valid?
- **No IDE support** - no auto-completion

## Solution: VarField Enum

### Step 1: Create VarField Enum

**File**: `src/version/zerv/components.rs`

```rust
use serde::{Deserialize, Serialize};
use strum::{EnumString, Display, EnumIter, AsRefStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display, EnumIter, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum Var {
    // Core version fields
    Major,
    Minor,
    Patch,
    Epoch,

    // Pre-release fields
    PreRelease,

    // Post-release fields
    Post,
    Dev,

    // VCS state fields
    Distance,
    Dirty,

    // VCS context fields (bumped)
    BumpedBranch,
    BumpedCommitHash,
    BumpedCommitHashShort,
    BumpedTimestamp,

    // VCS context fields (last)
    LastBranch,
    LastCommitHash,
    LastTimestamp,

    // Custom fields
    #[serde(rename = "custom")]
    #[strum(disabled)]
    Custom(String), // For custom.* fields
    #[serde(rename = "ts")]
    #[strum(disabled)]
    Timestamp(String), // For timestamp patterns like "%Y-%m-%d"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "str")]
    Str(String),
    #[serde(rename = "int")]
    Int(u64),
    #[serde(rename = "var")]
    Var(Var), // Unified - includes both fields and timestamps
}
```

### Step 2: Update ComponentConfig

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentConfig {
    Str { value: String },
    Int { value: u64 },
    Var { field: Var }, // Unified - includes both fields and timestamps
}
```

### Step 3: Migration Strategy

**Phase 1**: Replace `Component::VarField(String)` with `Component::VarField(VarField)`
**Phase 2**: Update all usage sites to use enum variants
**Phase 3**: Update RON parsing to use enum names
**Phase 4**: Update tests and validation

### Step 4: RON Native Enum Support

```rust
// RON automatically handles enum variants - no custom parsing needed!
// var(Major) → Component::Var(Var::Major)
// var(BumpedBranch) → Component::Var(Var::BumpedBranch)
// var(ts("%Y-%m-%d")) → Component::Var(Var::Timestamp("%Y-%m-%d"))

impl Var {
    /// Create custom field variant
    pub fn custom(field_name: &str) -> Self {
        Var::Custom(field_name.to_string())
    }

    /// Check if this is a custom field
    pub fn is_custom(&self) -> bool {
        matches!(self, Var::Custom(_))
    }
}
```

## Benefits

### 1. **Type Safety**

```rust
// Before: Runtime error
Component::VarField("majr".to_string()) // Typo not caught

// After: Compile error
Component::Var(Var::Majr) // Won't compile
```

### 2. **Performance**

```rust
// Before: String comparison
match field_name.as_str() {
    "major" => process_major(),
    "minor" => process_minor(),
    // ...
}

// After: Enum matching (faster)
match var_field {
    Var::Major => process_major(),
    Var::Minor => process_minor(),
    // ...
}
```

### 3. **Discoverability**

```rust
// IDE auto-completion shows all valid fields
Component::Var(Var::) // Shows: Major, Minor, Patch, etc.
```

### 4. **Exhaustive Matching**

```rust
// Compiler ensures all cases handled
match var_field {
    Var::Major => {},
    Var::Minor => {},
    // Compiler error if any field missing
}
```

## Implementation Steps

### Week 1: Foundation

1. Add `strum` dependency to `Cargo.toml`
2. Create `Var` enum in `components.rs`
3. Add conversion methods for backward compatibility
4. Update `Component` and `ComponentConfig` enums

### Week 2: Migration

1. Update RON parsing to convert strings to `VarField`
2. Update schema processing to use `Var` enum
3. Update bump processing to use enum matching
4. Add comprehensive tests

### Week 3: Cleanup

1. Remove string-based field handling
2. Update error messages to use enum names
3. Update documentation
4. Performance testing

## Example Usage

### Before (String-based)

```rust
let component = Component::VarField("major".to_string());
match component {
    Component::VarField(field_name) => {
        match field_name.as_str() {
            "major" => process_major(),
            "minor" => process_minor(),
            _ => return Err("Unknown field"),
        }
    }
}
```

### After (Enum-based)

```rust
let component = Component::Var(Var::Major);
match component {
    Component::Var(Var::Major) => process_major(),
    Component::Var(Var::Minor) => process_minor(),
    Component::Var(Var::Custom(field)) => process_custom(field),
    Component::Str(s) => process_string(s),
    Component::Int(n) => process_integer(n),
}
```

## RON Schema Changes

### Before (String-based)

```ron
(
    core: [var("major"), var("minor"), var("patch")],
    extra_core: [var("pre_release")],
    build: [var("branch"), ts("%Y-%m-%d")]
)
```

### After (Native RON enum syntax with abbreviation)

```ron
(
    core: [var(Major), var(Minor), var(Patch)],
    extra_core: [var(PreRelease)],
    build: [var(BumpedBranch), var(ts("%Y-%m-%d"))]
)
```

**Benefits**:

- **Native RON enum syntax** - no string parsing needed
- **Compile-time validation** - RON parser validates enum variants
- **IDE support** - auto-completion for enum variants
- **Type safety** - impossible to use invalid field names

## Custom Fields Support

```rust
// Custom fields still supported
Var::Custom("build_id".to_string())
Var::Custom("environment".to_string())

// Standard fields use enum
Var::Major
Var::BumpedBranch
```

This focused approach gives us:

- **Immediate type safety** for standard fields
- **Backward compatibility** for existing schemas
- **Foundation** for future enum expansions
- **Minimal disruption** to existing code

The key insight is starting with just `VarField` - the most commonly used and error-prone component type.
