# Plan 20: Component Resolution Centralization

## Prerequisites

**CRITICAL**: This plan must be implemented **AFTER** Plan 19 (String Sanitization Utils) is completed.

**Reason**: Component resolution will use sanitizers from Plan 19 for format-specific string cleaning.

## Problem

Current implementation has scattered "component → value" mapping logic across different format conversions (SemVer, PEP440), leading to:

1. **Duplicated Logic**: Each format reimplements the same `Var::Post → zerv.vars.post` mapping
2. **Inconsistent Handling**: Different formats handle the same `Var` variants differently
3. **Hard to Extend**: Adding new `Var` types requires updating multiple conversion functions
4. **No Custom Field Support**: Current `Var::Custom(String)` is a hack, spec requires custom hashmap

## Current Implementation Issues

**Scattered Logic Example:**

```rust
// In PEP440 conversion
Var::Post => {
    if let Some(post_num) = zerv.vars.post {
        components.post_number = Some(post_num as u32);
    }
}

// In SemVer conversion
Var::Post => {
    if let Some(post_num) = zerv.vars.post {
        identifiers.push(PreReleaseIdentifier::String("post".to_string()));
        identifiers.push(PreReleaseIdentifier::Integer(post_num));
    }
}
```

**Problems:**

- Same resolution logic (`zerv.vars.post`) duplicated
- Different output handling per format
- No support for custom hashmap from spec
- No consistent string sanitization

## Solution: Centralized Resolution Methods

### Core Concept

Move resolution logic into `Component`/`Var` themselves with sanitization support:

```rust
impl Component {
    fn resolve_value(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Option<String> { ... }
    fn resolve_expanded_values(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Vec<String> { ... }
}

impl Var {
    fn resolve_value(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Option<String> { ... }
    fn resolve_expanded_values(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Vec<String> { ... }
}
```

### Benefits

1. **Single Source of Truth**: Component knows how to resolve its own value
2. **Consistent Behavior**: All formats use same resolution logic
3. **Easier Extension**: New `Var` types only need implementation in one place
4. **Better Testability**: Component resolution tested independently
5. **Sanitization Integration**: Uses Plan 19 sanitizers for format-specific cleaning
6. **Future-Proof**: Supports custom hashmap design from spec

## Core Components

### Resolution Methods on Component

```rust
use crate::utils::sanitize::Sanitizer;

impl Component {
    /// Get just the primary value (no labels)
    pub fn resolve_value(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Option<String> {
        match self {
            Component::Str(s) => Some(sanitizer.sanitize(s)),
            Component::Int(n) => Some(sanitizer.sanitize(&n.to_string())),
            Component::Var(var) => var.resolve_value(vars, sanitizer),
        }
    }

    /// Get expanded values (for formats that need labels + values)
    pub fn resolve_expanded_values(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Vec<String> {
        match self {
            Component::Var(var) => var.resolve_expanded_values(vars, sanitizer),
            // For literals, expanded values is just the single value
            Component::Str(_) | Component::Int(_) => {
                self.resolve_value(vars, sanitizer).map(|v| vec![v]).unwrap_or_default()
            }
        }
    }
}
```

### Resolution Methods on Var

```rust
use crate::version::zerv::resolve_timestamp;

impl Var {
    /// Get just the primary value (no labels)
    pub fn resolve_value(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Option<String> {
        match self {
            // Core version fields
            Var::Major => vars.major.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::Minor => vars.minor.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::Patch => vars.patch.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::Epoch => vars.epoch.map(|v| sanitizer.sanitize(&v.to_string())),

            // Metadata fields - return just the value
            Var::Post => vars.post.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::Dev => vars.dev.map(|v| sanitizer.sanitize(&v.to_string())),

            // Pre-release - return number if present, otherwise None
            Var::PreRelease => {
                vars.pre_release.as_ref()
                    .and_then(|pr| pr.number)
                    .map(|num| sanitizer.sanitize(&num.to_string()))
            }

            // VCS fields
            Var::Branch => vars.bumped_branch.as_ref().map(|b| sanitizer.sanitize(b)),
            Var::Distance => vars.distance.map(|v| sanitizer.sanitize(&v.to_string())),
            Var::CommitHashShort => vars.bumped_commit_hash.as_ref().map(|h| sanitizer.sanitize(h)),

            // Custom fields - lookup in JSON with dot notation
            Var::Custom(name) => {
                vars.get_custom_value(name)
                    .map(|value| sanitizer.sanitize(&value))
            }

            // Timestamp
            Var::Timestamp(pattern) => {
                if let Some(ts) = vars.bumped_timestamp {
                    resolve_timestamp(pattern, ts).ok().map(|result| sanitizer.sanitize(&result))
                } else {
                    None
                }
            }
        }
    }

    /// Helper function for fields that return parts + value
    fn resolve_parts_with_value(&self, vars: &ZervVars, value_sanitizer: &Sanitizer, parts: Vec<String>) -> Vec<String> {
        if let Some(value) = self.resolve_value(vars, value_sanitizer) {
            let mut result = parts;
            result.push(value);
            result
        } else {
            vec![]
        }
    }

    /// Get expanded values with separate sanitizers for keys and values
    pub fn resolve_expanded_values_with_key_sanitizer(
        &self,
        vars: &ZervVars,
        value_sanitizer: &Sanitizer,
        key_sanitizer: &Sanitizer
    ) -> Vec<String> {

        match self {
            // Core version fields - return label + value
            Var::Major => self.resolve_parts_with_value(vars, value_sanitizer, vec!["major".to_string()]),
            Var::Minor => self.resolve_parts_with_value(vars, value_sanitizer, vec!["minor".to_string()]),
            Var::Patch => self.resolve_parts_with_value(vars, value_sanitizer, vec!["patch".to_string()]),
            Var::Epoch => self.resolve_parts_with_value(vars, value_sanitizer, vec!["epoch".to_string()]),

            // Metadata fields - return label + value
            Var::Post => self.resolve_parts_with_value(vars, value_sanitizer, vec!["post".to_string()]),
            Var::Dev => self.resolve_parts_with_value(vars, value_sanitizer, vec!["dev".to_string()]),

            // Pre-release - label + optional value
            Var::PreRelease => {
                if let Some(pr) = &vars.pre_release {
                    let mut parts = vec![key_sanitizer.sanitize(&pr.label.to_string())];
                    if let Some(value) = self.resolve_value(vars, value_sanitizer) {
                        parts.push(value);
                    }
                    parts
                } else {
                    vec![]
                }
            }

            Var::Branch => self.resolve_parts_with_value(vars, value_sanitizer, vec!["branch".to_string()]),
            Var::Distance => self.resolve_parts_with_value(vars, value_sanitizer, vec!["distance".to_string()]),
            Var::CommitHashShort => self.resolve_parts_with_value(vars, value_sanitizer, vec!["commit".to_string()]),

            Var::Custom(name) => {
                let key_parts: Vec<String> = name.split('.')
                    .map(|s| key_sanitizer.sanitize(s))
                    .collect();
                self.resolve_parts_with_value(vars, value_sanitizer, key_parts)
            }

            Var::Timestamp(_) => {
                self.resolve_value(vars, value_sanitizer).map(|v| vec![v]).unwrap_or_default()
            }
        }
    }

    /// Get expanded values (for formats that need labels + values)
    /// Uses identity sanitizer for keys by default
    pub fn resolve_expanded_values(&self, vars: &ZervVars, sanitizer: &Sanitizer) -> Vec<String> {
        let key_sanitizer = Sanitizer::identity();
        self.resolve_expanded_values_with_key_sanitizer(vars, sanitizer, &key_sanitizer)
    }


}
```

## Integration with Plan 19

This plan heavily uses sanitizers from Plan 19:

```rust
// Formats create their own sanitizers and call resolve_value/resolve_expanded_values directly
```

This ensures consistent string sanitization across all component resolution.

## Implementation Files

### Updated Files

- `src/version/zerv/components.rs` - Add resolution methods
- `src/version/pep440/from_zerv.rs` - Use centralized resolution
- `src/version/semver/from_zerv.rs` - Use centralized resolution

### New Files

- `src/version/zerv/resolution.rs` - Core resolution logic
- `src/version/zerv/errors.rs` - ComponentError definitions

## Migration Strategy

### Phase 1: Add Resolution Methods

Add `resolve_value` and `resolve_expanded_values` methods to `Component`/`Var` while keeping existing conversion logic intact.

### Phase 2: Update Conversion Functions

**File**: `src/version/pep440/from_zerv.rs`

```rust
// Before: Manual matching
fn process_var_field_pep440(var: &Var, zerv: &Zerv, components: &mut PEP440Components) {
    match var {
        Var::Post => {
            if let Some(post_num) = zerv.vars.post {
                components.post_label = Some(PostLabel::Post);
                components.post_number = Some(post_num as u32);
            }
        }
        // ... many more cases
    }
}

// After: Centralized resolution
fn process_var_field_pep440(var: &Var, vars: &ZervVars, components: &mut PEP440Components) {
    match var {
        Var::Post => {
            if let Some(value) = var.resolve_value(vars, &Sanitizer::pep440()) {
                if let Ok(num) = value.parse::<u32>() {
                    components.post_label = Some(PostLabel::Post);
                    components.post_number = Some(num);
                }
            }
        }
        Var::Dev => {
            if let Some(value) = var.resolve_value(vars, &Sanitizer::pep440()) {
                if let Ok(num) = value.parse::<u32>() {
                    components.dev_label = Some(DevLabel::Dev);
                    components.dev_number = Some(num);
                }
            }
        }
        _ => {
            if let Some(value) = var.resolve_value(vars, &Sanitizer::pep440()) {
                components.local_overflow.push(LocalSegment::String(value));
            }
        }
    }
}
```

**File**: `src/version/semver/from_zerv.rs`

```rust
// Before: Manual matching
fn process_var_field(identifiers: &mut Vec<PreReleaseIdentifier>, var: &Var, zerv: &Zerv) {
    match var {
        Var::Post => {
            if let Some(post_num) = zerv.vars.post {
                identifiers.push(PreReleaseIdentifier::String("post".to_string()));
                identifiers.push(PreReleaseIdentifier::Integer(post_num));
            }
        }
        // ... many more cases
    }
}

// After: Centralized resolution
fn process_var_field(identifiers: &mut Vec<PreReleaseIdentifier>, var: &Var, vars: &ZervVars) {
    let parts = var.resolve_expanded_values(vars, &Sanitizer::semver());
    for part in parts {
        if let Ok(num) = part.parse::<u64>() {
            identifiers.push(PreReleaseIdentifier::Integer(num));
        } else {
            identifiers.push(PreReleaseIdentifier::String(part));
        }
    }
}
```

### Phase 3: Add Custom Value Lookup Method

**File**: `src/version/zerv/vars.rs`

```rust
impl ZervVars {
    /// Get custom value by key with dot-separated nested access
    /// Examples: "build_id", "metadata.author", "config.database.host"
    pub fn get_custom_value(&self, key: &str) -> Option<String> {
        let mut current = &self.custom;

        for part in key.split('.') {
            current = current.get(part)?;
        }

        match current {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => None, // Unsupported types (arrays, objects, null)
        }
    }
}
```

## Testing Strategy

### Unit Tests

- Test resolution for each component type
- Test sanitization integration
- Test error conditions

### Integration Tests

- Test with real ZervVars data
- Test format-specific resolution
- Test error propagation

### Comprehensive Tests

**File**: `src/version/zerv/components.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::ZervFixture;
    use crate::utils::sanitize::Sanitizer;

    #[test]
    fn test_var_resolution() {
        let zerv = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_post(5)
            .with_branch("main".to_string())
            .build();

        let sanitizer = Sanitizer::default();

        // Test basic resolution
        assert_eq!(Var::Major.resolve_value(&zerv.vars, &sanitizer), Some("1".to_string()));
        assert_eq!(Var::Post.resolve_value(&zerv.vars, &sanitizer), Some("5".to_string()));
        assert_eq!(Var::Branch.resolve_value(&zerv.vars, &sanitizer), Some("main".to_string()));

        // Test expanded values resolution
        assert_eq!(Var::Post.resolve_expanded_values(&zerv.vars, &sanitizer), vec!["post".to_string(), "5".to_string()]);

        // Test custom field expanded values (keys preserved, value sanitized)
        let author_var = Var::Custom("metadata.author".to_string());
        assert_eq!(author_var.resolve_expanded_values(&zerv.vars, &sanitizer), vec!["metadata".to_string(), "author".to_string(), "ci".to_string()]);

        // Test timestamp expanded values (no label, just value)
        let timestamp_var = Var::Timestamp("YYYY".to_string());
        // Assuming timestamp resolves to "2024"
        assert_eq!(timestamp_var.resolve_expanded_values(&zerv.vars, &sanitizer), vec!["2024".to_string()]);

        // Test custom key sanitization
        let pep440_sanitizer = Sanitizer::pep440();
        let custom_key_sanitizer = Sanitizer::str(Some("_"), false, false, None);
        assert_eq!(
            author_var.resolve_expanded_values_with_key_sanitizer(&zerv.vars, &pep440_sanitizer, &custom_key_sanitizer),
            vec!["metadata".to_string(), "author".to_string(), "ci".to_string()]
        );

        // Test missing values
        assert_eq!(Var::Dev.resolve_value(&zerv.vars, &sanitizer), None);
        assert_eq!(Var::Epoch.resolve_value(&zerv.vars, &sanitizer), None);
    }

    #[test]
    fn test_component_resolution() {
        let zerv = ZervFixture::new().with_version(1, 2, 3).build();
        let sanitizer = Sanitizer::default();

        let str_comp = Component::Str("test".to_string());
        let int_comp = Component::Int(42);
        let var_comp = Component::Var(Var::Major);

        assert_eq!(str_comp.resolve_value(&zerv.vars, &sanitizer), Some("test".to_string()));
        assert_eq!(int_comp.resolve_value(&zerv.vars, &sanitizer), Some("42".to_string()));
        assert_eq!(var_comp.resolve_value(&zerv.vars, &sanitizer), Some("1".to_string()));
    }

    #[test]
    fn test_sanitized_resolution() {
        let zerv = ZervFixture::new()
            .with_branch("feature/API-v2".to_string())
            .build();

        // Test format-specific sanitization
        let pep440_sanitizer = Sanitizer::pep440();
        let semver_sanitizer = Sanitizer::semver();

        assert_eq!(
            Var::Branch.resolve_value(&zerv.vars, &pep440_sanitizer),
            Some("feature.api.v2".to_string())
        );
        assert_eq!(
            Var::Branch.resolve_expanded_values(&zerv.vars, &semver_sanitizer),
            vec!["branch".to_string(), "feature.API.v2".to_string()]
        );

        // Test custom sanitization
        let custom_sanitizer = Sanitizer::str(Some("-"), true, false, None);
        assert_eq!(
            Var::Branch.resolve_value(&zerv.vars, &custom_sanitizer),
            Some("feature-api-v2".to_string())
        );
    }

    #[test]
    fn test_custom_field_resolution() {
        let mut zerv = ZervFixture::new()
            .with_version(1, 0, 0)
            .build();

        // Add custom JSON data
        zerv.vars.custom = serde_json::json!({
            "build_id": "123",
            "environment": "prod",
            "metadata": {
                "author": "ci",
                "timestamp": 1703123456
            }
        });

        let pep440_sanitizer = Sanitizer::pep440();

        // Test simple key lookup
        let build_var = Var::Custom("build_id".to_string());
        assert_eq!(build_var.resolve_value(&zerv.vars, &pep440_sanitizer), Some("123".to_string()));

        // Test nested key lookup
        let author_var = Var::Custom("metadata.author".to_string());
        assert_eq!(author_var.resolve_value(&zerv.vars, &pep440_sanitizer), Some("ci".to_string()));

        // Test missing key
        let missing_var = Var::Custom("nonexistent".to_string());
        assert_eq!(missing_var.resolve_value(&zerv.vars, &pep440_sanitizer), None);
    }
}
```

## Benefits After Implementation

1. **Consistency**: All formats use identical resolution logic
2. **Maintainability**: Single place to update for new `Var` types
3. **Testability**: Component resolution tested independently of formats
4. **Extensibility**: Easy to add new resolution logic or custom fields
5. **Sanitization**: Automatic format-specific string cleaning using Plan 19
6. **Spec Compliance**: Supports custom hashmap design from spec
7. **Performance**: Potential for caching resolved values

## Success Criteria

- Single source of truth for component resolution
- Consistent sanitization across formats
- Clean API taking only `&ZervVars` parameter
- All existing functionality preserved
- Format-specific resolution methods work correctly
- Custom field support ready for future implementation
