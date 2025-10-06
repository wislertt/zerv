# Schema-Based Bump Implementation Plan

## Overview

This document outlines the implementation plan for schema-based bump functionality, which addresses:

1. **Doc 13**: Schema-based bump functionality - higher-level bumping system
2. **Doc 16**: Reset logic schema component issue - proper handling of schema components during bumping

## Current State vs Ideal State

### Current State (Problems)

#### 1. Limited Bumping Capability

```bash
# Current: Only field-based bumps
zerv version --bump-major        # Bumps vars.major
zerv version --bump-minor        # Bumps vars.minor

# Missing: Schema-based bumps
zerv version --bump-core 0 1          # Bump first component of core schema
zerv version --bump-extra-core 2 3    # Bump third component of extra_core schema
zerv version --bump-build 1 5         # Bump second component of build schema
```

#### 2. Incomplete Reset Logic

```bash
# Input: 1.5.2-rc.1+build.456
# Command: --bump-major
# Expected: 2.0.0 (all lower precedence removed)
# Actual: 2.0.0+build.456 (build metadata persists - BUG!)
```

**Root Cause**: Schema components are not reset when their underlying `ZervVars` fields are reset.

### Ideal State (Goals)

#### 1. Schema-Based Bumping

- Bump any schema component by position (index)
- Automatically resolve component type (VarField, Integer, String, Timestamp)
- Apply appropriate reset behavior based on schema precedence
- Handle complex scenarios with multiple index bumps

#### 2. Complete Reset Logic

- Reset both ZervVars fields AND schema components
- Schema-aware component filtering
- Preserve static components when appropriate

## Ideal Architecture

### Core Data Structures

```rust
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Precedence {
    // Field-based precedence
    Epoch,
    Major,
    Minor,
    Patch,
    PreReleaseLabel,
    PreReleaseNum,
    Post,
    Dev,

    // Schema-based precedence
    Core,
    ExtraCore,
    Build,
}

#[derive(Debug, Clone)]
pub struct PrecedenceOrder {
    order: IndexMap<Precedence, ()>,
}

impl PrecedenceOrder {
    pub fn from_precedences(precedences: Vec<Precedence>) -> Self {
        let order = precedences.into_iter()
            .map(|p| (p, ()))
            .collect();
        Self { order }
    }

    pub fn pep440_based() -> Self {
        Self::from_precedences(vec![
            Precedence::Epoch,
            Precedence::Major,
            Precedence::Minor,
            Precedence::Patch,
            Precedence::Core,
            Precedence::PreReleaseLabel,
            Precedence::PreReleaseNum,
            Precedence::Post,
            Precedence::Dev,
            Precedence::ExtraCore,
            Precedence::Build,
        ])
    }

    /// O(1) get precedence by index
    pub fn get_precedence(&self, index: usize) -> Option<&Precedence> {
        self.order.get_index(index).map(|(precedence, _)| precedence)
    }

    /// O(1) get index by precedence
    pub fn get_index(&self, precedence: &Precedence) -> Option<usize> {
        self.order.get_index_of(precedence)
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Precedence> {
        self.order.keys()
    }

    /// Get field precedence names in order (for backward compatibility with BumpType::PRECEDENCE_NAMES)
    pub fn field_precedence_names(&self) -> &[&'static str] {
        // Return the field-based precedence names in order
        // This maintains compatibility with existing BumpType logic
        &[
            "epoch", "major", "minor", "patch",
            "pre_release_label", "pre_release_num", "post", "dev"
        ]
    }
}

#[derive(Debug, Clone)]
pub struct ZervSchema {
    pub core: Vec<Component>,
    pub extra_core: Vec<Component>,
    pub build: Vec<Component>,
    precedence_order: PrecedenceOrder, // Single source of truth
}

impl ZervSchema {
    pub fn new(
        core: Vec<Component>,
        extra_core: Vec<Component>,
        build: Vec<Component>,
        precedence_order: PrecedenceOrder
    ) -> Self {
        Self { core, extra_core, build, precedence_order }
    }

}
```

### CLI Interface

```bash
# Schema-based bump arguments (relative modifications)
--bump-core <index> <value> [<index> <value> ...]
--bump-extra-core <index> <value> [<index> <value> ...]
--bump-build <index> <value> [<index> <value> ...]

# Schema-based override arguments (absolute values)
--core <index> <value> [<index> <value> ...]
--extra-core <index> <value> [<index> <value> ...]
--build <index> <value> [<index> <value> ...]
```

### RON Schema Support

```ron
SchemaConfig(
    core: [
        VarField(field: "major"),
        VarField(field: "minor"),
        VarField(field: "patch"),
    ],
    extra_core: [
        VarField(field: "pre_release"),
    ],
    build: [
        VarField(field: "branch"),
    ],
    precedence_order: [  // Optional - uses default if not specified
        Epoch,
        Major,
        Minor,
        Patch,
        Core,
        PreReleaseLabel,
        PreReleaseNum,
        Post,
        Dev,
        ExtraCore,
        Build
    ]
)
```

## Unified Reset Logic Implementation (Doc 16)

### Architecture Decision: Option 1a + 2a + 3a

**Chosen Approach:**

- **1a**: Method on `Zerv` (not `ZervVars`) - coordinates both vars and schema reset
- **2a**: Loop through field-based precedence using Precedence Order
- **3a**: Remove ALL component types (VarField, String, Integer, Timestamp) with lower precedence
- **Constraint**: Timestamp components (`VarTimestamp`) cannot be bumped - will raise error if attempted

### Implementation Details

```rust
impl Zerv {
    /// Reset all components (vars + schema) with lower precedence than the given component
    pub fn reset_lower_precedence_components(&mut self, component: &str) -> Result<(), ZervError> {
        let current_precedence = BumpType::precedence_from_str(component);

        // 1. Reset ZervVars fields with lower precedence
        for (index, &name) in self.schema.precedence_order.field_precedence_names().iter().enumerate() {
            if index > current_precedence {
                self.vars.reset_component_by_name(name);
            }
        }

        // 2. Determine which fields are reset
        let reset_fields: HashSet<&str> = self.schema.precedence_order.field_precedence_names()
            .iter()
            .skip(current_precedence + 1)
            .copied()
            .collect();

        // 3. Filter each schema section
        self.schema.core = Self::filter_section(&self.schema.core, &reset_fields);
        self.schema.extra_core = Self::filter_section(&self.schema.extra_core, &reset_fields);
        self.schema.build = Self::filter_section(&self.schema.build, &reset_fields);

        Ok(())
    }

    /// Bump a schema component by index with validation
    pub fn bump_schema_component(&mut self, section: &str, index: usize, value: u64) -> Result<(), ZervError> {
        let components = match section {
            "core" => &self.schema.core,
            "extra_core" => &self.schema.extra_core,
            "build" => &self.schema.build,
            _ => return Err(ZervError::InvalidBumpTarget(format!("Unknown schema section: {}", section))),
        };

        let component = components.get(index)
            .ok_or_else(|| ZervError::InvalidBumpTarget(format!("Index {} out of bounds for {} section", index, section)))?;

        match component {
            Component::VarField(field_name) => {
                // Validate field can be bumped
                if !self.schema.precedence_order.field_precedence_names().contains(&field_name.as_str()) {
                    return Err(ZervError::InvalidBumpTarget(format!("Cannot bump custom field: {}", field_name)));
                }
                // Bump the field and reset lower precedence components
                self.bump_field_and_reset(field_name, value)?;
            }
            Component::String(_) => {
                // String components can be bumped (e.g., version strings, labels)
                // Implementation would need to handle string bumping logic
                return Err(ZervError::NotImplemented("String component bumping not yet implemented".to_string()));
            }
            Component::Integer(_) => {
                // Integer components can be bumped (e.g., build numbers, patch versions)
                // Implementation would need to handle integer bumping logic
                return Err(ZervError::NotImplemented("Integer component bumping not yet implemented".to_string()));
            }
            Component::VarTimestamp(_) => {
                return Err(ZervError::InvalidBumpTarget("Cannot bump timestamp component - timestamps are generated dynamically".to_string()));
            }
        }

        Ok(())
    }

    /// Filter a schema section:
    /// - If section contains ANY reset VarField, clear ENTIRE section (aggressive per 3a)
    /// - Otherwise, keep section as-is
    fn filter_section(components: &[Component], reset_fields: &HashSet<&str>) -> Vec<Component> {
        // Check if section contains any reset fields
        let has_reset_field = components.iter().any(|comp| {
            matches!(comp, Component::VarField(field_name) if reset_fields.contains(field_name.as_str()))
        });

        if has_reset_field {
            // Clear entire section (aggressive removal per 3a)
            Vec::new()
        } else {
            // Keep section unchanged
            components.to_vec()
        }
    }
}
```

### Example Behavior

**Input Schema:**

```ron
core: [VarField("major"), String("."), VarField("minor"), String("."), VarField("patch")]
extra_core: [VarField("pre_release")]
build: [VarField("branch"), String("."), VarField("commit_hash_short")]
```

**Bump Major (precedence 1):**

- Reset fields: `minor`, `patch`, `pre_release_label`, `pre_release_num`, `post`, `dev`
- Core section: Contains `minor` and `patch` → **Clear entire section** → `[VarField("major")]`
- ExtraCore section: Contains `pre_release` → **Clear entire section** → `[]`
- Build section: No reset fields → **Keep unchanged** → `[VarField("branch"), String("."), VarField("commit_hash_short")]`

**Result:** `2.0.0+branch.abc123` (build metadata preserved because no reset fields in build section)

**Bump Minor (precedence 2):**

- Reset fields: `patch`, `pre_release_label`, `pre_release_num`, `post`, `dev`
- Core section: Contains `patch` → **Clear entire section** → `[VarField("major"), String("."), VarField("minor")]`
- ExtraCore section: Contains `pre_release` → **Clear entire section** → `[]`
- Build section: No reset fields → **Keep unchanged** → `[VarField("branch"), String("."), VarField("commit_hash_short")]`

**Result:** `1.3.0+branch.abc123`

### Component Bump Validation

**Allowed Components:**

- `VarField` with field names in Precedence Order (major, minor, patch, etc.)
- `String` - Static string literals (e.g., version labels, build identifiers)
- `Integer` - Static integer literals (e.g., build numbers, patch versions)

**Forbidden Components:**

- `VarTimestamp` - Timestamps are generated dynamically, not bumped
- `VarField` with custom field names (e.g., `custom.build_id`)

**Not Yet Implemented:**

- `String` and `Integer` component bumping (placeholder for future implementation)

**Error Types:**

```rust
// New error variants for ZervError
InvalidBumpTarget(String) // "Cannot bump timestamp component - timestamps are generated dynamically"
NotImplemented(String)    // "String component bumping not yet implemented"
```

**Example Error Scenarios:**

```bash
# Attempting to bump timestamp component
zerv version --bump-core 2 1  # If core[2] is VarTimestamp("YYYY")
# Error: Cannot bump timestamp component - timestamps are generated dynamically

# Attempting to bump string component (not yet implemented)
zerv version --bump-core 1 1  # If core[1] is String("alpha")
# Error: String component bumping not yet implemented

# Attempting to bump integer component (not yet implemented)
zerv version --bump-core 3 1  # If core[3] is Integer(42)
# Error: Integer component bumping not yet implemented

# Attempting to bump custom field
zerv version --bump-build 0 1  # If build[0] is VarField("custom.build_id")
# Error: Cannot bump custom field: custom.build_id
```

## Implementation Roadmap

### Phase 1: Core Infrastructure (Week 1)

**Goal**: Establish the foundation for schema-based bumping

**Tasks**:

- [x] Add `Precedence` enum
- [x] Update `ZervSchema` to use `IndexMap<Precedence, ()>`
- [x] Add `pep440_based_precedence_order()` method
- [x] Update `SchemaConfig` for RON parsing with default precedence
- [x] Add CLI argument parsing for schema-based flags

**Files to Create/Modify**:

- `src/version/zerv/bump/precedence.rs` - New Precedence enum
- `src/version/zerv/schema.rs` - Update ZervSchema
- `src/version/zerv/schema_config.rs` - Update for IndexMap
- `src/cli/version/args.rs` - Add schema-based arguments

**Success Criteria**:

- [x] Can create ZervSchema with custom precedence
- [x] RON parsing works with and without precedence_order
- [x] CLI can parse schema-based arguments

**Status**: ✅ **COMPLETED** - All tasks and success criteria met

**Verification**:

- ✅ 1712 tests passing
- ✅ `make lint` passes with no warnings
- ✅ CLI arguments parse and validate correctly
- ✅ ZervSchema creation with custom precedence works
- ✅ RON parsing with/without precedence_order works
- ✅ Schema-based bump arguments (`--bump-core`, `--bump-extra-core`, `--bump-build`) implemented

### Phase 2: Schema-Based Bump Logic (Week 2)

**Goal**: Implement the core bumping functionality

**Tasks**:

- [ ] Add `SchemaBump` variant to `BumpType` enum
- [ ] Implement `bump_by_schema()` method
- [ ] Add component type resolution logic
- [ ] Implement precedence-based sorting
- [ ] Add error handling for invalid operations

**Files to Create/Modify**:

- `src/version/zerv/bump/types.rs` - Add SchemaBump variant
- `src/version/zerv/bump/schema.rs` - New schema bump logic
- `src/version/zerv/bump/mod.rs` - Integrate schema bumps

**Success Criteria**:

- [ ] Can bump VarField components
- [ ] Can bump String/Integer components
- [ ] Appropriate errors for unsupported components
- [ ] Precedence-based processing works

### Phase 3: Reset Logic Enhancement (Week 3)

**Goal**: Fix Doc 16 - complete reset logic with unified Zerv method

**Tasks**:

- [ ] Move `reset_lower_precedence_components()` from `ZervVars` to `Zerv` impl
- [ ] Implement section-based schema filtering (aggressive removal per Option 3a)
- [ ] Add `filter_section()` helper method for schema component removal
- [ ] Update call sites to use `zerv.reset_lower_precedence_components()`
- [ ] Update tests for unified reset behavior

**Files to Create/Modify**:

- `src/version/zerv/bump/reset.rs` - Move method to Zerv impl, add schema filtering
- `src/version/zerv/bump/vars_primary.rs` - Update call sites
- `src/version/zerv/core.rs` - Add Component imports if needed

**Success Criteria**:

- [ ] `Zerv::reset_lower_precedence_components()` resets both vars and schema
- [ ] Sections with reset fields are completely cleared (aggressive per 3a)
- [ ] Sections without reset fields are preserved
- [ ] Doc 16 issue is resolved (build metadata removed when appropriate)

### Phase 4: Integration and Testing (Week 4)

**Goal**: Complete integration and ensure reliability

**Tasks**:

- [ ] Integrate schema-based bumps into main processing loop
- [ ] Add conflict detection and validation
- [ ] Write comprehensive tests
- [ ] Test end-to-end scenarios

**Files to Create/Modify**:

- `src/version/zerv/bump/mod.rs` - Main integration
- `tests/integration_tests/` - Add schema bump tests

**Success Criteria**:

- [ ] Schema-based bumps work in CLI
- [ ] All tests pass
- [ ] No regressions in existing functionality

### Phase 5: Documentation and Polish (Week 5)

**Goal**: Complete the feature with proper documentation

**Tasks**:

- [ ] Update CLI help text
- [ ] Update README with examples
- [ ] Add migration guide
- [ ] Polish error messages

**Success Criteria**:

- [ ] Complete documentation
- [ ] Clear error messages
- [ ] User-friendly examples

## Key Design Decisions

### 1. Unified Reset Logic (Doc 16 Solution)

**Decision**: Move `reset_lower_precedence_components()` to `Zerv` and implement aggressive section-based filtering
**Rationale**:

- Zerv owns both schema and vars - natural place for coordinated reset logic
- Aggressive removal (Option 3a) ensures semantic correctness
- Example: Bumping major in `1.5.2-rc.1+build.456` yields `2.0.0`, not `2.0.0+build`

**Implementation Approach**:

- Loop through field-based precedence using Precedence Order from schema
- Reset ZervVars fields with lower precedence
- Clear entire schema sections that contain reset fields (aggressive per 3a)
- Preserve sections without reset fields

### 2. IndexMap for Precedence Order

**Decision**: Use `IndexMap<Precedence, ()>` instead of `Vec<Precedence>`
**Rationale**: Provides O(1) bidirectional lookup, guarantees no duplicates, maintains order

### 3. Explicit Precedence in Constructor

**Decision**: `ZervSchema::new()` requires explicit precedence_order parameter
**Rationale**: Makes precedence explicit, allows custom precedence, uses `pep440_based_precedence_order()` for defaults

### 4. RON with Default Precedence

**Decision**: Use `#[serde(default)]` for precedence_order in RON
**Rationale**: Backward compatible, no Option complexity, clear default behavior

### 5. Component Type Resolution

**Decision**: Resolve component type at bump time with validation
**Rationale**: Flexible, handles different component types appropriately, clear error messages

### 6. Component Bump Constraints

**Decision**: Different component types have different bump constraints
**Rationale**:

- `VarField`: Can be bumped if field name is in Precedence Order
- `String`/`Integer`: Can be bumped (useful for version labels, build numbers)
- `VarTimestamp`: Cannot be bumped (dynamic time-based values)
- Custom `VarField`: Cannot be bumped (not in Precedence Order)

**Error Handling**:

- `ZervError::InvalidBumpTarget` for forbidden components (timestamps, custom fields)
- `ZervError::NotImplemented` for not-yet-implemented components (strings, integers)

## Benefits

### 1. Solves Doc 13: Schema-Based Bumping

- ✅ Bump any schema component by position
- ✅ Works with any schema configuration
- ✅ Automatically adapts to schema changes
- ✅ No hardcoded field assumptions

### 2. Solves Doc 16: Reset Logic Issue

- ✅ Unified `Zerv::reset_lower_precedence_components()` method
- ✅ Aggressive section-based filtering removes entire sections with reset fields
- ✅ Schema components are properly reset alongside vars
- ✅ Build metadata is removed when appropriate (e.g., major bump clears build section)
- ✅ Semantic versioning compliance maintained

### 3. Flexible and Extensible

- ✅ Works across all schema parts (core, extra_core, build)
- ✅ Type-safe component resolution
- ✅ Extensible to new schema structures
- ✅ Backwards compatible with existing field-based bumps

### 4. User-Friendly

- ✅ Intuitive CLI syntax
- ✅ Clear error messages
- ✅ Supports complex scenarios
- ✅ Deterministic behavior

## Success Metrics

### Functional Requirements

- [ ] Can bump schema components by position
- [ ] Component type resolution works correctly
- [ ] Reset logic handles schema components
- [ ] CLI arguments parse correctly
- [ ] Doc 16 issue is resolved

### Non-Functional Requirements

- [ ] Performance is not degraded
- [ ] Error messages are clear
- [ ] Documentation is complete
- [ ] Tests provide good coverage
- [ ] Backwards compatible with existing functionality

## Conclusion

This implementation plan provides a clear path from the current state to the ideal state of schema-based bumping. The phased approach ensures that each step builds on the previous one, with clear success criteria and measurable outcomes.

The key insight is that schema-based bumping and proper reset logic are two sides of the same coin - both require understanding the precedence relationship between schema components and their underlying data structures.
