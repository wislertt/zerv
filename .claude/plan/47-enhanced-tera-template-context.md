# Plan 47: Enhanced Tera Template Context with Version Parsing

**Status**: Completed
**Priority**: Medium
**Context**: Users need more granular access to version components in Tera templates, particularly for SemVer and PEP440 format parsing
**Goals**: Provide structured access to version parts (base, pre-release, build) and add PEP440 label parsing

## Implementation Plan

### 1. Enhance Template Context Structures

**File**: `src/cli/utils/template/context.rs`

**Add import at top of file:**

```rust
use crate::version::pep440::PEP440;
use crate::version::pep440::utils::pre_release_label_to_pep440_string;
use crate::version::semver::SemVer;
use crate::version::zerv::Zerv;
```

**Update PreReleaseContext struct** - add PEP440 label using existing function:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PreReleaseContext {
    pub label: String,                    // Current: "alpha", "beta", etc.
    pub number: Option<u64>,
    pub pep440_label: Option<String>,     // PEP440 format: "rc", "a", "b", etc.
}
```

**Add new fields to ZervTemplateContext struct**:

```rust
// SemVer parsed components
pub semver_base_part: String,
pub semver_pre_release_part: Option<String>,  // None if no pre-release
pub semver_build_part: Option<String>,         // None if no build metadata
pub semver_docker: String,

// PEP440 parsed components
pub pep440_base_part: String,
pub pep440_pre_release_part: Option<String>,  // None if no pre-release
pub pep440_build_part: Option<String>,         // None if no build metadata
```

### 2. Add Conditional Prefix Function to Tera

**Add new function to template functions** - File: `src/cli/utils/template/functions.rs`

```rust
/// Add conditional prefix to string (only if string is not empty)
/// Usage: {{ prefix_if(value, prefix="+") }}
fn prefix_if_function(args: &std::collections::HashMap<String, Value>) -> Result<Value, tera::Error> {
    let value = args
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("prefix_if function requires a 'value' parameter"))?;

    let prefix = args
        .get("prefix")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("prefix_if function requires a 'prefix' parameter"))?;

    if value.is_empty() {
        Ok(Value::String("".to_string()))
    } else {
        Ok(Value::String(format!("{}{}", prefix, value)))
    }
}
```

**Update register_functions() to include the new function:**

```rust
pub fn register_functions(tera: &mut Tera) -> Result<(), ZervError> {
    tera.register_function("sanitize", Box::new(sanitize_function));
    tera.register_function("hash", Box::new(hash_function));
    tera.register_function("hash_int", Box::new(hash_int_function));
    tera.register_function("prefix", Box::new(prefix_function));
    tera.register_function("prefix_if", Box::new(prefix_if_function));  // NEW
    tera.register_function("format_timestamp", Box::new(format_timestamp_function));
    Ok(())
}
```

### 3. Add Methods to Version Classes

#### Add methods to SemVer class

**File**: `src/version/semver.rs`

```rust
impl SemVer {
    pub fn to_base_part(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn to_pre_release_part(&self) -> Option<String> {
        self.pre_release.as_ref().map(|pr| pr.to_string())
    }

    pub fn to_build_part(&self) -> Option<String> {
        self.build.as_ref().map(|b| b.to_string())
    }

    pub fn to_docker_format(&self) -> String {
        let mut parts = vec![self.to_base_part()];
        if let Some(pre) = self.to_pre_release_part() {
            parts.push(pre);
        }
        if let Some(build) = self.to_build_part() {
            parts.push(build);
        }
        parts.join("-")
    }
}
```

#### Add methods to PEP440 class

**File**: `src/version/pep440/mod.rs` or appropriate file

```rust
impl PEP440 {
    pub fn to_base_part(&self) -> String {
        // Return base version including epoch if present
        // Implementation depends on PEP440 structure
    }

    pub fn to_pre_release_part(&self) -> Option<String> {
        // Return pre-release components
        // Implementation depends on PEP440 structure
    }

    pub fn to_build_part(&self) -> Option<String> {
        // Return build metadata
        // Implementation depends on PEP440 structure
    }
}
```

### 4. Update ZervTemplateContext::from_zerv()

Modify the `from_zerv()` method to use the new methods:

```rust
impl ZervTemplateContext {
    pub fn from_zerv(zerv: &Zerv) -> Self {
        let vars = &zerv.vars;

        let semver = SemVer::from(zerv.clone());
        let pep440 = PEP440::from(zerv.clone());

        Self {
            // ... existing fields ...
            pre_release: vars.pre_release.as_ref().map(|pr| PreReleaseContext {
                label: pr.label.label_str().to_string(),
                number: pr.number,
                pep440_label: Some(pre_release_label_to_pep440_string(&pr.label).to_string()),
            }),
            semver_base_part: semver.to_base_part(),
            semver_pre_release_part: semver.to_pre_release_part(),
            semver_build_part: semver.to_build_part(),
            semver_docker: semver.to_docker_format(),
            pep440_base_part: pep440.to_base_part(),
            pep440_pre_release_part: pep440.to_pre_release_part(),
            pep440_build_part: pep440.to_build_part(),
        }
    }
}
```

### 5. Add Comprehensive Tests

Add test cases covering:

- Basic SemVer parsing: `1.2.3`
- SemVer with pre-release: `1.2.3-alpha.1`
- SemVer with build: `1.2.3+build.456`
- SemVer with both: `1.2.3-alpha.1+build.456`
- Basic PEP440: `1.2.3`
- PEP440 with rc: `1.2.3rc1`
- PEP440 with dev: `1.2.3dev1`
- PEP440 with post: `1.2.3.post1`
- Complex PEP440: `1.2.3rc1.post2.dev3+build.456`

### 5. Integration with Existing Version Classes

Ensure compatibility with:

- `SemVer` class in `src/version/semver.rs`
- `PEP440` class in `src/version/pep440.rs`
- `Zerv` class in `src/version/zerv.rs`

## Testing Strategy

### Unit Tests

- Test each parsing function with various version formats
- Test edge cases (empty strings, malformed versions)
- Test integration with existing version classes

### Integration Tests

- Test template rendering with new context fields
- Verify Docker tag format generation
- Test with real Zerv versions from fixtures

## Success Criteria

1. ✅ All new template context fields are properly populated
2. ✅ SemVer parsing correctly separates base, pre-release, and build parts
3. ✅ PEP440 parsing correctly extracts labels (rc, a, b) and components
4. ✅ Docker format follows `base-pre-release-build` pattern
5. ✅ All existing tests pass without modification
6. ✅ New comprehensive test coverage for parsing functions
7. ✅ Template examples work correctly with new fields

## Documentation Updates

- Update template documentation with new field examples
- Add usage examples for Docker tag generation
- Document PEP440 label mapping

## Example Template Usage

After implementation, templates will be able to use:

```tera
# SemVer components
semver: {{ semver }}
base: {{ semver_base_part }}
pre: {{ semver_pre_release_part }}
build: {{ semver_build_part }}
docker: {{ semver_docker }}

# PEP440 components
pep440: {{ pep440 }}
base: {{ pep440_base_part }}
pre: {{ pep440_pre_release_part }}
build: {{ pep440_build_part }}

# Pre-release context (access both label formats)
{% if pre_release %}
label: {{ pre_release.label }}        # "alpha", "beta", etc.
pep440_label: {{ pre_release.pep440_label }}  # "a", "b", "rc", etc.
number: {{ pre_release.number }}
{% endif %}

# Conditional prefix usage (NEW)
# semver_build_part might be "build.456" or ""
build_with_prefix: {{ prefix_if(semver_build_part, prefix="+") }}  # "+build.456" or ""
pre_with_prefix: {{ prefix_if(semver_pre_release_part, prefix="-") }}  # "-alpha.1" or ""
```

## Implementation Notes

- **COMPLETED**: All features implemented successfully
- **COMPLETED**: All tests passing (2,334 tests)
- **COMPLETED**: Template context enhanced with new fields
- **COMPLETED**: prefix_if function added and tested
- **COMPLETED**: SemVer and PEP440 parsing methods implemented
- **COMPLETED**: Comprehensive test coverage added
- **COMPLETED**: Backward compatibility maintained
