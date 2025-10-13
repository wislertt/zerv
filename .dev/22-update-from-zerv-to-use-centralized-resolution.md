# Plan 22: Schema-First from/to Zerv Redesign

## Background Context

### Current State Problems

The existing `from_zerv.rs` implementations in PEP440 and SemVer suffer from:

- **Schema Inference Complexity**: Trying to guess schema from target format is error-prone
- **Manual Resolution Logic**: Duplicated component→value mapping across formats
- **No Sanitization**: String values used as-is without format-specific cleaning
- **Hard to Maintain**: Changes require updates in multiple places
- **Ambiguous Mapping**: Same component could map to different schema positions

### Plan 20 Foundation

Plan 20 (Component Resolution Centralization) provides:

- `Component::resolve_value(&vars, &sanitizer)` - Get sanitized component value
- `Var::resolve_expanded_values(&vars, &sanitizer)` - Get label+value pairs
- Format-specific sanitizers from Plan 19
- Single source of truth for component resolution

### Current Conversion Flow Issues

```rust
// Current problematic pattern in from_zerv.rs
Var::Post => {
    if let Some(post_num) = zerv.vars.post {  // Manual vars access
        components.post_label = Some(PostLabel::Post);
        components.post_number = Some(post_num as u32);  // No sanitization
    }
}
```

## Prerequisites

**CRITICAL**: This plan must be implemented **AFTER** Plan 20 (Component Resolution Centralization) is completed.

## Core Problem

Current schema inference approach is complex and error-prone. Need schema-first design with clear constraints and centralized resolution.

## Proposed Solution Direction

### Two-Tier API Approach

```rust
// From Zerv (uses schema already in Zerv)
impl From<Zerv> for PEP440 { ... }
impl From<Zerv> for SemVer { ... }

// To Zerv - Simple API (uses default schema)
impl From<PEP440> for Zerv { ... }
impl From<SemVer> for Zerv { ... }

// To Zerv - Advanced API (explicit schema for power users)
impl PEP440 {
    fn to_zerv_with_schema(&self, schema: &ZervSchema) -> Result<Zerv, ConversionError>
}

impl SemVer {
    fn to_zerv_with_schema(&self, schema: &ZervSchema) -> Result<Zerv, ConversionError>
}
```

### Schema Constraint Philosophy

Enforce rules to reduce complexity:

- Core positions reserved for major/minor/patch only
- Standard components have designated areas
- Custom components flexible but validated
- Clear error messages for violations

## Key Design Questions to Resolve

### Schema Architecture

- **Two-tier API**: Simple default vs explicit schema?
    - _Context_: 80/20 rule - most users want standard behavior, power users need flexibility
    - _Current issue_: Single complex API tries to handle all cases
- **Schema constraints**: What rules should be enforced?
    - _Context_: Without rules, combinatorial explosion of edge cases
    - _Current issue_: Schema inference tries to guess intent
- **Component positioning**: Where can major/minor/patch appear?
    - _Context_: All target formats expect core version components in predictable places
    - _Current issue_: Flexible positioning creates parsing ambiguity
- **Validation timing**: Schema validation vs conversion validation?
    - _Context_: Early validation prevents runtime errors
    - _Current issue_: Errors discovered during conversion, hard to debug

### API Design

- **Simple API**: `From<Zerv>` with default Tier 3 schema?
    - _Context_: Most users just want "convert my version to PEP440/SemVer"
    - _Current issue_: Complex API scares away simple use cases
- **Advanced API**: `to_zerv_with_schema()` for custom schemas?
    - _Context_: Power users need control over schema when converting TO Zerv
    - _Current issue_: No way to specify custom schemas when creating Zerv from other formats

- **Backward compatibility**: How to migrate existing code?
    - _Context_: Existing tests and usage must continue working
    - _Current issue_: Any API change could break existing code
    - _Migration Strategy_: No backward compatibility handling in code. For simple changes, update all at once. For complex changes, implement new API first, ensure it works, then delete old API and rename new API to final name.

### Schema Constraints

**Component Categories (documented in Var enum comments):**

- **Primary**: major/minor/patch - schema.core only, correct order when present, used once each
- **Secondary**: epoch/pre_release/post/dev - schema.extra_core only, used once each, any order
- **Context**: VCS fields, timestamps, custom - can appear anywhere, multiple uses allowed

**Validation Rules:**

- Primary components MUST be in schema.core only, in correct order (major→minor→patch), no duplicates
- Secondary components MUST be in schema.extra_core only, used once each
- Context components have full flexibility for placement and usage
- Clear error messages for constraint violations

**Examples:**

- Valid: `[major, minor, patch]`, `[yyyy, mm, dd, patch]`, `[major, patch]`
- Invalid: `[minor, major, patch]` (wrong order), `[major, major, patch]` (duplicate)

**Benefits:**

- Predictable core version parsing for all formats
- Flexible secondary component ordering per user preference
- Full extensibility for context components
- Consistent with existing bump process categories

### Conversion Strategy

- **Schema validation**: Validate before conversion or during?
    - _Context_: Fail fast vs lazy validation trade-offs
    - _Solution_: Private fields with validated getters/setters ensure schemas are always valid, eliminating validation timing concerns
- **Component resolution**: Use Plan 20 methods with schema context?
    - _Context_: Plan 20 provides centralized resolution, need schema awareness
    - _Solution_: Plan 20 is complete. Use Component::resolve_value() and Var::resolve_expanded_values() exclusively, no manual resolution
- **Error propagation**: How to handle resolution failures?
    - _Context_: Missing vars vs invalid schemas vs format limitations
    - _Solution_: All resolution failures return Result<T, ZervError>, no panics

### Implementation Approach

- **Phase 1**: Schema validation and constraints?
    - _Context_: Foundation for all other phases
    - _Dependencies_: Plan 20 component resolution methods
- **Phase 2**: Simple API with default schema?
    - _Context_: Preserve existing behavior for most users
    - _Dependencies_: Phase 1 validation
- **Phase 3**: Advanced API with custom schemas?
    - _Context_: Enable power user scenarios
    - _Dependencies_: Phase 2 simple API working
- **Phase 4**: Migrate existing from_zerv implementations?
    - _Context_: Replace manual resolution with Plan 20 methods
    - _Dependencies_: All previous phases complete

## Files Potentially Affected

- `src/version/zerv/schema.rs` - Private fields, getters/setters, validation enforcement
- `src/version/pep440/from_zerv.rs` - Replace manual resolution with Plan 20 methods
- `src/version/semver/from_zerv.rs` - Replace manual resolution with Plan 20 methods
- `src/version/zerv/components.rs` - Add schema-aware resolution methods
- `src/version/zerv/errors.rs` - New ValidationError types for schema failures

## Related Plans

- **Plan 19**: String Sanitization Utils (provides format-specific sanitizers)
- **Plan 20**: Component Resolution Centralization (provides resolve_value methods)
- **Future Plan**: Custom field support using hashmap design from spec

## Success Criteria (TBD)

- **Schema Rules**: Clear, documented constraints with good error messages
- **Simple API**: `From<Zerv>` works for 80% of use cases without schema knowledge
- **Advanced API**: Power users can specify custom schemas explicitly
- **Code Quality**: Single source of truth for resolution, no duplication
- **Backward Compatibility**: All existing tests pass, no breaking changes
- **Performance**: No significant regression in conversion speed
- **Maintainability**: Easy to add new Var types and target formats

## ZervSchema API Design

### Private Fields with Validated Access

```rust
#[derive(Debug, Clone)]
pub struct ZervSchema {
    core: Vec<Component>,           // Private fields
    extra_core: Vec<Component>,
    build: Vec<Component>,
    precedence_order: PrecedenceOrder,
}

impl ZervSchema {
    // Getters for read access
    pub fn core(&self) -> &Vec<Component> {
        &self.core
    }

    pub fn extra_core(&self) -> &Vec<Component> {
        &self.extra_core
    }

    pub fn build(&self) -> &Vec<Component> {
        &self.build
    }

    pub fn precedence_order(&self) -> &PrecedenceOrder {
        &self.precedence_order
    }

    // Setters with validation
    pub fn set_core(&mut self, core: Vec<Component>) -> Result<(), ValidationError> {
        validate_core(&core)?;
        self.core = core;
        Ok(())
    }

    pub fn set_extra_core(&mut self, extra_core: Vec<Component>) -> Result<(), ValidationError> {
        validate_extra_core(&extra_core)?;
        self.extra_core = extra_core;
        Ok(())
    }

    // Constructor with validation
    pub fn new(
        core: Vec<Component>,
        extra_core: Vec<Component>,
        build: Vec<Component>,
        precedence_order: PrecedenceOrder,
    ) -> Result<Self, ValidationError> {
        validate_core(&core)?;
        validate_extra_core(&extra_core)?;
        Ok(Self { core, extra_core, build, precedence_order })
    }
}
```

**Benefits:**

- Impossible to create invalid schemas (private fields + validated constructors)
- Read access via getters returning references
- Write access only through validated setters
- Validation enforced at compile time through API design

## Schema Validation Implementation

```rust
// Validation logic using Var enum comments (no explicit ComponentCategory needed)
fn validate_component_placement(var: &Var, section: SchemaSection) -> Result<(), Error> {
    match var {
        // Primary components (schema.core only, correct order when present, used once each)
        Var::Major | Var::Minor | Var::Patch => {
            if section != SchemaSection::Core {
                return Err("Primary components must be in core section");
            }
        }
        // Secondary components (schema.extra_core only, used once each, any order)
        Var::Epoch | Var::PreRelease | Var::Post | Var::Dev => {
            if section != SchemaSection::ExtraCore {
                return Err("Secondary components must be in extra_core section");
            }
        }
        // Context components (anywhere, multiple uses allowed)
        _ => {} // No restrictions
    }
    Ok(())
}
```

## Next Steps

Implement schema validation with the defined component categories, then create concrete conversion plan.
