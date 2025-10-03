# ZervFixture Improvement Plan

## Overview

This document outlines the plan to improve the ZervFixture by following the chainable pattern established in VersionArgsFixture, minimizing methods and making the API more intuitive and maintainable.

## Current State Analysis

### Current ZervFixture Issues

1. **Too many static methods**: Many specialized methods like `pep_zerv_1_2_3_alpha_1()`, `sem_zerv_1_0_0_alpha_1()` etc.
2. **Non-chainable API**: Methods return `Zerv` instead of `Self` for chaining
3. **Redundant legacy methods**: Multiple ways to do the same thing
4. **Complex constructors**: Methods like `with_all_components()` take too many parameters
5. **Inconsistent patterns**: Mix of builder pattern and static factory methods

### VersionArgsFixture Pattern (Good Example)

```rust
pub struct VersionArgsFixture {
    args: VersionArgs,
}

impl VersionArgsFixture {
    pub fn new() -> Self { /* ... */ }
    pub fn build(self) -> VersionArgs { /* ... */ }

    // Chainable methods
    pub fn with_source(mut self, source: &str) -> Self { /* ... */ }
    pub fn with_schema(mut self, schema: &str) -> Self { /* ... */ }
    pub fn with_tag_version(mut self, tag_version: &str) -> Self { /* ... */ }
}
```

## Target State Design

### Core Principles

1. **Single entry point**: `ZervFixture::new()` or `ZervFixture::default()`
2. **Chainable methods**: All methods return `Self` for chaining
3. **Minimal API**: Only essential methods, remove redundant ones
4. **Clear naming**: Method names clearly indicate what they do
5. **Consistent patterns**: Follow the same pattern as VersionArgsFixture

### Proposed API

```rust
pub struct ZervFixture {
    zerv: Zerv,
}

impl ZervFixture {
    // Core constructors
    pub fn new() -> Self
    pub fn build(self) -> Zerv

    // Version components (chainable)
    pub fn with_version(mut self, major: u64, minor: u64, patch: u64) -> Self
    pub fn with_major(mut self, major: u64) -> Self
    pub fn with_minor(mut self, minor: u64) -> Self
    pub fn with_patch(mut self, patch: u64) -> Self

    // Extended components (chainable)
    pub fn with_epoch(mut self, epoch: u64) -> Self
    pub fn with_pre_release(mut self, label: PreReleaseLabel, number: Option<u64>) -> Self
    pub fn with_post(mut self, post: u64) -> Self
    pub fn with_dev(mut self, dev: u64) -> Self

    // Schema components (chainable)
    pub fn with_extra_core(mut self, components: Vec<Component>) -> Self
    pub fn with_build(mut self, components: Vec<Component>) -> Self

    // VCS data (chainable)
    pub fn with_vcs_data(mut self, distance: u64, dirty: bool, /* ... */) -> Self

    // Common presets (chainable)
    pub fn basic() -> Self  // 1.0.0
    pub fn semver() -> Self // Basic SemVer setup
    pub fn pep440() -> Self // Basic PEP440 setup
}
```

## Implementation Plan

### Phase 0: Foundation Redesign (Eliminate ZervSchemaFixture, Redesign ZervVarsFixture)

#### 0.1 Redesign ZervSchemaFixture to Use Presets First ✅

**Key Insight:** ZervSchemaFixture should primarily use existing schema presets from `src/schema/presets/*`, but keep advanced methods for rare edge cases:

- `zerv_standard_tier_1()` - Basic major.minor.patch
- `zerv_standard_tier_2()` - With build metadata
- `zerv_standard_tier_3()` - With dev components
- `zerv_calver_tier_1/2/3()` - CalVer variants

**Target Design (Phase 0 - Minimal):**

```rust
impl ZervSchemaFixture {
    // Primary constructors using presets
    pub fn new() -> Self                    // Uses zerv_standard_tier_1()
    pub fn build(self) -> ZervSchema

    // Preset constructors
    pub fn standard_tier_1() -> Self        // Uses zerv_standard_tier_1()
    pub fn standard_tier_2() -> Self        // Uses zerv_standard_tier_2()
    pub fn standard_tier_3() -> Self        // Uses zerv_standard_tier_3()
    pub fn calver_tier_1() -> Self          // Uses zerv_calver_tier_1()
    pub fn calver_tier_2() -> Self          // Uses zerv_calver_tier_2()
    pub fn calver_tier_3() -> Self          // Uses zerv_calver_tier_3()
}

// Advanced methods for rare cases - IMPLEMENT LATER
// pub fn with_extra_core(mut self, components: Vec<Component>) -> Self
// pub fn add_extra_core(mut self, component: Component) -> Self
// pub fn with_build(mut self, components: Vec<Component>) -> Self
// pub fn add_build(mut self, component: Component) -> Self
```

#### 0.2 Redesign ZervVarsFixture ✅

**Current Issues:**

- Missing `new()` + `build()` pattern
- `with_version()` is static constructor (not chainable)
- Has unused RON methods (`to_ron_string`, `from_ron_string`)
- Mix of static constructors and chainable methods

**Target Design (Phase 0 - Minimal):**

```rust
impl ZervVarsFixture {
    pub fn new() -> Self                    // Default 1.0.0 (replaces basic())
    pub fn build(self) -> ZervVars

    // Convert to chainable
    pub fn with_version(mut self, major: u64, minor: u64, patch: u64) -> Self

    // Keep existing chainable methods (already correct):
    pub fn with_pre_release(mut self, label: PreReleaseLabel, number: Option<u64>) -> Self
    pub fn with_epoch(mut self, epoch: u64) -> Self
    pub fn with_post(mut self, post: u64) -> Self
    pub fn with_dev(mut self, dev: u64) -> Self
    pub fn with_distance(mut self, distance: u64) -> Self
    pub fn with_dirty(mut self, dirty: bool) -> Self
    pub fn with_vcs_data(mut self, ...) -> Self
    // ... etc (all existing chainable methods)
}

// Remove unused methods:
// - basic() -> replaced by new()
// - to_ron_string() -> unused
// - from_ron_string() -> unused
```

### Phase 1: ZervFixture Refactoring

#### 1.1 Update ZervFixture to Use Schema Presets

**After Phase 0 completion, update ZervFixture:**

```rust
use crate::schema::presets::{zerv_standard_tier_1, zerv_standard_tier_2, zerv_standard_tier_3};

impl ZervFixture {
    pub fn new() -> Self {
        Self {
            zerv: Zerv::new(
                zerv_standard_tier_1(),  // Use preset directly
                ZervVarsFixture::new().build(),
            ).unwrap()
        }
    }

    pub fn build(self) -> Zerv { self.zerv }

    // Chainable methods that coordinate schema + vars
    pub fn with_version(mut self, major: u64, minor: u64, patch: u64) -> Self
    pub fn with_epoch(mut self, epoch: u64) -> Self  // Updates vars only
    pub fn with_pre_release(mut self, label: PreReleaseLabel, number: Option<u64>) -> Self

    // Schema preset methods
    pub fn with_standard_tier_1(mut self) -> Self  // Basic schema
    pub fn with_standard_tier_2(mut self) -> Self  // With build metadata
    pub fn with_standard_tier_3(mut self) -> Self  // With dev components
    // ... etc
}
```

#### 1.2 Simplify Constructor Pattern

**Current:**

```rust
impl ZervFixture {
    pub fn basic() -> Self { /* ... */ }
    pub fn with_version(major: u64, minor: u64, patch: u64) -> Self { /* ... */ }
    pub fn with_pre_release(label: PreReleaseLabel, number: Option<u64>) -> Self { /* ... */ }
    // ... many more static constructors
}
```

**Target:**

```rust
impl ZervFixture {
    /// Create a new fixture with default 1.0.0 version
    pub fn new() -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::basic().into(),
                ZervVarsFixture::basic().into(),
            ).unwrap_or_else(|e| panic!("Failed to create basic Zerv: {e}"))
        }
    }

    /// Build and return the final Zerv
    pub fn build(self) -> Zerv {
        self.zerv
    }

    /// Alias for new() for consistency
    pub fn basic() -> Self {
        Self::new()
    }
}
```

#### 1.2 Add Chainable Version Methods

```rust
impl ZervFixture {
    /// Set version components
    pub fn with_version(mut self, major: u64, minor: u64, patch: u64) -> Self {
        self.zerv.vars.major = Some(major);
        self.zerv.vars.minor = Some(minor);
        self.zerv.vars.patch = Some(patch);
        self
    }

    /// Set major version
    pub fn with_major(mut self, major: u64) -> Self {
        self.zerv.vars.major = Some(major);
        self
    }

    /// Set minor version
    pub fn with_minor(mut self, minor: u64) -> Self {
        self.zerv.vars.minor = Some(minor);
        self
    }

    /// Set patch version
    pub fn with_patch(mut self, patch: u64) -> Self {
        self.zerv.vars.patch = Some(patch);
        self
    }
}
```

#### 1.3 Add Chainable Extended Component Methods

```rust
impl ZervFixture {
    /// Set epoch
    pub fn with_epoch(mut self, epoch: u64) -> Self {
        self.zerv.vars.epoch = Some(epoch);
        // Add epoch to schema if not present
        if !self.zerv.schema.extra_core.contains(&Component::VarField(ron_fields::EPOCH.to_string())) {
            self.zerv.schema.extra_core.push(Component::VarField(ron_fields::EPOCH.to_string()));
        }
        self
    }

    /// Set pre-release
    pub fn with_pre_release(mut self, label: PreReleaseLabel, number: Option<u64>) -> Self {
        self.zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        // Add pre-release to schema if not present
        if !self.zerv.schema.extra_core.contains(&Component::VarField(ron_fields::PRE_RELEASE.to_string())) {
            self.zerv.schema.extra_core.push(Component::VarField(ron_fields::PRE_RELEASE.to_string()));
        }
        self
    }

    /// Set post version
    pub fn with_post(mut self, post: u64) -> Self {
        self.zerv.vars.post = Some(post);
        // Add post to schema if not present
        if !self.zerv.schema.extra_core.contains(&Component::VarField(ron_fields::POST.to_string())) {
            self.zerv.schema.extra_core.push(Component::VarField(ron_fields::POST.to_string()));
        }
        self
    }

    /// Set dev version
    pub fn with_dev(mut self, dev: u64) -> Self {
        self.zerv.vars.dev = Some(dev);
        // Add dev to schema if not present
        if !self.zerv.schema.extra_core.contains(&Component::VarField(ron_fields::DEV.to_string())) {
            self.zerv.schema.extra_core.push(Component::VarField(ron_fields::DEV.to_string()));
        }
        self
    }
}
```

#### 1.4 Add Chainable Schema Methods

```rust
impl ZervFixture {
    /// Set extra core components
    pub fn with_extra_core(mut self, components: Vec<Component>) -> Self {
        self.zerv.schema.extra_core = components;
        self
    }

    /// Add single extra core component
    pub fn add_extra_core(mut self, component: Component) -> Self {
        self.zerv.schema.extra_core.push(component);
        self
    }

    /// Set build components
    pub fn with_build(mut self, components: Vec<Component>) -> Self {
        self.zerv.schema.build = components;
        self
    }

    /// Add single build component
    pub fn add_build(mut self, component: Component) -> Self {
        self.zerv.schema.build.push(component);
        self
    }
}
```

#### 1.5 Add Chainable VCS Methods

```rust
impl ZervFixture {
    /// Set VCS data
    pub fn with_vcs_data(
        mut self,
        distance: u64,
        dirty: bool,
        bumped_branch: String,
        bumped_commit_hash: String,
        last_commit_hash: String,
        last_timestamp: u64,
        last_branch: String,
    ) -> Self {
        self.zerv.vars.distance = Some(distance);
        self.zerv.vars.dirty = Some(dirty);
        self.zerv.vars.bumped_branch = Some(bumped_branch);
        self.zerv.vars.bumped_commit_hash = Some(bumped_commit_hash);
        self.zerv.vars.last_commit_hash = Some(last_commit_hash);
        self.zerv.vars.last_timestamp = Some(last_timestamp);
        self.zerv.vars.last_branch = Some(last_branch);
        self
    }

    /// Set distance
    pub fn with_distance(mut self, distance: u64) -> Self {
        self.zerv.vars.distance = Some(distance);
        self
    }

    /// Set dirty flag
    pub fn with_dirty(mut self, dirty: bool) -> Self {
        self.zerv.vars.dirty = Some(dirty);
        self
    }
}
```

### Phase 2: Remove Redundant Methods

#### 2.1 Remove Legacy Static Methods

**Remove these methods:**

- `base_zerv()` → Use `ZervFixture::new().build()`
- `zerv_1_0_0_with_pre_release()` → Use `ZervFixture::new().with_pre_release().build()`
- `zerv_1_0_0_with_build()` → Use `ZervFixture::new().with_build().build()`
- `zerv_1_0_0_with_epoch()` → Use `ZervFixture::new().with_epoch().build()`
- `zerv_1_0_0_with_post()` → Use `ZervFixture::new().with_post().build()`
- `zerv_1_0_0_with_dev()` → Use `ZervFixture::new().with_dev().build()`
- `zerv_version()` → Use `ZervFixture::new().with_version().build()`

#### 2.2 Remove Specialized PEP440/SemVer Methods

**Remove these methods:**

- `pep_zerv_1_2_3()` → Use `ZervFixture::new().with_version(1, 2, 3).build()`
- `pep_zerv_1_2_3_epoch_2()` → Use `ZervFixture::new().with_version(1, 2, 3).with_epoch(2).build()`
- `pep_zerv_1_2_3_alpha_1()` → Use `ZervFixture::new().with_version(1, 2, 3).with_pre_release(PreReleaseLabel::Alpha, Some(1)).build()`
- `sem_zerv_1_2_3()` → Use `ZervFixture::new().with_version(1, 2, 3).build()`
- `sem_zerv_1_0_0_alpha_1()` → Use `ZervFixture::new().with_pre_release(PreReleaseLabel::Alpha, Some(1)).build()`

#### 2.3 Simplify Complex Methods

**Replace:**

```rust
pub fn with_all_components(
    epoch: u64,
    label: PreReleaseLabel,
    number: Option<u64>,
    post: u64,
    dev: u64,
) -> Self
```

**With:**

```rust
// Use chaining instead
ZervFixture::new()
    .with_epoch(epoch)
    .with_pre_release(label, number)
    .with_post(post)
    .with_dev(dev)
```

### Phase 3: Add Preset Methods

#### 3.1 Add Common Presets

```rust
impl ZervFixture {
    /// Create SemVer preset (basic 1.0.0)
    pub fn semver() -> Self {
        Self::new() // Already creates 1.0.0
    }

    /// Create PEP440 preset (basic 1.0.0)
    pub fn pep440() -> Self {
        Self::new() // Already creates 1.0.0
    }

    /// Create preset with all components
    pub fn with_all_components() -> Self {
        Self::new()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(1)
            .with_dev(1)
    }
}
```

### Phase 4: Update Tests

#### 4.1 Update Test Usage Patterns

**Before:**

```rust
let zerv = ZervFixture::pep_zerv_1_2_3_alpha_1();
```

**After:**

```rust
let zerv = ZervFixture::new()
    .with_version(1, 2, 3)
    .with_pre_release(PreReleaseLabel::Alpha, Some(1))
    .build();
```

**Before:**

```rust
let zerv = ZervFixture::with_all_components(2, PreReleaseLabel::Beta, Some(3), 1, 2);
```

**After:**

```rust
let zerv = ZervFixture::new()
    .with_epoch(2)
    .with_pre_release(PreReleaseLabel::Beta, Some(3))
    .with_post(1)
    .with_dev(2)
    .build();
```

## Benefits of New Design

### 1. Consistency

- Follows the same pattern as VersionArgsFixture
- Consistent method naming and behavior
- Single way to do things

### 2. Maintainability

- Fewer methods to maintain
- Clear separation of concerns
- Easy to extend with new components

### 3. Usability

- Chainable API is more intuitive
- Self-documenting method names
- Flexible composition

### 4. Testability

- Easy to create complex test scenarios
- Clear test setup code
- Reusable patterns

## Migration Strategy

### Phase 1: Add New Methods (Non-breaking)

- Add new chainable methods alongside existing ones
- Update documentation to prefer new methods

### Phase 2: Update Tests (Non-breaking)

- Update test files to use new methods
- Keep old methods for backward compatibility

### Phase 3: Deprecate Old Methods (Breaking)

- Mark old methods as deprecated
- Add deprecation warnings

### Phase 4: Remove Old Methods (Breaking)

- Remove deprecated methods
- Clean up unused code

## Example Usage

### Simple Version

```rust
let zerv = ZervFixture::new()
    .with_version(2, 1, 0)
    .build();
```

### Complex Version

```rust
let zerv = ZervFixture::new()
    .with_version(1, 2, 3)
    .with_epoch(2)
    .with_pre_release(PreReleaseLabel::Alpha, Some(1))
    .with_post(1)
    .with_dev(1)
    .with_build(vec![
        Component::String("ubuntu".to_string()),
        Component::Integer(20),
        Component::Integer(4),
    ])
    .build();
```

### VCS Data

```rust
let zerv = ZervFixture::new()
    .with_version(1, 0, 0)
    .with_distance(5)
    .with_dirty(true)
    .build();
```

## Files to Modify

### Phase 0: Foundation

- `src/test_utils/zerv/schema.rs` - ZervSchemaFixture redesign (use presets first, keep advanced methods)
- `src/test_utils/zerv/vars.rs` - ZervVarsFixture redesign (add new() + build())

### Phase 1: Core Implementation

- `src/test_utils/zerv/zerv.rs` - Main ZervFixture implementation

### Test Files (Update usage patterns)

- All test files that use ZervFixture methods
- Update to use new chainable API

### Documentation

- Update examples in comments
- Update README if it references ZervFixture

## Success Criteria

1. **API Consistency**: ZervFixture follows the same pattern as VersionArgsFixture
2. **Method Count**: Reduce total number of methods by at least 50%
3. **Chainability**: All methods return `Self` for chaining
4. **Test Coverage**: All existing functionality works with new API
5. **Documentation**: Clear examples of new usage patterns

## Risk Mitigation

### Breaking Changes

- **Risk**: Existing tests will break
- **Mitigation**: Update tests incrementally, keep old methods during transition

### Complexity

- **Risk**: New API might be too complex
- **Mitigation**: Start with simple cases, add complexity gradually

### Performance

- **Risk**: Chaining might impact performance
- **Mitigation**: Benchmark critical paths, optimize if needed
