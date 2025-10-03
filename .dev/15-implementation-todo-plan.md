# Implementation Todo Plan: Semantic Versioning Bump Behavior

## Overview

This document outlines the implementation plan to transform the current additive-only bump behavior into proper semantic versioning behavior with component precedence hierarchy.

## Current State vs Target State

### Current State (Additive Only)

- `1.2.3 + --bump-major` → `2.2.3` ❌
- `1.2.3 + --bump-minor` → `1.3.3` ❌
- No reset behavior for any components
- No precedence hierarchy

### Target State (Semantic Versioning)

- `1.2.3 + --bump-major` → `2.0.0` ✅
- `1.2.3 + --bump-minor` → `1.3.0` ✅
- Component precedence: Epoch → Major → Minor → Patch → Pre-release → Post → Dev
- Higher precedence bumps reset lower precedence components

## Implementation Phases

### Phase 1: Core Infrastructure Changes ✅

#### 1.1 Update Component Precedence System ✅

**Priority**: High
**Effort**: Medium

**Tasks:**

- [x] Add precedence hierarchy to ZervVars module
- [x] Update bump processing to respect precedence order
- [x] Add tests for precedence behavior

**Code Examples:**

**Create `src/version/zerv/bump/reset.rs`:**

```rust
use crate::version::zerv::vars::ZervVars;
use crate::version::zerv::bump::types::BumpType;
use crate::constants::{bump_types, pre_release_labels};

impl ZervVars {
    /// Reset all components with lower precedence than the given component
    pub fn reset_lower_precedence_components(&mut self, component: &str) -> Result<(), ZervError> {
        let current_precedence = BumpType::precedence_from_str(component);

        // Loop through all bump types in precedence order and reset those with lower precedence
        for bump_type in BumpType::all_in_precedence_order() {
            if bump_type.precedence() > current_precedence {
                self.reset_component(bump_type);
            }
        }

        Ok(())
    }

    /// Reset a specific component based on its bump type
    fn reset_component(&mut self, bump_type: &BumpType) {
        match bump_type {
            BumpType::Major => self.major = Some(0),
            BumpType::Minor => self.minor = Some(0),
            BumpType::Patch => self.patch = Some(0),
            BumpType::PreReleaseLabel => self.pre_release = None,
            BumpType::PreReleaseNum => {
                if let Some(ref mut pre_release) = self.pre_release {
                    pre_release.num = Some(0);
                }
            },
            BumpType::Post => self.post = None,
            BumpType::Dev => self.dev = None,
            BumpType::Epoch => self.epoch = Some(0),
        }
    }
}
```

**Create `src/version/zerv/bump/types.rs`:**

```rust
use crate::constants::{shared_constants, bump_types};
use crate::version::zerv::core::PreReleaseLabel;

/// Enum for bump types - stores increment value and label
/// This defines the core bump operations and their precedence
#[derive(Debug, Clone, PartialEq)]
pub enum BumpType {
    Epoch(u64),
    Major(u64),
    Minor(u64),
    Patch(u64),
    PreReleaseLabel(PreReleaseLabel),  // NEW: Separate pre-release label bump
    PreReleaseNum(u64),                // NEW: Separate pre-release number bump
    Post(u64),
    Dev(u64),
    // REMOVE: Distance,
}

impl BumpType {
    /// Get all bump types in precedence order (highest to lowest)
    pub fn all_in_precedence_order() -> &'static [BumpType] {
        &[
            BumpType::Epoch(0),           // 0 - highest precedence
            BumpType::Major(0),           // 1
            BumpType::Minor(0),           // 2
            BumpType::Patch(0),           // 3
            BumpType::PreReleaseLabel(PreReleaseLabel::Alpha), // 4 - pre-release label before number
            BumpType::PreReleaseNum(0),   // 5 - pre-release number after label
            BumpType::Post(0),            // 6
            BumpType::Dev(0),             // 7 - lowest precedence
        ]
    }

    /// Get precedence level for this bump type (higher = more precedence)
    pub fn precedence(&self) -> usize {
        match self {
            BumpType::Epoch(_) => 0,           // highest precedence
            BumpType::Major(_) => 1,
            BumpType::Minor(_) => 2,
            BumpType::Patch(_) => 3,
            BumpType::PreReleaseLabel(_) => 4, // pre-release label before number
            BumpType::PreReleaseNum(_) => 5,   // pre-release number after label
            BumpType::Post(_) => 6,
            BumpType::Dev(_) => 7,             // lowest precedence
        }
    }

    /// Get the field name constant for this bump type
    pub fn field_name(&self) -> &'static str {
        match self {
            BumpType::Major(_) => shared_constants::MAJOR,
            BumpType::Minor(_) => shared_constants::MINOR,
            BumpType::Patch(_) => shared_constants::PATCH,
            BumpType::PreReleaseLabel(_) => bump_types::PRE_RELEASE_LABEL,
            BumpType::PreReleaseNum(_) => shared_constants::PRE_RELEASE,
            BumpType::Post(_) => shared_constants::POST,
            BumpType::Dev(_) => shared_constants::DEV,
            BumpType::Epoch(_) => shared_constants::EPOCH,
        }
    }

    /// Get precedence level from component string
    pub fn precedence_from_str(component: &str) -> usize {
        match component {
            bump_types::EPOCH => 0,
            bump_types::MAJOR => 1,
            bump_types::MINOR => 2,
            bump_types::PATCH => 3,
            bump_types::PRE_RELEASE_LABEL => 4,
            bump_types::PRE_RELEASE_NUM => 5,
            bump_types::POST => 6,
            bump_types::DEV => 7,
            _ => 0, // Default to highest precedence for unknown components
        }
    }

    /// Get increment value (for numeric bumps)
    pub fn increment(&self) -> u64 {
        match self {
            BumpType::Epoch(increment) => *increment,
            BumpType::Major(increment) => *increment,
            BumpType::Minor(increment) => *increment,
            BumpType::Patch(increment) => *increment,
            BumpType::PreReleaseNum(increment) => *increment,
            BumpType::Post(increment) => *increment,
            BumpType::Dev(increment) => *increment,
            BumpType::PreReleaseLabel(_) => 1, // Default increment for label bumps
        }
    }

    /// Get label value (for pre-release label bumps)
    pub fn label(&self) -> Option<&PreReleaseLabel> {
        match self {
            BumpType::PreReleaseLabel(label) => Some(label),
            _ => None,
        }
    }
}
```

**Update `src/version/zerv/bump/mod.rs`:**

```rust
use crate::version::zerv::vars::ZervVars;
use crate::version::zerv::bump::types::BumpType;
use crate::version::zerv::core::PreReleaseLabel;

// Export types module
pub mod types;

impl Zerv {
    pub fn apply_bumps(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Collect all specified bumps
        let mut bumps = self.collect_bumps(args);

        // Sort bumps by precedence (highest first)
        bumps.sort_by_key(|bump_type| bump_type.precedence());

        // Process bumps in precedence order
        for bump_type in bumps {
            self.execute_bump(bump_type)?;
        }

        // Update bumped_timestamp based on dirty state
        self.bump_bumped_timestamp()?;

        Ok(())
    }

    /// Collect all specified bumps
    fn collect_bumps(&self, args: &VersionArgs) -> Vec<BumpType> {
        let mut bumps = Vec::new();

        if let Some(Some(increment)) = args.bump_epoch {
            bumps.push(BumpType::Epoch(increment as u64));
        }
        if let Some(Some(increment)) = args.bump_major {
            bumps.push(BumpType::Major(increment as u64));
        }
        if let Some(Some(increment)) = args.bump_minor {
            bumps.push(BumpType::Minor(increment as u64));
        }
        if let Some(Some(increment)) = args.bump_patch {
            bumps.push(BumpType::Patch(increment as u64));
        }
        if let Some(ref label) = args.bump_pre_release_label {
            if let Ok(pre_release_label) = PreReleaseLabel::from_str(label) {
                bumps.push(BumpType::PreReleaseLabel(pre_release_label));
            }
        }
        if let Some(Some(increment)) = args.bump_pre_release_num {
            bumps.push(BumpType::PreReleaseNum(increment as u64));
        }
        if let Some(Some(increment)) = args.bump_post {
            bumps.push(BumpType::Post(increment as u64));
        }
        if let Some(Some(increment)) = args.bump_dev {
            bumps.push(BumpType::Dev(increment as u64));
        }

        bumps
    }

    /// Execute a bump based on type
    fn execute_bump(&mut self, bump_type: BumpType) -> Result<(), ZervError> {
        match bump_type {
            BumpType::Epoch(increment) => self.bump_epoch(increment),
            BumpType::Major(increment) => self.bump_major(increment),
            BumpType::Minor(increment) => self.bump_minor(increment),
            BumpType::Patch(increment) => self.bump_patch(increment),
            BumpType::PreReleaseLabel(label) => self.bump_pre_release_label(&label.as_str()),
            BumpType::PreReleaseNum(increment) => self.bump_pre_release(increment),
            BumpType::Post(increment) => self.bump_post(increment),
            BumpType::Dev(increment) => self.bump_dev(increment),
        }
    }

// Export reset module
pub mod reset;
```

**Files to modify:**

- `src/version/zerv/bump/reset.rs` - Add precedence hierarchy and reset logic
- `src/version/zerv/bump/mod.rs` - Update BumpProcessor to use reset module
- `src/version/zerv/bump/vars_primary.rs` - Update version component bumps with reset logic
- `src/version/zerv/bump/vars_secondary.rs` - Update pre-release bumps with reset logic

#### 1.2 Remove `--bump-distance` Flag ✅

**Priority**: High
**Effort**: Low

**Tasks:**

- [x] Remove `bump_distance` field from CLI args
- [x] Remove distance bump logic from bump modules
- [x] Update tests to remove distance bump scenarios
- [x] Update documentation

**Code Examples:**

**Remove from `src/cli/version/args.rs`:**

```rust
// REMOVE this field:
#[arg(long)]
pub bump_distance: Option<u32>,
```

**Remove from `src/version/zerv/bump/vars_secondary.rs`:**

```rust
// REMOVE this method:
pub fn bump_distance(&mut self, increment: u32) -> Result<(), ZervError> {
    // ... implementation
}
```

**Remove `src/test_utils/bump_type.rs`:**

```rust
// REMOVE: This file is no longer needed
// BumpType has been moved to src/version/zerv/bump/types.rs
```

**Files to modify:**

- `src/version/zerv/bump/types.rs` - Create new BumpType module (moved from test_utils)
- `src/version/zerv/bump/mod.rs` - Export types module and update imports
- `src/version/zerv/bump/reset.rs` - Update import to use new BumpType location
- `src/cli/version/args.rs` - Remove `bump_distance` field
- `src/version/zerv/bump/vars_secondary.rs` - Remove `bump_distance` method
- `src/test_utils/bump_type.rs` - REMOVE: File no longer needed
- All test files - Update imports to use `crate::version::zerv::bump::types::BumpType`

#### 1.3 Refactor Bump Methods to Process Methods ✅

**Priority**: High
**Effort**: High
**Status**: COMPLETED

**Tasks:**

- [x] Rename `bump_*` methods to `process_*` methods
- [x] Add override logic to each `process_*` method
- [x] Integrate reset logic with bump operations (atomic)
- [x] Update main processing loop to use component-by-component processing
- [x] Remove override processing from CLI layer
- [x] Update tests to use new method names

**Code Examples:**

**Update `src/version/zerv/bump/vars_primary.rs`:**

```rust
impl ZervVars {
    // OLD: pub fn bump_major(&mut self, increment: u32) -> Result<(), ZervError>
    // NEW: pub fn process_major(&mut self, args: &VersionArgs) -> Result<(), ZervError>

    pub fn process_major(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(override_value) = args.override_major {
            self.major = Some(override_value);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_major {
            self.major = Some(self.major.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(bump_types::MAJOR)?;
        }

        Ok(())
    }

    pub fn process_minor(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(override_value) = args.override_minor {
            self.minor = Some(override_value);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_minor {
            self.minor = Some(self.minor.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(bump_types::MINOR)?;
        }

        Ok(())
    }

    pub fn process_patch(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(override_value) = args.override_patch {
            self.patch = Some(override_value);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_patch {
            self.patch = Some(self.patch.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(bump_types::PATCH)?;
        }

        Ok(())
    }
}
```

**Update `src/version/zerv/bump/vars_secondary.rs`:**

```rust
impl ZervVars {
    pub fn process_pre_release_label(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(ref override_label) = args.override_pre_release_label {
            if let Ok(label) = PreReleaseLabel::from_str(override_label) {
                self.pre_release = Some(PreReleaseVar {
                    label,
                    number: Some(0),
                });
            }
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(ref bump_label) = args.bump_pre_release_label {
            if let Ok(label) = PreReleaseLabel::from_str(bump_label) {
                self.bump_pre_release_label(bump_label)?;
                self.reset_lower_precedence_components(bump_types::PRE_RELEASE_LABEL)?;
            }
        }

        Ok(())
    }

    pub fn process_pre_release_num(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(override_value) = args.override_pre_release_num {
            if self.pre_release.is_none() {
                self.pre_release = Some(PreReleaseVar {
                    label: PreReleaseLabel::Alpha,
                    number: Some(0),
                });
            }
            if let Some(ref mut pre_release) = self.pre_release {
                pre_release.number = Some(override_value);
            }
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_pre_release_num {
            self.bump_pre_release_num(increment as u64)?;
            self.reset_lower_precedence_components(bump_types::PRE_RELEASE_NUM)?;
        }

        Ok(())
    }

    pub fn process_post(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(override_value) = args.override_post {
            self.post = Some(override_value);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_post {
            self.post = Some(self.post.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(bump_types::POST)?;
        }

        Ok(())
    }

    pub fn process_dev(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(override_value) = args.override_dev {
            self.dev = Some(override_value);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_dev {
            self.dev = Some(self.dev.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(bump_types::DEV)?;
        }

        Ok(())
    }
}
```

**Update `src/version/zerv/bump/vars_timestamp.rs`:**

```rust
impl ZervVars {
    pub fn process_epoch(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(override_value) = args.override_epoch {
            self.epoch = Some(override_value);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_epoch {
            self.epoch = Some(self.epoch.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(bump_types::EPOCH)?;
        }

        Ok(())
    }
}
```

**Update `src/version/zerv/bump/mod.rs`:**

```rust
use crate::version::zerv::vars::ZervVars;
use crate::version::zerv::bump::types::BumpType;

// Export types module
pub mod types;

impl Zerv {
    pub fn apply_bumps(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Process components in precedence order (component-by-component processing)
        self.vars.process_epoch(args)?;
        self.vars.process_major(args)?;
        self.vars.process_minor(args)?;
        self.vars.process_patch(args)?;
        self.vars.process_pre_release_label(args)?;
        self.vars.process_pre_release_num(args)?;
        self.vars.process_post(args)?;
        self.vars.process_dev(args)?;

        // Update bumped_timestamp based on dirty state
        self.bump_bumped_timestamp()?;

        Ok(())
    }
}

// Export reset module
pub mod reset;
```

**Files to modify:**

- `src/version/zerv/bump/vars_primary.rs` - Rename `bump_*` to `process_*` and add override logic
- `src/version/zerv/bump/vars_secondary.rs` - Same for secondary components
- `src/version/zerv/bump/vars_timestamp.rs` - Same for timestamp components
- `src/version/zerv/bump/mod.rs` - Update main processing loop
- `src/cli/version/args.rs` - Remove override processing (move to ZervVars)
- All test files - Update method calls to use `process_*` methods

### Phase 2: Pre-release Component Enhancements ✅

#### 2.1 Add `--bump-pre-release-label` Flag ✅

**Priority**: High
**Effort**: Medium

**Tasks:**

- [ ] Update existing `--pre-release-label` flag to use validation
- [ ] Add `bump_pre_release_label` field to CLI args
- [ ] Implement pre-release label bump logic with reset behavior
- [ ] Add validation for conflicting pre-release flags
- [ ] Integrate `validate_pre_release_flags()` into existing `validate()` method
- [ ] **Migration**: Replace `normalize_pre_release_label()` with `PreReleaseLabel::try_from_str()`
- [ ] **Migration**: Replace `normalize_pre_label()` with `PreReleaseLabel::from_str_or_alpha()`
- [ ] **Migration**: Update all imports to use new unified methods
- [ ] **Migration**: Remove old redundant functions after migration complete
- [ ] Add tests for pre-release label bumps

**Code Examples:**

**Add to `src/constants.rs`:**

```rust
// Pre-release label constants
pub mod pre_release_labels {
    pub const ALPHA: &str = "alpha";
    pub const BETA: &str = "beta";
    pub const RC: &str = "rc";

    pub const VALID_LABELS: &[&str] = &[ALPHA, BETA, RC];
}

// Rename shared_fields to shared_constants
pub mod shared_constants {
    pub const EPOCH: &str = "epoch";
    pub const MAJOR: &str = "major";
    pub const MINOR: &str = "minor";
    pub const PATCH: &str = "patch";
    pub const PRE_RELEASE: &str = "pre_release";
    pub const POST: &str = "post";
    pub const DEV: &str = "dev";
    pub const DISTANCE: &str = "distance";
    // ... other existing fields
}

// Bump type field constants - defined from shared_constants
pub mod bump_types {
    use super::shared_constants;

    pub const EPOCH: &str = shared_constants::EPOCH;
    pub const MAJOR: &str = shared_constants::MAJOR;
    pub const MINOR: &str = shared_constants::MINOR;
    pub const PATCH: &str = shared_constants::PATCH;
    pub const PRE_RELEASE_LABEL: &str = "pre_release_label";
    pub const PRE_RELEASE_NUM: &str = shared_constants::PRE_RELEASE;
    pub const POST: &str = shared_constants::POST;
    pub const DEV: &str = shared_constants::DEV;
}
```

**Add to `src/version/zerv/core.rs`:**

```rust
use crate::constants::pre_release_labels;

impl PreReleaseLabel {
    /// Get string representation of the label
    pub fn as_str(&self) -> &'static str {
        match self {
            PreReleaseLabel::Alpha => pre_release_labels::ALPHA,
            PreReleaseLabel::Beta => pre_release_labels::BETA,
            PreReleaseLabel::Rc => pre_release_labels::RC,
        }
    }

    /// Get all valid label strings
    pub fn valid_labels() -> &'static [&'static str] {
        pre_release_labels::VALID_LABELS
    }

    /// Convert string to PreReleaseLabel enum (strict validation)
    pub fn from_str(label: &str) -> Result<Self, ZervError> {
        match label {
            pre_release_labels::ALPHA => Ok(PreReleaseLabel::Alpha),
            pre_release_labels::BETA => Ok(PreReleaseLabel::Beta),
            pre_release_labels::RC => Ok(PreReleaseLabel::Rc),
            _ => Err(ZervError::InvalidPreReleaseLabel(format!(
                "Invalid pre-release label '{}'. Valid labels: {:?}",
                label, pre_release_labels::VALID_LABELS
            ))),
        }
    }

    /// Flexible parsing with alternative forms
    /// This replaces the existing normalize_pre_release_label function
    pub fn try_from_str(label: &str) -> Option<Self> {
        match label.to_lowercase().as_str() {
            pre_release_labels::ALPHA | "a" => Some(PreReleaseLabel::Alpha),
            pre_release_labels::BETA | "b" => Some(PreReleaseLabel::Beta),
            pre_release_labels::RC | "c" | "preview" | "pre" => Some(PreReleaseLabel::Rc),
            _ => None,
        }
    }

    /// Flexible parsing with alpha fallback (for PEP440 parser compatibility)
    /// This replaces the existing normalize_pre_label function
    pub fn from_str_or_alpha(label: &str) -> Self {
        Self::try_from_str(label).unwrap_or(PreReleaseLabel::Alpha)
    }
}
```

**Add to `src/cli/version/args.rs`:**

```rust
use crate::version::zerv::core::PreReleaseLabel;

#[derive(Parser)]
pub struct VersionArgs {
    // ... existing fields ...

    // Update existing pre_release_label to use validation
    #[arg(long, value_parser = PreReleaseLabel::valid_labels(),
          help = "Override pre-release label (alpha, beta, rc)")]
    pub pre_release_label: Option<String>,

    #[arg(long, value_parser = PreReleaseLabel::valid_labels())]
    pub bump_pre_release_label: Option<String>,

    // Add validation method (integrate into existing validate method)
    pub fn validate_pre_release_flags(&self) -> Result<(), ZervError> {
        if self.pre_release_label.is_some() && self.bump_pre_release_label.is_some() {
            return Err(ZervError::ConflictingFlags(
                "Cannot use --pre-release-label with --bump-pre-release-label".to_string()
            ));
        }
        Ok(())
    }

    // Note: Call validate_pre_release_flags() from existing validate() method
}
```

**Add to `src/version/zerv/bump/vars_secondary.rs`:**

```rust
use crate::version::zerv::core::PreReleaseLabel;

impl ZervVars {
    pub fn bump_pre_release_label(&mut self, label: &str) -> Result<(), ZervError> {
        // Convert string to enum using the new from_str method
        let pre_release_label = PreReleaseLabel::from_str(label)?;

        // Reset post/dev components (lower precedence)
        self.post = None;
        self.dev = None;

        // Set pre-release label and reset number to 0
        self.pre_release = Some(PreRelease {
            label: pre_release_label,
            num: 0,
        });

        Ok(())
    }
}
```

**Add to `src/version/zerv/bump/mod.rs`:**

```rust
pub fn process_bump_pre_release_label(
    vars: &mut ZervVars,
    label: &str
) -> Result<(), ZervError> {
    vars.bump_pre_release_label(label)?;
    Ok(())
}
```

**Files to modify:**

- `src/constants.rs` - Add pre_release_labels constants module
- `src/version/zerv/core.rs` - Add unified methods to PreReleaseLabel
- `src/cli/version/args.rs` - Add bump_pre_release_label flag with validation
- `src/version/zerv/bump/vars_secondary.rs` - Add bump_pre_release_label method using PreReleaseLabel::from_str
- `src/version/zerv/bump/mod.rs` - Add processing function
- `src/error.rs` - Add InvalidPreReleaseLabel error variant

**Migration Examples:**

**Replace in `src/version/zerv/vars.rs`:**

```rust
// OLD
label: normalize_pre_release_label(label).ok_or_else(|| {
    ZervError::InvalidVersion(format!("Invalid pre-release label: {label}"))
})?,

// NEW
label: PreReleaseLabel::try_from_str(label).ok_or_else(|| {
    ZervError::InvalidVersion(format!("Invalid pre-release label: {label}"))
})?,
```

**Replace in `src/version/semver/to_zerv.rs`:**

```rust
// OLD
if let Some(normalized_label) = normalize_pre_release_label(label)

// NEW
if let Some(normalized_label) = PreReleaseLabel::try_from_str(label)
```

**Replace in `src/version/pep440/parser.rs`:**

```rust
// OLD
let label = normalize_pre_label(pre_l.as_str());

// NEW
let label = PreReleaseLabel::from_str_or_alpha(pre_l.as_str());
```

**Remove redundant functions:**

- `src/version/zerv/utils/general.rs` - Remove `normalize_pre_release_label()`
- `src/version/pep440/parser.rs` - Remove `normalize_pre_label()`

**Migration Strategy:**

1. **Phase 1**: Add new `PreReleaseLabel` methods alongside existing functions
2. **Phase 2**: Update all call sites to use new methods
3. **Phase 3**: Remove old functions after all call sites are updated
4. **Phase 4**: Update imports and clean up unused code

**Migration Benefits:**

- **Unified API**: All pre-release label logic in one place
- **Type Safety**: Methods return proper enum types
- **Consistency**: Same patterns across codebase
- **Maintainability**: Single source of truth for label handling

#### 2.2 Update Pre-release Creation Logic ✅

**Priority**: High
**Effort**: Low

**Tasks:**

- [ ] Update `--bump-pre-release-num` to create alpha label if none exists
- [ ] Add pre-release label validation (SemVer spec compliance)
- [ ] Update error handling for pre-release operations

**Files to modify:**

- `src/version/zerv/bump/vars_secondary.rs` - Update `bump_pre_release` method
- `src/error.rs` - Add validation error types

### Phase 3: Reset Behavior Implementation ✅

#### 3.1 Implement Version Component Reset Logic ✅

**Priority**: High
**Effort**: High

**Tasks:**

- [ ] Update major bumps to reset minor, patch, pre-release, post, dev
- [ ] Update minor bumps to reset patch, pre-release, post, dev
- [ ] Update patch bumps to reset pre-release, post, dev
- [ ] Add comprehensive tests for reset behavior

**Code Examples:**

**Note: After Phase 1.2, all bump*\* methods are replaced with process*\* methods**
**The process\_\* methods already handle override + bump + reset logic atomically**

**Example of what process_major looks like (already implemented in Phase 1.2):**

```rust
impl ZervVars {
    pub fn process_major(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(override_value) = args.override_major {
            self.major = Some(override_value);
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_major {
            self.major = Some(self.major.unwrap_or(0) + increment as u64);
            self.reset_lower_precedence_components(bump_types::MAJOR)?;
        }

        Ok(())
    }

    // Similar for process_minor, process_patch, etc.
}
```

**Note: After Phase 1.2, all bump*\* methods are replaced with process*\* methods**
**The process\_\* methods already handle override + bump + reset logic atomically**

**Example of what process_pre_release_num looks like (already implemented in Phase 1.2):**

```rust
impl ZervVars {
    pub fn process_pre_release_num(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // 1. Override step
        if let Some(override_value) = args.override_pre_release_num {
            if self.pre_release.is_none() {
                self.pre_release = Some(PreReleaseVar {
                    label: PreReleaseLabel::Alpha,
                    number: Some(0),
                });
            }
            if let Some(ref mut pre_release) = self.pre_release {
                pre_release.number = Some(override_value);
            }
        }

        // 2. Bump + Reset step (atomic operation)
        if let Some(Some(increment)) = args.bump_pre_release_num {
            // Create alpha label if none exists
            if self.pre_release.is_none() {
                self.pre_release = Some(PreReleaseVar {
                    label: PreReleaseLabel::Alpha,
                    number: Some(0),
                });
            }

            // Increment pre-release number
            if let Some(ref mut pre_release) = self.pre_release {
                pre_release.number = Some(pre_release.number.unwrap_or(0) + increment as u64);
            }

            self.reset_lower_precedence_components(bump_types::PRE_RELEASE_NUM)?;
        }

        Ok(())
    }
}
```

**Add test examples:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_major_with_bump_resets_lower_components() {
        let mut vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            pre_release: Some(PreRelease { label: "alpha".to_string(), num: 1 }),
            post: Some(5),
            dev: Some(10),
            ..Default::default()
        };

        let args = VersionArgs {
            bump_major: Some(Some(1)),
            ..Default::default()
        };

        vars.process_major(&args).unwrap();

        assert_eq!(vars.major, Some(2));
        assert_eq!(vars.minor, Some(0));  // Reset
        assert_eq!(vars.patch, Some(0));  // Reset
        assert_eq!(vars.pre_release, None);  // Reset
        assert_eq!(vars.post, None);  // Reset
        assert_eq!(vars.dev, None);   // Reset
    }

    #[test]
    fn test_process_minor_with_bump_resets_lower_components() {
        let mut vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            pre_release: Some(PreRelease { label: "alpha".to_string(), num: 1 }),
            post: Some(5),
            dev: Some(10),
            ..Default::default()
        };

        let args = VersionArgs {
            bump_minor: Some(Some(1)),
            ..Default::default()
        };

        vars.process_minor(&args).unwrap();

        assert_eq!(vars.major, Some(1));  // Unchanged
        assert_eq!(vars.minor, Some(3));  // Incremented
        assert_eq!(vars.patch, Some(0));  // Reset
        assert_eq!(vars.pre_release, None);  // Reset
        assert_eq!(vars.post, None);  // Reset
        assert_eq!(vars.dev, None);   // Reset
    }
}
```

**Files to modify:**

- **Note**: After Phase 1.2, all files already use process\_\* methods
- Test files - Add reset behavior tests using process\_\* methods
- This phase mainly adds comprehensive testing for the reset behavior

#### 3.2 Implement Epoch Reset Logic ✅

**Priority**: Medium
**Effort**: Medium

**Tasks:**

- [ ] Update internal epoch bump methods to reset all lower precedence components
- [ ] Add epoch bump tests using process_epoch method
- [ ] Update epoch bump documentation

**Files to modify:**

- **Note**: After Phase 1.2, process_epoch method already exists and handles override + bump + reset
- Test files - Add epoch reset tests using process_epoch method
- This phase mainly adds comprehensive testing for epoch reset behavior

### Phase 4: Multiple Bump Processing

#### 4.1 Implement Explicit vs Implicit Processing

**Priority**: High
**Effort**: High

**Tasks:**

- [ ] **Note**: After Phase 1.2, component-by-component processing is already implemented
- [ ] Add comprehensive integration tests for complex mixed scenarios
- [ ] Validate that precedence order is correctly maintained
- [ ] Test edge cases with multiple bump combinations

**Code Examples:**

**Note: After Phase 1.2, the apply_bumps method already handles component-by-component processing:**

```rust
impl Zerv {
    pub fn apply_bumps(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Process components in precedence order (component-by-component processing)
        self.vars.process_epoch(args)?;
        self.vars.process_major(args)?;
        self.vars.process_minor(args)?;
        self.vars.process_patch(args)?;
        self.vars.process_pre_release_label(args)?;
        self.vars.process_pre_release_num(args)?;
        self.vars.process_post(args)?;
        self.vars.process_dev(args)?;

        // Update bumped_timestamp based on dirty state
        self.bump_bumped_timestamp()?;

        Ok(())
    }
}
```

**This phase focuses on comprehensive testing of the existing implementation.**

**Add integration test example:**

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complex_mixed_bumps_with_process_methods() {
        let mut zerv = Zerv {
            vars: ZervVars {
                major: Some(1),
                minor: Some(2),
                patch: Some(3),
                pre_release: Some(PreReleaseVar {
                    label: PreReleaseLabel::Alpha,
                    number: Some(1)
                }),
                post: Some(5),
                dev: Some(10),
                ..Default::default()
            },
            ..Default::default()
        };

        let args = VersionArgs {
            bump_major: Some(Some(1)),
            bump_minor: Some(Some(2)),
            bump_patch: Some(Some(3)),
            bump_pre_release_num: Some(Some(1)),
            bump_post: Some(Some(1)),
            bump_dev: Some(Some(1)),
            ..Default::default()
        };

        zerv.apply_bumps(&args).unwrap();

        // Expected: 1.2.3-alpha.1.post5.dev10 + bumps → 2.2.3-alpha.1.post1.dev1
        // Major resets minor/patch/pre/post/dev, then explicit bumps applied
        assert_eq!(zerv.vars.major, Some(2));  // 1 + 1
        assert_eq!(zerv.vars.minor, Some(2));  // Reset to 0, then +2
        assert_eq!(zerv.vars.patch, Some(3));  // Reset to 0, then +3
        assert_eq!(zerv.vars.pre_release, Some(PreReleaseVar {
            label: PreReleaseLabel::Alpha,
            number: Some(1)
        })); // Reset, then +1
        assert_eq!(zerv.vars.post, Some(1));   // Reset to 0, then +1
        assert_eq!(zerv.vars.dev, Some(1));    // Reset to 0, then +1
    }
}
```

**Files to modify:**

- **Note**: After Phase 1.2, the bump processing pipeline is already implemented
- Test files - Add comprehensive integration tests for complex scenarios
- This phase focuses on testing the existing component-by-component processing

#### 4.2 Add Validation for Conflicting Flags

**Priority**: Medium
**Effort**: Low

**Tasks:**

- [ ] Add early validation for conflicting pre-release flags
- [ ] Add validation for invalid combinations
- [ ] Improve error messages

**Files to modify:**

- `src/cli/version/args.rs` - Add validation logic
- `src/error.rs` - Add new error types

### Phase 5: Testing and Validation

#### 5.1 Update Existing Tests

**Priority**: High
**Effort**: Medium

**Tasks:**

- [ ] Update all existing bump tests to reflect new behavior
- [ ] Add edge case tests for precedence hierarchy
- [ ] Add tests for pre-release creation and reset behavior
- [ ] Add tests for complex mixed scenarios

**Files to modify:**

- All test files in `tests/` directory
- `src/test_utils/` - Update test utilities

#### 5.2 Add Integration Tests

**Priority**: High
**Effort**: Medium

**Tasks:**

- [ ] Add end-to-end tests for bump behavior
- [ ] Add tests for CLI argument combinations
- [ ] Add tests for error scenarios
- [ ] Add performance tests

**Files to modify:**

- `tests/integration_tests/version/` - Add comprehensive bump tests

### Phase 6: Documentation and Examples

#### 6.1 Update Documentation

**Priority**: Medium
**Effort**: Low

**Tasks:**

- [ ] Update CLI help text
- [ ] Update README with new bump behavior
- [ ] Add examples for complex scenarios
- [ ] Update error message documentation

**Files to modify:**

- `README.md`
- `src/cli/version/args.rs` - Update help text
- Documentation files

#### 6.2 Add Migration Guide

**Priority**: Low
**Effort**: Low

**Tasks:**

- [ ] Document breaking changes
- [ ] Provide migration examples
- [ ] Add deprecation warnings for old behavior

## Implementation Order

### Week 1: Core Infrastructure

1. Remove `--bump-distance` flag
2. Add precedence constants
3. Update basic bump processing logic

### Week 2: Pre-release Enhancements

1. Add `--bump-pre-release-label` flag
2. Update pre-release creation logic
3. Add pre-release validation

### Week 3: Reset Behavior

1. Implement version component reset logic
2. Implement epoch reset logic
3. Add reset behavior tests

### Week 4: Multiple Bump Processing

1. Implement explicit vs implicit processing
2. Add validation for conflicting flags
3. Add complex scenario tests

### Week 5: Testing and Documentation

1. Update all existing tests
2. Add integration tests
3. Update documentation

## Risk Mitigation

### Breaking Changes

- **Risk**: Existing scripts may break due to behavior changes
- **Mitigation**: Add deprecation warnings, provide migration guide

### Complex Edge Cases

- **Risk**: Complex bump combinations may have unexpected behavior
- **Mitigation**: Comprehensive testing, clear documentation

### Performance Impact

- **Risk**: New precedence logic may impact performance
- **Mitigation**: Benchmark tests, optimize hot paths

## Success Criteria

### Functional Requirements

- [ ] All bump operations follow semantic versioning rules
- [ ] Component precedence hierarchy is correctly implemented
- [ ] Pre-release components work as specified
- [ ] Multiple bump scenarios work correctly
- [ ] All existing functionality is preserved

### Non-Functional Requirements

- [ ] Performance is not significantly degraded
- [ ] Error messages are clear and helpful
- [ ] Documentation is complete and accurate
- [ ] Tests provide good coverage

### Quality Gates

- [ ] All tests pass
- [ ] No breaking changes without migration path
- [ ] Performance benchmarks meet requirements
- [ ] Code review approval
- [ ] Documentation review approval

## Dependencies

### External Dependencies

- None (all changes are internal to the codebase)

### Internal Dependencies

- CLI argument parsing system
- Version parsing and formatting system
- Test infrastructure
- Error handling system

## Rollback Plan

If issues arise during implementation:

1. **Immediate**: Revert to previous commit
2. **Short-term**: Disable new behavior with feature flag
3. **Long-term**: Fix issues and re-enable

## Monitoring and Metrics

### Key Metrics

- Test coverage percentage
- Performance benchmark results
- Error rate in CI/CD
- User feedback on new behavior

### Monitoring Points

- CI/CD pipeline success rate
- Test execution time
- Memory usage during bump operations
- User adoption of new features
