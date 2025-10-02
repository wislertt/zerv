# Zerv Version Command - Implementation Plan

## Overview

This document provides a comprehensive step-by-step plan to achieve the ideal version command specification outlined in `.dev/11-version-command-ideal-spec.md`. The plan is organized into logical phases with clear dependencies and testing requirements.

## Current State Analysis

### ✅ Already Implemented

- Basic CLI structure with `VersionArgs`
- VCS override functionality (limited set)
- Basic output formatting (PEP440, SemVer, Zerv RON)
- Schema system with presets (zerv-standard, zerv-calver)
- RON parsing and serialization
- Basic timestamp patterns (YYYY, MM, DD, etc.)
- Input format handling (auto, semver, pep440)
- Stdin support for Zerv RON format

### ❌ Missing from Ideal State

- **Field naming**: Uses `current_*` instead of `bumped_*` fields
- **Bump functionality**: No bump arguments (--bump-major, --bump-patch, etc.)
- **Context control**: No --bump-context/--no-bump-context flags
- **Templating**: No Handlebars templating support
- **Custom variables**: Limited custom variable support
- **RON format**: Uses old Component enum without serde attributes
- **Constants**: Extensive use of bare strings instead of constants
- **Error messages**: Generic VCS errors instead of source-specific
- **Timestamp patterns**: Missing compact_date, compact_datetime, custom formats
- **Comprehensive overrides**: Missing many override options

## Implementation Plan

### Phase 1: Core Data Structure Updates (Foundation) - [x]

#### Task 1.1: Define Comprehensive Constants - [x]

**Priority**: High | **Estimated Time**: 2-3 hours

**Goal**: Replace 280+ bare strings with type-safe constants

**Files to Update**:

- `src/constants.rs` - Add comprehensive constants
- All files using bare strings (280+ instances)

**Implementation**:

```rust
// src/constants.rs
pub mod fields {
    // Core version fields
    pub const MAJOR: &str = "major";
    pub const MINOR: &str = "minor";
    pub const PATCH: &str = "patch";
    pub const EPOCH: &str = "epoch";

    // Pre-release fields
    pub const PRE_RELEASE: &str = "pre_release";

    // Post-release fields
    pub const POST: &str = "post";
    pub const DEV: &str = "dev";

    // VCS state fields (current → bumped naming)
    pub const DISTANCE: &str = "distance";
    pub const DIRTY: &str = "dirty";
    pub const BUMPED_BRANCH: &str = "bumped_branch";
    pub const BUMPED_COMMIT_HASH: &str = "bumped_commit_hash";
    pub const BUMPED_COMMIT_HASH_SHORT: &str = "bumped_commit_hash_short";
    pub const BUMPED_TIMESTAMP: &str = "bumped_timestamp";
    pub const LAST_BRANCH: &str = "last_branch";
    pub const LAST_COMMIT_HASH: &str = "last_commit_hash";
    pub const LAST_TIMESTAMP: &str = "last_timestamp";

    // Custom fields
    pub const CUSTOM: &str = "custom";
}

pub mod timestamp_patterns {
    pub const COMPACT_DATE: &str = "compact_date";
    pub const COMPACT_DATETIME: &str = "compact_datetime";
}

pub mod sources {
    pub const GIT: &str = "git";
    pub const STDIN: &str = "stdin";
}

pub mod formats {
    pub const AUTO: &str = "auto";
    pub const SEMVER: &str = "semver";
    pub const PEP440: &str = "pep440";
    pub const ZERV: &str = "zerv";
}
```

**Testing**: Update all test files to use constants instead of bare strings

#### Task 1.2: Update ZervVars Field Structure - [x]

**Priority**: High | **Estimated Time**: 3-4 hours

**Goal**: Rename fields and add missing fields for ideal state

**Changes**:

```rust
// src/version/zerv/core.rs
pub struct ZervVars {
    // Core version fields (unchanged)
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub epoch: Option<u64>,
    pub pre_release: Option<PreReleaseVar>,
    pub post: Option<u64>,
    pub dev: Option<u64>,

    // VCS state fields (renamed and restructured)
    pub distance: Option<u64>,
    pub dirty: Option<bool>,

    // Bumped fields (for template access)
    pub bumped_branch: Option<String>,
    pub bumped_commit_hash: Option<String>,
    pub bumped_commit_hash_short: Option<String>,
    pub bumped_timestamp: Option<u64>,

    // Last version fields (for template access)
    pub last_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_timestamp: Option<u64>,

    // Custom variables (changed to nested JSON)
    pub custom: serde_json::Value,
}
```

**Implementation Strategy**:

1. Directly replace old field names with new ones
2. Update all field access to use new names immediately
3. Remove old fields completely - no legacy support needed

#### Task 1.3: Update Component Enum for RON Format - [x]

**Priority**: High | **Estimated Time**: 2-3 hours

**Goal**: Update Component enum with serde attributes and remove VarCustom

**Changes**:

```rust
// src/version/zerv/core.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "str")]
    String(String),
    #[serde(rename = "int")]
    Integer(u64),
    #[serde(rename = "var")]
    VarField(String),
    #[serde(rename = "ts")]
    VarTimestamp(String),
    // Remove VarCustom - use var("custom.xxx") instead
}
```

**RON Format Migration**:

- Old: `VarField("major")` → New: `var("major")`
- Old: `VarCustom("build_id")` → New: `var("custom.build_id")`
- Old: `String("stable")` → New: `str("stable")`
- Old: `Integer(1)` → New: `int(1)`

### Phase 2: Enhanced Timestamp Support - [x]

#### Task 2.1: Add Preset Timestamp Patterns - [x]

**Priority**: Medium | **Estimated Time**: 2-3 hours

**Goal**: Add compact_date, compact_datetime and custom format support

**Implementation**:

```rust
// src/version/zerv/utils.rs
pub fn resolve_timestamp(pattern: &str, timestamp: Option<u64>) -> Result<u64> {
    // ... existing code ...

    let result = match pattern {
        // Preset patterns (exact matches first)
        constants::timestamp_patterns::COMPACT_DATE => {
            parse_timestamp_component(&dt, "%Y%m%d", "compact-date")?
        }
        constants::timestamp_patterns::COMPACT_DATETIME => {
            parse_timestamp_component(&dt, "%Y%m%d%H%M%S", "compact-datetime")?
        }

        // Single component patterns
        "YYYY" => parse_timestamp_component(&dt, "%Y", "year")?,
        "MM" => parse_timestamp_component(&dt, "%-m", "month")?,
        // ... other existing patterns

        // Custom format fallback
        _ => {
            if pattern.starts_with('%') {
                parse_timestamp_component(&dt, pattern, "custom format")
                    .map_err(|_| ZervError::InvalidFormat(format!(
                        "Invalid custom format string: {pattern}"
                    )))?
            } else {
                return Err(ZervError::InvalidFormat(format!(
                    "Unknown timestamp pattern: {pattern}. Valid patterns: YYYY, MM, DD, HH, mm, SS, compact_date, compact_datetime, or custom format starting with %"
                )));
            }
        }
    };

    Ok(result)
}
```

**Test Cases**:

```rust
#[rstest]
// Preset patterns
#[case(1710511845, "compact_date", 20240315)]
#[case(1710511845, "compact_datetime", 20240315141045)]

// Custom format strings
#[case(1710511845, "%Y%m", 202403)]
#[case(1710511845, "%Y-%m-%d", 20240315)]
#[case(1710511845, "%H:%M:%S", 141045)]
```

### Phase 3: Critical Missing Functionality - [ ]

**Architecture Principle**: Clean separation of concerns - each source (git, stdin, string) only creates a Zerv object. Bump logic and overrides are applied to the Zerv object after creation, maintaining loose coupling.

#### Task 3.1: Add Bump Arguments - [x]

**Priority**: High | **Estimated Time**: 4-5 hours

**Goal**: Add all bump and override arguments from ideal spec

**New Arguments**:

```rust
// src/cli/version.rs
pub struct VersionArgs {
    // ... existing arguments ...

    // Override Options (Absolute Values)
    pub tag_version: Option<String>,
    pub distance: Option<u32>,
    pub dirty: bool,
    pub no_dirty: bool,
    pub clean: bool,
    pub current_branch: Option<String>,
    pub commit_hash: Option<String>,
    pub post: Option<u32>,
    pub dev: Option<u32>,
    pub pre_release_label: Option<String>,
    pub pre_release_num: Option<u32>,
    pub epoch: Option<u32>,
    pub custom: Option<String>, // JSON format

    // Bump Options (Relative Modifications)
    pub bump_major: Option<Option<u32>>, // Optional value, defaults to 1
    pub bump_minor: Option<Option<u32>>,
    pub bump_patch: Option<Option<u32>>,
    pub bump_distance: Option<Option<u32>>,
    pub bump_post: Option<Option<u32>>,
    pub bump_dev: Option<Option<u32>>,
    pub bump_pre_release_num: Option<Option<u32>>,
    pub bump_epoch: Option<Option<u32>>,

    // Context Control Options
    pub bump_context: bool,
    pub no_bump_context: bool,

    // ... existing output options ...
}
```

**Implementation Strategy**:

1. Add argument definitions with proper help text
2. Implement conflict validation (--bump-context vs --no-bump-context)
3. Add processing logic in `VcsOverrideProcessor`

#### Task 3.2: Add --source stdin Support - [x]

**Priority**: CRITICAL | **Estimated Time**: 2-3 hours

**Goal**: Add --source stdin for reading Zerv RON objects from stdin

**Architecture Note**: Each source (git, stdin) should only be responsible for creating a Zerv object. Bump logic and overrides are applied to the Zerv object after creation, maintaining clean separation of concerns.

**Implementation**:

```rust
// src/cli/version/pipeline.rs - Add stdin source handling
sources::STDIN => {
    // Read Zerv RON object from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)
        .map_err(|e| ZervError::Io(io::Error::other(format!("Failed to read from stdin: {}", e))))?;

    // Parse RON input as ZervVars
    let vars: ZervVars = ron::from_str(&input)
        .map_err(|e| ZervError::InvalidVersion(format!("Failed to parse RON from stdin: {}", e)))?;

    // Create Zerv object
    let mut zerv_object = create_zerv_version(vars, args.schema.as_deref(), args.schema_ron.as_deref())?;

    // Apply overrides and bumps
    if args.has_overrides() {
        zerv_object.apply_overrides(&args)?;
    }

    if args.has_bumps() {
        zerv_object.apply_bumps(&args)?;
    }

    zerv_object
}
```

**Constants already exist**:

```rust
// src/constants.rs - Already implemented
pub mod sources {
    pub const GIT: &str = "git";
    pub const STDIN: &str = "stdin";
}
```

#### Task 3.3: Add Bump Method to Zerv Object - [x]

**Priority**: CRITICAL | **Estimated Time**: 4-5 hours

**Goal**: Add bump processing methods to Zerv object as centralized logic with dedicated module structure

**Note**: This bump logic is applied to Zerv objects after they are created from any source (git, stdin, string), maintaining clean separation of concerns.

**Module Structure**:

```
src/version/zerv/bump/
├── mod.rs           # BumpType enum and apply_bumps coordinator method
├── vars_primary.rs  # Primary ZervVars field bumps (major, minor, patch)
├── vars_secondary.rs # Secondary ZervVars field bumps (distance, post, dev, pre_release, epoch)
├── vars_timestamp.rs # Timestamp bump logic (bump_bumped_timestamp)
└── tests.rs         # Comprehensive bump tests
```

**Implementation**:

```rust
// src/version/zerv/bump/mod.rs
use super::ZervVars;
use crate::error::ZervError;
use crate::cli::version::args::VersionArgs;
use crate::constants::shared_fields;

pub mod vars_primary;
pub mod vars_secondary;
pub mod vars_timestamp;

#[cfg(test)]
mod tests;

/// Enum for bump types - uses constants for field names
#[derive(Debug, Clone, PartialEq)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
    Distance,
    Post,
    Dev,
    Epoch,
    PreRelease,
}

impl BumpType {
    /// Get the field name constant for this bump type
    pub fn field_name(&self) -> &'static str {
        match self {
            BumpType::Major => shared_fields::MAJOR,
            BumpType::Minor => shared_fields::MINOR,
            BumpType::Patch => shared_fields::PATCH,
            BumpType::Distance => shared_fields::DISTANCE,
            BumpType::Post => shared_fields::POST,
            BumpType::Dev => shared_fields::DEV,
            BumpType::Epoch => shared_fields::EPOCH,
            BumpType::PreRelease => shared_fields::PRE_RELEASE,
        }
    }
}

impl Zerv {
    /// Apply bump operations from VersionArgs
    pub fn apply_bumps(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        if let Some(Some(increment)) = args.bump_major {
            self.bump_major(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_minor {
            self.bump_minor(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_patch {
            self.bump_patch(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_distance {
            self.bump_distance(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_post {
            self.bump_post(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_dev {
            self.bump_dev(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_pre_release_num {
            self.bump_pre_release(increment as u64)?;
        }

        if let Some(Some(increment)) = args.bump_epoch {
            self.bump_epoch(increment as u64)?;
        }

        // Update bumped_timestamp based on context
        self.bump_bumped_timestamp(args)?;

        Ok(())
    }
}
```

```rust
// src/version/zerv/bump/vars_primary.rs
use super::Zerv;
use crate::error::ZervError;

impl Zerv {
    /// Bump major version by the specified increment
    pub fn bump_major(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.major = Some(self.vars.major.unwrap_or(0) + increment);
        Ok(())
    }

    /// Bump minor version by the specified increment
    pub fn bump_minor(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.minor = Some(self.vars.minor.unwrap_or(0) + increment);
        Ok(())
    }

    /// Bump patch version by the specified increment
    pub fn bump_patch(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.patch = Some(self.vars.patch.unwrap_or(0) + increment);
        Ok(())
    }
}

// src/version/zerv/bump/extra_core.rs
use super::Zerv;
use crate::error::ZervError;

impl Zerv {
    /// Bump distance by the specified increment
    pub fn bump_distance(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.distance = Some(self.vars.distance.unwrap_or(0) + increment);
        Ok(())
    }

    /// Bump post-release version by the specified increment
    pub fn bump_post(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.post = Some(self.vars.post.unwrap_or(0) + increment);
        Ok(())
    }

    /// Bump dev version by the specified increment
    pub fn bump_dev(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.dev = Some(self.vars.dev.unwrap_or(0) + increment);
        Ok(())
    }

    /// Bump pre-release number by the specified increment
    pub fn bump_pre_release(&mut self, increment: u64) -> Result<(), ZervError> {
        if let Some(ref mut pre_release) = self.vars.pre_release {
            pre_release.number = Some(pre_release.number.unwrap_or(0) + increment);
        } else {
            return Err(ZervError::InvalidVersion(
                "Cannot bump pre-release number: no pre-release exists".to_string()
            ));
        }
        Ok(())
    }

    /// Bump epoch by the specified increment
    pub fn bump_epoch(&mut self, increment: u64) -> Result<(), ZervError> {
        self.vars.epoch = Some(self.vars.epoch.unwrap_or(0) + increment);
        Ok(())
    }
}
```

```rust
// src/version/zerv/bump/vars_timestamp.rs
use super::Zerv;
use crate::cli::version::args::VersionArgs;
use crate::error::ZervError;

impl Zerv {
    /// Bump bumped_timestamp based on bump context and dirty state
    pub fn bump_bumped_timestamp(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Only update timestamp if bump context is enabled (default behavior)
        if !args.bump_context {
            return Ok(());
        }

        // bumped_timestamp should represent the timestamp of the current commit
        // If dirty, use current timestamp (uncommitted changes)
        // If clean, use the VCS commit timestamp (already set from VCS data)
        if self.vars.dirty == Some(true) {
            self.vars.bumped_timestamp = Some(chrono::Utc::now().timestamp() as u64);
        }
        // If clean, keep the existing timestamp (from VCS data)
        // This represents the timestamp of the current commit

        Ok(())
    }
}
```

**Testing Requirements**:

```rust
// src/version/zerv/bump/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::zerv::ZervFixture;
    use crate::cli::version::args::VersionArgs;
    use rstest::*;

    // Test apply_bumps method - only test the main coordinator method
    #[rstest]
    #[case("v1.0.0", vec![(BumpType::Major, 1), (BumpType::Minor, 2)], (Some(2), Some(2), Some(0)))]
    #[case("v2.5.3", vec![(BumpType::Major, 0), (BumpType::Patch, 7)], (Some(2), Some(5), Some(10)))]
    #[case("v0.0.0", vec![(BumpType::Major, 3), (BumpType::Minor, 2), (BumpType::Patch, 1)], (Some(3), Some(2), Some(1)))]
    fn test_apply_bumps_method(
        #[case] version: &str,
        #[case] bumps: Vec<(BumpType, u64)>,
        #[case] expected: (Option<u64>, Option<u64>, Option<u64>)
    ) {
        let mut zerv = ZervFixture::tagged(version).unwrap();
        let mut args = VersionArgs::default();

        // Set up args based on bump specifications
        for (bump_type, increment) in bumps {
            match bump_type {
                BumpType::Major => args.bump_major = Some(Some(increment as i64)),
                BumpType::Minor => args.bump_minor = Some(Some(increment as i64)),
                BumpType::Patch => args.bump_patch = Some(Some(increment as i64)),
                BumpType::Distance => args.bump_distance = Some(Some(increment as i64)),
                BumpType::Post => args.bump_post = Some(Some(increment as i64)),
                BumpType::Dev => args.bump_dev = Some(Some(increment as i64)),
                BumpType::Epoch => args.bump_epoch = Some(Some(increment as i64)),
                BumpType::PreRelease => args.bump_pre_release_num = Some(Some(increment as i64)),
            }
        }

        zerv.apply_bumps(&args).unwrap();

        assert_eq!(zerv.vars.major, expected.0);
        assert_eq!(zerv.vars.minor, expected.1);
        assert_eq!(zerv.vars.patch, expected.2);
    }

    // Test BumpType enum functionality
    #[test]
    fn test_bump_type_field_names() {
        assert_eq!(BumpType::Major.field_name(), "major");
        assert_eq!(BumpType::Minor.field_name(), "minor");
        assert_eq!(BumpType::Patch.field_name(), "patch");
        assert_eq!(BumpType::Distance.field_name(), "distance");
        assert_eq!(BumpType::Post.field_name(), "post");
        assert_eq!(BumpType::Dev.field_name(), "dev");
        assert_eq!(BumpType::Epoch.field_name(), "epoch");
        assert_eq!(BumpType::PreRelease.field_name(), "pre_release");
    }

    // Test bumped_timestamp logic
    #[test]
    fn test_bumped_timestamp_dirty_with_bump_context() {
        let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
        zerv.vars.dirty = Some(true);
        zerv.vars.bumped_timestamp = Some(1234567890); // Old timestamp

        let args = VersionArgs::try_parse_from(["zerv", "--bump-major", "1"]).unwrap();
        zerv.apply_bumps(&args).unwrap();

        // Should update to current timestamp when dirty (uncommitted changes)
        assert!(zerv.vars.bumped_timestamp.is_some());
        assert!(zerv.vars.bumped_timestamp.unwrap() > 1234567890);
    }

    #[test]
    fn test_bumped_timestamp_clean_with_bump_context() {
        let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
        zerv.vars.dirty = Some(false);
        zerv.vars.bumped_timestamp = Some(1234567890); // VCS commit timestamp

        let args = VersionArgs::try_parse_from(["zerv", "--bump-major", "1"]).unwrap();
        zerv.apply_bumps(&args).unwrap();

        // Should keep VCS commit timestamp when clean (represents current commit)
        assert_eq!(zerv.vars.bumped_timestamp, Some(1234567890));
    }

    #[test]
    fn test_bumped_timestamp_no_bump_context() {
        let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
        zerv.vars.dirty = Some(true);
        zerv.vars.bumped_timestamp = Some(1234567890); // Old timestamp

        let args = VersionArgs::try_parse_from(["zerv", "--no-bump-context", "--bump-major", "1"]).unwrap();
        zerv.apply_bumps(&args).unwrap();

        // Should keep existing timestamp when bump context is disabled
        assert_eq!(zerv.vars.bumped_timestamp, Some(1234567890));
    }

    #[test]
    fn test_bumped_timestamp_clean_no_bump_context() {
        let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
        zerv.vars.dirty = Some(false);
        zerv.vars.bumped_timestamp = Some(1234567890); // VCS timestamp

        let args = VersionArgs::try_parse_from(["zerv", "--no-bump-context", "--bump-major", "1"]).unwrap();
        zerv.apply_bumps(&args).unwrap();

        // Should keep existing timestamp when bump context is disabled
        assert_eq!(zerv.vars.bumped_timestamp, Some(1234567890));
    }
}
```

**Individual Module Tests**:

```rust
// src/version/zerv/bump/core.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::zerv::ZervFixture;
    use rstest::*;

    #[rstest]
    #[case("v1.0.0", 2, Some(3))]
    #[case("v2.5.3", 1, Some(3))]
    #[case("v0.0.0", 5, Some(5))]
    fn test_bump_major(#[case] version: &str, #[case] increment: u64, #[case] expected: Option<u64>) {
        let mut zerv = ZervFixture::tagged(version).unwrap();
        zerv.bump_major(increment).unwrap();
        assert_eq!(zerv.vars.major, expected);
    }

    #[rstest]
    #[case("v1.0.0", 3, Some(3))]
    #[case("v1.2.3", 1, Some(3))]
    #[case("v0.0.0", 7, Some(7))]
    fn test_bump_minor(#[case] version: &str, #[case] increment: u64, #[case] expected: Option<u64>) {
        let mut zerv = ZervFixture::tagged(version).unwrap();
        zerv.bump_minor(increment).unwrap();
        assert_eq!(zerv.vars.minor, expected);
    }

    #[rstest]
    #[case("v1.0.0", 1, Some(1))]
    #[case("v1.2.3", 5, Some(8))]
    #[case("v0.0.0", 2, Some(2))]
    fn test_bump_patch(#[case] version: &str, #[case] increment: u64, #[case] expected: Option<u64>) {
        let mut zerv = ZervFixture::tagged(version).unwrap();
        zerv.bump_patch(increment).unwrap();
        assert_eq!(zerv.vars.patch, expected);
    }

    // Edge cases for core bumps
    #[rstest]
    #[case("v1.0.0", 0, Some(1))]  // Bump by 0
    #[case("v1.0.0", 1000, Some(1001))]  // Large increment
    fn test_bump_major_edge_cases(#[case] version: &str, #[case] increment: u64, #[case] expected: Option<u64>) {
        let mut zerv = ZervFixture::tagged(version).unwrap();
        zerv.bump_major(increment).unwrap();
        assert_eq!(zerv.vars.major, expected);
    }
}

// src/version/zerv/bump/extra_core.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::zerv::ZervFixture;
    use rstest::*;

    #[rstest]
    #[case(1, 2, Some(3))]
    #[case(5, 0, Some(5))]  // Bump by 0
    #[case(0, 10, Some(10))]  // From zero
    fn test_bump_distance(#[case] initial: u64, #[case] increment: u64, #[case] expected: Option<u64>) {
        let mut zerv = ZervFixture::with_distance("v1.0.0", initial).unwrap();
        zerv.bump_distance(increment).unwrap();
        assert_eq!(zerv.vars.distance, expected);
    }

    #[rstest]
    #[case(0, 1, Some(1))]
    #[case(3, 2, Some(5))]
    #[case(10, 0, Some(10))]
    fn test_bump_post(#[case] initial: u64, #[case] increment: u64, #[case] expected: Option<u64>) {
        let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
        zerv.vars.post = Some(initial);
        zerv.bump_post(increment).unwrap();
        assert_eq!(zerv.vars.post, expected);
    }

    #[rstest]
    #[case(0, 1, Some(1))]
    #[case(5, 3, Some(8))]
    #[case(7, 0, Some(7))]
    fn test_bump_dev(#[case] initial: u64, #[case] increment: u64, #[case] expected: Option<u64>) {
        let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
        zerv.vars.dev = Some(initial);
        zerv.bump_dev(increment).unwrap();
        assert_eq!(zerv.vars.dev, expected);
    }

    #[rstest]
    #[case(0, 1, Some(1))]
    #[case(2, 3, Some(5))]
    #[case(5, 0, Some(5))]
    fn test_bump_epoch(#[case] initial: u64, #[case] increment: u64, #[case] expected: Option<u64>) {
        let mut zerv = ZervFixture::tagged("v1.0.0").unwrap();
        zerv.vars.epoch = Some(initial);
        zerv.bump_epoch(increment).unwrap();
        assert_eq!(zerv.vars.epoch, expected);
    }

    #[rstest]
    #[case("alpha", 1, 1, Some(2))]
    #[case("beta", 5, 3, Some(8))]
    #[case("rc", 0, 2, Some(2))]
    fn test_bump_pre_release(#[case] label: &str, #[case] initial: u64, #[case] increment: u64, #[case] expected: Option<u64>) {
        let mut zerv = ZervFixture::with_pre_release("v1.0.0", label, initial).unwrap();
        zerv.bump_pre_release(increment).unwrap();
        assert_eq!(zerv.vars.pre_release.as_ref().unwrap().number, expected);
    }

    #[rstest]
    #[case("v1.0.0")]  // No pre-release
    #[case("v1.0.0-alpha")]  // Pre-release without number
    fn test_bump_pre_release_errors(#[case] version: &str) {
        let mut zerv = ZervFixture::tagged(version).unwrap();
        let result = zerv.bump_pre_release(1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no pre-release exists"));
    }
}
```

#### Task 3.4: Fix Missing bumped_timestamp Population - [ ]

**Priority**: HIGH | **Estimated Time**: 30 minutes

**Goal**: Fix missing `bumped_timestamp` field population from VCS data

**Current Issue**: The `bumped_timestamp` field exists in `ZervVars` but is not being populated from `VcsData.commit_timestamp`.

**Implementation**:

```rust
// src/pipeline/vcs_data_to_zerv_vars.rs
pub fn vcs_data_to_zerv_vars(vcs_data: VcsData) -> Result<ZervVars, ZervError> {
    // Parse version from tag_version
    let version = if let Some(ref tag_version) = vcs_data.tag_version {
        parse_version_from_tag(tag_version, None).ok_or_else(|| {
            ZervError::InvalidFormat(format!("Failed to parse version from tag: {tag_version}"))
        })?
    } else {
        return Err(ZervError::NoTagsFound);
    };

    let mut vars: ZervVars = version.into();

    // VCS-specific fields
    vars.distance = Some(vcs_data.distance as u64);
    vars.bumped_branch = vcs_data.current_branch;
    vars.dirty = Some(vcs_data.is_dirty);
    vars.bumped_commit_hash = Some(vcs_data.commit_hash);
    vars.bumped_timestamp = Some(vcs_data.commit_timestamp as u64);  // ← ADD THIS LINE
    vars.last_timestamp = vcs_data.tag_timestamp.map(|t| t as u64);

    Ok(vars)
}
```

**Testing**:

```rust
#[test]
fn test_bumped_timestamp_population() {
    let vcs_data = VcsData {
        tag_version: Some("v1.0.0".to_string()),
        commit_timestamp: 1234567890,
        ..Default::default()
    };

    let vars = vcs_data_to_zerv_vars(vcs_data).unwrap();
    assert_eq!(vars.bumped_timestamp, Some(1234567890));
}
```

**Impact**: Enables template usage like `--pre-release-num={{ bumped_timestamp }}` and `--custom-build-time={{ bumped_timestamp }}`.

#### Task 3.5: Integrate Bump Functionality in Pipeline - [ ]

**Priority**: CRITICAL | **Estimated Time**: 2-3 hours

**Goal**: Integrate bump functionality into the version pipeline with new module structure

**Implementation**:

```rust
// src/cli/version/pipeline.rs
pub fn run_version_pipeline(mut args: VersionArgs) -> Result<String, ZervError> {
    // ... existing validation and source resolution ...

    // 3. Apply bumps to Zerv object (overrides handled in source processing)
    if args.has_bumps() {
        zerv_object.apply_bumps(&args)?;
    }

    // ... rest of pipeline ...
}

// src/cli/version/stdin_pipeline.rs
pub fn process_stdin_source(args: &VersionArgs) -> Result<Zerv, ZervError> {
    // Parse stdin as Zerv RON
    let mut zerv_from_stdin = InputFormatHandler::parse_stdin_to_zerv()?;

    // Apply overrides directly to ZervVars (no conversion needed)
    if args.has_overrides() {
        zerv_from_stdin.vars.apply_overrides(args)?;
    }

    // Apply bumps if specified
    if args.has_bumps() {
        zerv_from_stdin.apply_bumps(args)?;
    }

    Ok(zerv_from_stdin)
}
```

**Integration Testing**:

```rust
// tests/integration_tests/version/bump_integration.rs
use rstest::*;

#[rstest]
#[case("v1.0.0", "--bump-major", "1", "2.0.0")]
#[case("v2.5.3", "--bump-major", "2", "4.5.3")]
#[case("v0.0.0", "--bump-major", "5", "5.0.0")]
fn test_bump_integration_git_source(#[case] version: &str, #[case] flag: &str, #[case] increment: &str, #[case] expected: &str) {
    // Test bump functionality with git source
    let output = run_zerv_command(&["version", flag, increment]);
    assert!(output.contains(expected));
}

#[rstest]
#[case("v1.0.0", "--bump-minor", "2", "1.2.0")]
#[case("v1.2.3", "--bump-patch", "5", "1.2.8")]
#[case("v0.0.0", "--bump-minor", "3", "0.3.0")]
fn test_bump_integration_core_bumps(#[case] version: &str, #[case] flag: &str, #[case] increment: &str, #[case] expected: &str) {
    let output = run_zerv_command(&["version", flag, increment]);
    assert!(output.contains(expected));
}

#[rstest]
#[case("v1.0.0", "--bump-distance", "3", "distance: 3")]
#[case("v1.0.0", "--bump-post", "2", "post: 2")]
#[case("v1.0.0", "--bump-dev", "1", "dev: 1")]
#[case("v1.0.0", "--bump-epoch", "1", "epoch: 1")]
fn test_bump_integration_extra_bumps(#[case] version: &str, #[case] flag: &str, #[case] increment: &str, #[case] expected: &str) {
    let output = run_zerv_command(&["version", flag, increment]);
    assert!(output.contains(expected));
}

#[rstest]
#[case("v1.0.0", vec!["--bump-major", "1", "--bump-minor", "2"], "2.2.0")]
#[case("v2.5.3", vec!["--bump-major", "0", "--bump-patch", "7"], "2.5.10")]
#[case("v0.0.0", vec!["--bump-major", "3", "--bump-minor", "2", "--bump-patch", "1"], "3.2.1")]
fn test_bump_integration_multiple_bumps(
    #[case] version: &str,
    #[case] flags: Vec<&str>,
    #[case] expected: &str
) {
    let mut args = vec!["version"];
    args.extend(flags);
    let output = run_zerv_command(&args);
    assert!(output.contains(expected));
}

#[rstest]
#[case("v1.0.0", vec!["--bump-major", "1", "--bump-distance", "2", "--bump-post", "1"], "2.0.0")]
#[case("v0.0.0", vec!["--bump-minor", "5", "--bump-patch", "3", "--bump-epoch", "1"], "0.5.3")]
fn test_bump_integration_mixed_bumps(
    #[case] version: &str,
    #[case] flags: Vec<&str>,
    #[case] expected: &str
) {
    let mut args = vec!["version"];
    args.extend(flags);
    let output = run_zerv_command(&args);
    assert!(output.contains(expected));
}

#[rstest]
#[case("v1.0.0", "--bump-major", "0", "1.0.0")]  // Bump by 0
#[case("v1.0.0", "--bump-major", "100", "101.0.0")]  // Large increment
fn test_bump_integration_edge_cases(#[case] version: &str, #[case] flag: &str, #[case] increment: &str, #[case] expected: &str) {
    let output = run_zerv_command(&["version", flag, increment]);
    assert!(output.contains(expected));
}

#[rstest]
#[case("v1.0.0", "--bump-pre-release-num", "1", "no pre-release exists")]
#[case("v1.0.0-alpha", "--bump-pre-release-num", "1", "no pre-release exists")]
fn test_bump_integration_pre_release_errors(#[case] version: &str, #[case] flag: &str, #[case] increment: &str, #[case] expected: &str) {
    let result = run_zerv_command(&["version", flag, increment]);
    assert!(result.contains(expected));
}

#[rstest]
#[case("v1.0.0", BumpType::Major, 1, BumpType::Minor, 2, (Some(2), Some(2)))]
#[case("v2.5.3", BumpType::Major, 0, BumpType::Patch, 7, (Some(2), Some(10)))]
fn test_apply_bumps_method(
    #[case] version: &str,
    #[case] bump1: BumpType,
    #[case] inc1: u64,
    #[case] bump2: BumpType,
    #[case] inc2: u64,
    #[case] expected: (Option<u64>, Option<u64>)
) {
    use crate::test_utils::zerv::ZervFixture;
    use crate::cli::version::args::VersionArgs;

    let mut zerv = ZervFixture::tagged(version).unwrap();
    let mut args = VersionArgs::default();

    // Set up args based on bump types
    match bump1 {
        BumpType::Major => args.bump_major = Some(Some(inc1 as i64)),
        BumpType::Minor => args.bump_minor = Some(Some(inc1 as i64)),
        BumpType::Patch => args.bump_patch = Some(Some(inc1 as i64)),
        BumpType::Distance => args.bump_distance = Some(Some(inc1 as i64)),
        BumpType::Post => args.bump_post = Some(Some(inc1 as i64)),
        BumpType::Dev => args.bump_dev = Some(Some(inc1 as i64)),
        BumpType::Epoch => args.bump_epoch = Some(Some(inc1 as i64)),
        BumpType::PreRelease => args.bump_pre_release_num = Some(Some(inc1 as i64)),
    }

    match bump2 {
        BumpType::Major => args.bump_major = Some(Some(inc2 as i64)),
        BumpType::Minor => args.bump_minor = Some(Some(inc2 as i64)),
        BumpType::Patch => args.bump_patch = Some(Some(inc2 as i64)),
        BumpType::Distance => args.bump_distance = Some(Some(inc2 as i64)),
        BumpType::Post => args.bump_post = Some(Some(inc2 as i64)),
        BumpType::Dev => args.bump_dev = Some(Some(inc2 as i64)),
        BumpType::Epoch => args.bump_epoch = Some(Some(inc2 as i64)),
        BumpType::PreRelease => args.bump_pre_release_num = Some(Some(inc2 as i64)),
    }

    zerv.apply_bumps(&args).unwrap();

    assert_eq!(zerv.vars.major, expected.0);
    assert_eq!(zerv.vars.minor, expected.1);
}
```

**Module Integration**:

```rust
// src/version/zerv/mod.rs - Add bump module
pub mod bump;

// The bump methods are now implemented directly on Zerv in the bump module files
// No additional integration needed in core.rs - methods are available via module imports
```

#### Task 3.6: Add Override Methods to ZervVars - [x]

**Priority**: CRITICAL | **Estimated Time**: 3-4 hours

**Goal**: Add override processing methods to ZervVars for complete field coverage

**Implementation**:

```rust
// src/version/zerv/vars.rs
impl ZervVars {
    /// Apply override operations to version components
    pub fn apply_overrides(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Apply tag version override (parse and extract components)
        if let Some(ref tag_version) = args.tag_version {
            self.parse_and_apply_tag_version(tag_version, &args.input_format)?;
        }

        // Apply individual field overrides
        if let Some(distance) = args.distance {
            self.distance = Some(distance as u64);
        }

        if let Some(dirty_value) = args.dirty_override() {
            self.dirty = Some(dirty_value);
        }

        if let Some(ref branch) = args.current_branch {
            self.bumped_branch = Some(branch.clone());
        }

        if let Some(ref commit_hash) = args.commit_hash {
            self.bumped_commit_hash = Some(commit_hash.clone());
        }

        if let Some(post) = args.post {
            self.post = Some(post as u64);
        }

        if let Some(dev) = args.dev {
            self.dev = Some(dev as u64);
        }

        if let Some(ref label) = args.pre_release_label {
            self.pre_release = Some(PreReleaseVar {
                label: normalize_pre_release_label(label)
                    .ok_or_else(|| ZervError::InvalidVersion(format!("Invalid pre-release label: {}", label)))?,
                number: args.pre_release_num.map(|n| n as u64),
            });
        }

        if let Some(epoch) = args.epoch {
            self.epoch = Some(epoch as u64);
        }

        if let Some(ref custom_json) = args.custom {
            self.custom = serde_json::from_str(custom_json)
                .map_err(|e| ZervError::InvalidVersion(format!("Invalid custom JSON: {}", e)))?;
        }

        Ok(())
    }

    /// Parse tag version and apply to version components
    fn parse_and_apply_tag_version(&mut self, tag_version: &str, input_format: &str) -> Result<(), ZervError> {
        // Remove 'v' prefix if present
        let version_str = if tag_version.starts_with('v') {
            &tag_version[1..]
        } else {
            tag_version
        };

        // Parse based on input format
        match input_format {
            "semver" => self.parse_semver_version(version_str)?,
            "pep440" => self.parse_pep440_version(version_str)?,
            "auto" => {
                // Try semver first, then pep440
                if self.parse_semver_version(version_str).is_err() {
                    self.parse_pep440_version(version_str)?;
                }
            }
            _ => return Err(ZervError::InvalidFormat(format!("Unsupported input format: {}", input_format))),
        }

        Ok(())
    }

    fn parse_semver_version(&mut self, version_str: &str) -> Result<(), ZervError> {
        // Parse semantic version (e.g., "1.2.3", "1.2.3-alpha.1")
        let parts: Vec<&str> = version_str.split('.').collect();

        if parts.len() >= 3 {
            self.major = Some(parts[0].parse().map_err(|_| ZervError::InvalidVersion("Invalid major version".to_string()))?);
            self.minor = Some(parts[1].parse().map_err(|_| ZervError::InvalidVersion("Invalid minor version".to_string()))?);

            // Handle patch with potential pre-release
            let patch_part = parts[2];
            if let Some(dash_pos) = patch_part.find('-') {
                self.patch = Some(patch_part[..dash_pos].parse().map_err(|_| ZervError::InvalidVersion("Invalid patch version".to_string()))?);
                // Parse pre-release part
                let pre_release_part = &patch_part[dash_pos + 1..];
                self.parse_pre_release(pre_release_part)?;
            } else {
                self.patch = Some(patch_part.parse().map_err(|_| ZervError::InvalidVersion("Invalid patch version".to_string()))?);
            }
        } else {
            return Err(ZervError::InvalidVersion("Invalid semantic version format".to_string()));
        }

        Ok(())
    }

    fn parse_pep440_version(&mut self, version_str: &str) -> Result<(), ZervError> {
        // Parse PEP440 version (e.g., "1.2.3", "1!2.3.4", "1.2.3.post1")
        // This is a simplified parser - full PEP440 parsing would be more complex
        if let Some(bang_pos) = version_str.find('!') {
            // Has epoch
            self.epoch = Some(version_str[..bang_pos].parse().map_err(|_| ZervError::InvalidVersion("Invalid epoch".to_string()))?);
            self.parse_semver_version(&version_str[bang_pos + 1..])?;
        } else {
            self.parse_semver_version(version_str)?;
        }

        Ok(())
    }

    fn parse_pre_release(&mut self, pre_release_str: &str) -> Result<(), ZervError> {
        // Parse pre-release string (e.g., "alpha.1", "beta.2", "rc.3")
        let parts: Vec<&str> = pre_release_str.split('.').collect();

        if !parts.is_empty() {
            let label = parts[0];
            let number = if parts.len() > 1 {
                Some(parts[1].parse().map_err(|_| ZervError::InvalidVersion("Invalid pre-release number".to_string()))?)
            } else {
                None
            };

            self.pre_release = Some(PreReleaseVar {
                label: normalize_pre_release_label(label)
                    .ok_or_else(|| ZervError::InvalidVersion(format!("Invalid pre-release label: {}", label)))?,
                number,
            });
        }

        Ok(())
    }
}
```

**Architectural Improvement**: Moving `apply_overrides` to `ZervVars` instead of `Zerv` provides:

- **Better separation of concerns**: ZervVars handles its own data manipulation
- **More focused API**: Method only operates on what it actually needs
- **Cleaner implementation**: No need for lossy Zerv → VcsData → Zerv conversions
- **Complete coverage**: Handles all version fields, not just VCS-level ones
- **Consistent patterns**: Matches existing helper methods on ZervVars

#### Task 3.7: Implement Context Control Logic - [x]

**Priority**: High | **Estimated Time**: 3-4 hours

**Goal**: Implement --bump-context and --no-bump-context functionality

**Logic**:

- `--bump-context` (default): Include full VCS metadata in version
- `--no-bump-context`: Use tag version only, ignore VCS metadata

**Implementation**:

```rust
// src/cli/utils/vcs_override.rs
impl VcsOverrideProcessor {
    pub fn apply_context_control(vcs_data: VcsData, args: &VersionArgs) -> Result<VcsData> {
        // Validate context flags
        if args.bump_context && args.no_bump_context {
            return Err(ZervError::ConflictingFlags(
                "Cannot use --bump-context with --no-bump-context".to_string()
            ));
        }

        // Apply context control
        if args.no_bump_context {
            // Force clean state - no VCS metadata
            vcs_data.distance = 0;
            vcs_data.is_dirty = false;
            vcs_data.current_branch = None;
            vcs_data.commit_hash = None;
        }
        // --bump-context is default behavior, no changes needed

        Ok(vcs_data)
    }
}
```

### Phase 4: Templating System Implementation - [ ]

#### Task 4.1: Add Handlebars Dependency - [ ]

**Priority**: Medium | **Estimated Time**: 1 hour

**Goal**: Add handlebars crate for templating support

**Implementation**:

```toml
# Cargo.toml
[dependencies]
handlebars = "4.4"
serde_json = "1.0"
```

#### Task 4.2: Implement Template Processing - [ ]

**Priority**: High | **Estimated Time**: 6-8 hours

**Goal**: Implement comprehensive templating system with custom helpers

**Files to Create**:

- `src/cli/utils/template_processor.rs` - Main templating logic
- `src/cli/utils/template_helpers.rs` - Custom helper functions

**Template Context**:

```rust
// src/cli/utils/template_processor.rs
use handlebars::{Handlebars, Context, JsonValue};

pub struct TemplateProcessor {
    handlebars: Handlebars<'static>,
}

impl TemplateProcessor {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();

        // Register custom helpers
        handlebars.register_helper("add", Box::new(AddHelper));
        handlebars.register_helper("subtract", Box::new(SubtractHelper));
        handlebars.register_helper("multiply", Box::new(MultiplyHelper));
        handlebars.register_helper("hash", Box::new(HashHelper));
        handlebars.register_helper("hash_int", Box::new(HashIntHelper));
        handlebars.register_helper("prefix", Box::new(PrefixHelper));
        handlebars.register_helper("format_timestamp", Box::new(FormatTimestampHelper));

        Self { handlebars }
    }

    pub fn process_template(&self, template: &str, zerv: &Zerv) -> Result<String, ZervError> {
        let context = self.create_template_context(zerv)?;
        self.handlebars.render_template(template, &context)
            .map_err(|e| ZervError::TemplateError(format!("Template processing failed: {}", e)))
    }

    fn create_template_context(&self, zerv: &Zerv) -> Result<Context, ZervError> {
        let mut data = serde_json::Map::new();

        // Add version components
        if let Some(major) = zerv.vars.major {
            data.insert("major".to_string(), JsonValue::Number(major.into()));
        }
        // ... add all other fields

        // Add custom variables
        data.insert("custom".to_string(), zerv.vars.custom.clone());

        Ok(Context::from(data))
    }
}
```

**Custom Helpers**:

```rust
// src/cli/utils/template_helpers.rs
use handlebars::{Helper, Context, RenderContext, Output, RenderError, HelperResult};

pub struct AddHelper;
impl handlebars::HelperDef for AddHelper {
    fn call(&self, h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
        let a = h.param(0).and_then(|v| v.value().as_f64()).ok_or_else(|| RenderError::new("First parameter must be a number"))?;
        let b = h.param(1).and_then(|v| v.value().as_f64()).ok_or_else(|| RenderError::new("Second parameter must be a number"))?;
        let result = (a + b) as u64;
        out.write(&result.to_string())?;
        Ok(())
    }
}

// ... implement other helpers
```

#### Task 4.3: Integrate Templating with CLI - [ ]

**Priority**: High | **Estimated Time**: 3-4 hours

**Goal**: Integrate templating with all override and bump arguments

**Implementation**:

```rust
// src/cli/utils/vcs_override.rs
impl VcsOverrideProcessor {
    pub fn apply_templated_overrides(mut vcs_data: VcsData, args: &VersionArgs, zerv: &Zerv) -> Result<VcsData> {
        let template_processor = TemplateProcessor::new();

        // Process templated overrides
        if let Some(ref tag_version) = args.tag_version {
            if tag_version.contains("{{") {
                let processed = template_processor.process_template(tag_version, zerv)?;
                vcs_data.tag_version = Some(processed);
            } else {
                vcs_data.tag_version = Some(tag_version.clone());
            }
        }

        // ... process other templated arguments

        Ok(vcs_data)
    }
}
```

### Phase 5: Error Message Improvements - [ ]

#### Task 5.1: Source-Aware Error Messages - [ ]

**Priority**: Medium | **Estimated Time**: 2-3 hours

**Goal**: Replace generic VCS errors with source-specific messages

**Current Issues**:

- "VCS not found" → "Not in a git repository"
- "No version tag found in VCS data" → "No version tags found in git repository"
- Raw git errors → User-friendly messages

**Implementation**:

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ZervError {
    // ... existing variants ...

    #[error("Not in a git repository (--source {source}). Use -C <dir> to specify directory or --source string to parse version string")]
    NotInGitRepository { source: String },

    #[error("No version tags found in git repository")]
    NoVersionTags,

    #[error("No commits found in git repository")]
    NoCommits,

    #[error("Template processing failed: {message}")]
    TemplateError { message: String },
}
```

### Phase 6: Testing and Validation - [ ]

#### Task 6.1: Update All Tests - [ ]

**Priority**: High | **Estimated Time**: 4-6 hours

**Goal**: Update all tests to use new field names and RON format

**Files to Update**:

- `src/version/zerv/test_utils.rs` - Update test data
- `src/schema/presets/*.rs` - Update schema tests
- `src/cli/utils/format_handler.rs` - Update field validation
- All integration tests

**Test Strategy**:

1. Update field names in test data
2. Update RON format in test cases
3. Add new test cases for templating
4. Add test cases for bump functionality
5. Add test cases for context control

#### Task 6.2: Add Comprehensive Integration Tests - [ ]

**Priority**: High | **Estimated Time**: 3-4 hours

**Goal**: Add tests for all new functionality

**Test Categories**:

- Templating system tests
- Bump functionality tests
- Context control tests
- Error message tests
- RON format migration tests

### Phase 7: Documentation and Migration - [ ]

#### Task 7.1: Update Documentation - [ ]

**Priority**: Medium | **Estimated Time**: 2-3 hours

**Goal**: Update all documentation to reflect new functionality

**Files to Update**:

- README.md
- CLI help text
- Error message documentation
- Example usage

#### Task 7.2: Create Migration Guide - [ ]

**Priority**: Medium | **Estimated Time**: 1-2 hours

**Goal**: Create guide for users migrating from old RON format

**Content**:

- Field name changes
- RON format changes
- Breaking changes
- Migration examples

## Implementation Timeline

### Week 1: Critical Missing Functionality (Phase 3) - [ ]

- **Days 1-2**: Implement centralized bump logic in Zerv object - [ ]
- **Days 3-4**: Fix --source git implementation and add --source stdin - [ ]
- **Day 5**: Integration testing and validation - [ ]

### Week 2: Foundation (Phase 1-2) - [ ]

- **Days 1-2**: Constants definition and field renaming - [ ]
- **Days 3-4**: Component enum updates and RON format migration - [ ]
- **Day 5**: Timestamp pattern enhancements - [ ]

### Week 3: Advanced Features (Phase 4-5) - [ ]

- **Days 1-3**: Templating system implementation - [ ]
- **Days 4-5**: Error message improvements - [ ]

### Week 4: Polish and Testing (Phase 6-7) - [ ]

- **Days 1-2**: Test updates and new test cases - [ ]
- **Days 3-4**: Documentation and migration guide - [ ]
- **Day 5**: Final integration testing - [ ]

## Risk Mitigation

### Breaking Changes

- **Risk**: RON format changes break existing schemas
- **Mitigation**: Direct migration - no backward compatibility needed in early development

### Performance Impact

- **Risk**: Templating system adds overhead
- **Mitigation**: Lazy template processing, caching for repeated operations

### Test Coverage

- **Risk**: New functionality not properly tested
- **Mitigation**: Comprehensive test suite, integration tests for all new features

## Success Criteria

### Functional Requirements

- [ ] All ideal spec features implemented
- [ ] Clean, maintainable code with no redundant implementations
- [ ] All tests passing
- [ ] Performance within acceptable limits

### Quality Requirements

- [ ] Error messages are clear and actionable
- [ ] Documentation is comprehensive and up-to-date
- [ ] Code follows project standards
- [ ] No breaking changes without migration path

### User Experience

- [ ] CLI is intuitive and follows expected patterns
- [ ] Error messages help users resolve issues
- [ ] Examples work as documented
- [ ] Code is clean and easily maintainable

## Dependencies

### External Dependencies

- `handlebars` crate for templating
- `serde_json` for custom variable handling
- `chrono` for timestamp formatting (already present)

### Internal Dependencies

- Constants must be defined before field renaming
- Field renaming must be complete before templating
- Component enum updates must be complete before RON format migration
- All core changes must be complete before comprehensive testing
- Direct implementation approach - no backward compatibility layers needed

## Notes

### Code Quality Focus

- Clean, maintainable code with no redundant implementations
- Scalable architecture that can easily accommodate future features
- No backward compatibility concerns - this is early development
- Direct migration to ideal state without legacy support

### Performance Considerations

- Template processing will be cached for repeated operations
- Constants will be defined at compile time
- Error message formatting will be optimized

### Future Extensibility

- Template system is designed to be easily extensible
- New timestamp patterns can be added without breaking changes
- New bump types can be added following established patterns

This plan provides a comprehensive roadmap to achieve the ideal version command specification with clean, maintainable, and scalable code architecture.
