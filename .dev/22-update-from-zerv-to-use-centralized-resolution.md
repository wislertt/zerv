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
// Simple API - uses default Tier 3 schema (80% use case)
impl From<Zerv> for PEP440 { ... }
impl From<PEP440> for Zerv { ... }

// Advanced API - explicit schema (20% use case)
impl PEP440 {
    fn from_zerv_with_schema(zerv: &Zerv, schema: &ZervSchema) -> Result<Self, ConversionError>
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
- **Advanced API**: `from_zerv_with_schema()` for custom schemas?
    - _Context_: Power users need control over component placement
    - _Current issue_: No way to specify custom schemas explicitly
- **Error handling**: How to handle invalid schemas vs invalid data?
    - _Context_: Different error types need different handling strategies
    - _Current issue_: Generic errors, hard to understand what went wrong
- **Backward compatibility**: How to migrate existing code?
    - _Context_: Existing tests and usage must continue working
    - _Current issue_: Any API change could break existing code

### Schema Constraints

- **Core rules**: major/minor/patch must be in schema.core positions 0,1,2?
    - _Context_: All version formats expect major.minor.patch as base
    - _Current issue_: Components can appear anywhere, breaking format assumptions
- **Standard components**: epoch/pre_release/post/dev placement rules?
    - _Context_: These have semantic meaning in target formats
    - _Current issue_: No enforcement of where these can appear
- **Custom components**: Where can they appear? Any restrictions?
    - _Context_: User flexibility vs format compatibility
    - _Current issue_: Custom components can break target format parsing
- **VCS components**: Flexible placement or fixed rules?
    - _Context_: VCS info is metadata, should be in build/local sections
    - _Current issue_: VCS components mixed with version semantics

### Conversion Strategy

- **Schema validation**: Validate before conversion or during?
    - _Context_: Fail fast vs lazy validation trade-offs
    - _Current issue_: Validation scattered throughout conversion process
- **Component resolution**: Use Plan 20 methods with schema context?
    - _Context_: Plan 20 provides centralized resolution, need schema awareness
    - _Current issue_: Manual resolution duplicated across formats
- **Error propagation**: How to handle resolution failures?
    - _Context_: Missing vars vs invalid schemas vs format limitations
    - _Current issue_: Panics on invalid data, poor error messages
- **Performance**: Schema validation cost vs conversion cost?
    - _Context_: Validation overhead vs runtime safety
    - _Current issue_: No performance benchmarks for current approach

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

- `src/version/zerv/schema.rs` - Schema validation and constraint enforcement
- `src/version/pep440/from_zerv.rs` - Replace manual resolution with Plan 20 methods
- `src/version/semver/from_zerv.rs` - Replace manual resolution with Plan 20 methods
- `src/version/zerv/components.rs` - Add schema-aware resolution methods
- `src/version/zerv/errors.rs` - New error types for schema/conversion failures

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

## Next Steps

Review each design question, make decisions, then create concrete implementation plan with specific code changes.
