# Handlebars to Tera Migration Plan

**Status**: Planned
**Priority**: High
**Created**: 2025-11-01
**Author**: Claude Code

## Context

The current Handlebars implementation in Zerv has grown to over 1000 lines of custom helper code to compensate for Handlebars limitations. Key pain points include:

- **No default value handling**: Simple needs like `{{post | default: 0}}` require custom helpers
- **Limited built-in functionality**: Math operations, string manipulation require custom implementation
- **Complex helper API**: Parameter extraction and validation is verbose and error-prone
- **Rabbit hole effect**: Each new requirement often needs a new custom helper

## Current Handlebars Implementation Analysis

### Scope (Lines of Code)

- **Custom helpers**: 1006 lines (`src/cli/utils/template/helpers.rs`)
- **Template types**: 287 lines (`src/cli/utils/template/types.rs`)
- **Template context**: 258 lines (`src/cli/utils/template/context.rs`)
- **Integration tests**: 430 lines (`tests/integration_tests/version/main/templates.rs`)
- **Total**: ~2000 lines of template-related code

### Custom Helpers Implemented

1. **`sanitize`** (141 lines) - String sanitization with presets/custom params
2. **`hash`** (25 lines) - Hex string generation
3. **`hash_int`** (35 lines) - Numeric hash with length control
4. **`prefix`** (23 lines) - String prefix extraction
5. **`format_timestamp`** (30 lines) - Timestamp formatting
6. **Math helpers** (37 lines) - `add`, `subtract`, `multiply`
7. **Parameter extraction utilities** (40 lines) - Helper param validation

### Key Limitations Encountered

- No built-in default value handling
- Limited conditional logic
- No built-in string manipulation functions
- Verbose custom helper implementation
- Complex parameter validation boilerplate

## Tera Advantages

### Built-in Functionality

- **Default filter**: `{{ post | default(value=0) }}` - Solves the primary pain point
- **Math operations**: `+`, `-`, `*`, `/`, `%` operators in expressions
- **String filters**: `capitalize`, `upper`, `lower`, `trim`, `truncate`, etc.
- **Control structures**: `if/elif/else`, `for` loops, `set` assignments
- **Built-in functions**: `range`, `now`, `get_random`, `get_env`
- **Date formatting**: `| date(format="...")` filter
- **Hashing**: Can be implemented as simple filter functions

### Expression Capabilities

- Mathematical expressions: `{{ major + 1 }}`
- String concatenation: `{{ prefix ~ "-" ~ suffix }}`
- Comparison operations: `{% if post > 0 %}`
- Logical operations: `{% if dirty or distance > 0 %}`

### Simpler Custom Functions

- Function-based API vs Handlebars' trait-based helpers
- No manual parameter extraction required
- Better error handling and debugging

## Migration Strategy

### Phase 1: Isolate Handlebars Implementation (Zero Risk)

**Duration**: 1 day

#### 1.1 Create Handlebars Subdirectory

```
src/cli/utils/template/
├── mod.rs              # Module exports (unchanged)
├── handlebars/         # NEW: Isolated Handlebars implementation
│   ├── mod.rs          # Handlebars module exports
│   ├── types.rs        # Move existing Template<T> here
│   ├── context.rs      # Move existing TemplateContext here
│   ├── helpers.rs      # Move existing custom helpers here
│   └── constants.rs    # Move template constants here
└── (future tera files)
```

#### 1.2 Move Existing Handlebars Code

- Move `types.rs` → `handlebars/types.rs`
- Move `context.rs` → `handlebars/context.rs`
- Move `helpers.rs` → `handlebars/helpers.rs`
- Move relevant constants to `handlebars/constants.rs`

#### 1.3 Update Module Exports

- Update `mod.rs` to re-export from `handlebars/` subdirectory
- Ensure all existing imports still work
- **CRITICAL**: Run full test suite to confirm nothing broke

### Phase 2: Tera Foundation Setup (Low Risk)

**Duration**: 1-2 days

#### 2.1 Add Tera Dependencies

```toml
# Cargo.toml
tera = "1.19"
serde_json = "1.0"  # For template context serialization
```

#### 2.2 Create Tera Module Structure

```
src/cli/utils/template/
├── mod.rs              # Module exports (both handlebars and tera)
├── handlebars/         # EXISTING: Working Handlebars implementation
│   ├── mod.rs
│   ├── types.rs
│   ├── context.rs
│   ├── helpers.rs
│   └── constants.rs
└── tera/               # NEW: Tera implementation
    ├── mod.rs          # Tera module exports
    ├── types.rs        # TeraTemplate struct
    ├── context.rs      # TeraTemplateContext (adapted from handlebars)
    ├── functions.rs    # Tera custom functions
    └── constants.rs    # Shared constants (if any)
```

#### 2.3 Basic Tera Integration

- Create `TeraTemplate` type alongside existing Handlebars `Template<T>`
- Implement basic rendering with Tera engine
- Set up template context conversion for Tera
- **DO NOT** replace any Handlebars usage yet

### Phase 3: Core Tera Functionality (Medium Risk)

**Duration**: 3-4 days

#### 3.1 Migrate Template Context

- Convert `TemplateContext` to be `serde::Serialize` for Tera
- Ensure all fields are Tera-compatible
- Test basic variable access: `{{ major }}.{{ minor }}.{{ patch }}`

#### 3.2 Replace Built-in Functionality

- **Math operations**: Replace `{{add a b}}` with `{{ a + b }}`
- **String operations**: Use built-in filters where possible
- **Default values**: Replace custom helpers with `| default(value=0)`
- **Timestamp formatting**: Use `| date(format="...")` filter

#### 3.3 Migrate Simple Templates

- Basic version templates: `{{major}}.{{minor}}.{{patch}}`
- Simple conditional logic
- Variable access patterns

### Phase 4: Custom Function Migration (High Risk)

**Duration**: 4-5 days

#### 4.1 Implement Tera Custom Functions

Map Handlebars helpers to Tera functions:

| Handlebars Helper  | Tera Function                                         | Complexity |
| ------------------ | ----------------------------------------------------- | ---------- |
| `sanitize`         | `sanitize(value, preset="dotted")`                    | Medium     |
| `hash`             | `hash(value, length=7)`                               | Low        |
| `hash_int`         | `hash_int(value, length=7, allow_leading_zero=false)` | Medium     |
| `prefix`           | `prefix(value, length)`                               | Low        |
| `format_timestamp` | Use built-in `date` filter                            | Trivial    |

#### 4.2 Migrate Sanitization Logic

- Convert sanitization presets to Tera function parameters
- Preserve all existing functionality
- Map preset names: `"semver"`, `"pep440"`, `"uint"`

#### 4.3 Hash Function Implementation

- Implement as simple Tera functions
- Preserve all edge cases and validation
- Test length limits and formatting

### Phase 5: Integration and Parallel Testing (Critical)

**Duration**: 2-3 days

#### 5.1 Side-by-Side Testing

- Create test harness that runs templates with both Handlebars and Tera
- Validate identical output for all existing templates
- **DO NOT** replace any production usage yet

#### 5.2 CLI Integration Testing

- Add feature flag or environment variable to switch between engines
- Test all CLI commands with both template engines
- Performance comparison testing

#### 5.3 Comprehensive Test Migration

- Migrate all existing template tests (430 lines) to work with Tera
- Ensure 100% test coverage maintained for both engines
- Add Tera-specific tests for new functionality

### Phase 6: Handlebars Removal (Final Step)

**Duration**: 1 day

#### 6.1 Replace All Handlebars Usage

- Update all CLI code to use Tera instead of Handlebars
- Remove Handlebars dependency from Cargo.toml
- Update module exports in `mod.rs`

#### 6.2 Delete Handlebars Directory

- Remove `src/cli/utils/template/handlebars/` entirely
- Clean up any remaining references

#### 6.3 Final Validation

- Run complete test suite
- Test all CLI commands end-to-end
- Performance validation
- **CRITICAL**: Ensure nothing broke after Handlebars removal

## Implementation Details

### Template Syntax Migration

#### Handlebars → Tera Mapping

| Handlebars                             | Tera                                      | Notes                           |
| -------------------------------------- | ----------------------------------------- | ------------------------------- | --------------- |
| `{{add major minor}}`                  | `{{ major + minor }}`                     | Built-in math                   |
| `{{sanitize branch preset='dotted'}}`  | `{{ sanitize(branch, preset="dotted") }}` | Custom function                 |
| `{{hash branch 7}}`                    | `{{ hash(branch, length=7) }}`            | Custom function                 |
| `{{prefix branch 10}}`                 | `{{ branch                                | truncate(length=10, end="") }}` | Built-in filter |
| `{{format_timestamp ts}}`              | `{{ ts                                    | date(format="%Y-%m-%d") }}`     | Built-in filter |
| `{{#if post}}{{post}}{{else}}0{{/if}}` | `{{ post                                  | default(value=0) }}`            | Built-in filter |

### Code Structure Changes

#### Template Type Migration

```rust
// Current (Handlebars)
pub struct Template<T> {
    template: String,
    _phantom: PhantomData<T>,
}

impl<T> Template<T> where T: serde::Serialize {
    pub fn render(&self, context: &T) -> Result<String, ZervError> {
        // Handlebars rendering logic
    }
}

// New (Tera)
pub struct TeraTemplate {
    template: String,
    tera: Tera,
}

impl TeraTemplate {
    pub fn render(&self, context: &impl serde::Serialize) -> Result<String, ZervError> {
        // Tera rendering logic
    }
}
```

#### Custom Function Implementation

```rust
// Tera custom function (much simpler than Handlebars helper)
fn sanitize_fn(args: &HashMap<String, Value>) -> Result<Value, Error> {
    let value = try_get_value!("sanitize", "value", String, args);
    let preset = args.get("preset")
        .map(|v| try_get_value!("sanitize", "preset", String, v))
        .unwrap_or_else(|| "dotted".to_string());

    // Use existing sanitization logic
    let sanitized = match preset.as_str() {
        "semver" => Sanitizer::semver_str().sanitize(&value),
        "pep440" => Sanitizer::pep440_local_str().sanitize(&value),
        // ... other presets
        _ => return Err("Unknown preset".into()),
    };

    Ok(to_value(&sanitized)?)
}
```

## Risk Assessment

### High Risk Areas

1. **Custom helper complexity**: Some helpers have complex validation logic
2. **Template syntax differences**: Some templates may need updates
3. **Error handling patterns**: Different error types and handling
4. **Performance characteristics**: Need to validate performance

### Medium Risk Areas

1. **Test migration**: Large test suite to migrate
2. **Integration points**: CLI integration code changes
3. **Context serialization**: Ensuring all data is Tera-compatible

### Low Risk Areas

1. **Basic template rendering**: Core functionality is straightforward
2. **Simple variable access**: Direct mapping
3. **Built-in functionality**: Tera has more features, not fewer

## Benefits Analysis

### Code Reduction

- **Custom helpers**: 1006 → ~300 lines (70% reduction)
- **Parameter utilities**: 40 → 0 lines (100% elimination)
- **Simple templates**: More expressive, less verbose
- **Overall template code**: ~2000 → ~1200 lines (40% reduction)

### Functionality Improvements

- **Default values**: `| default(value=0)` vs custom helper
- **Math operations**: `{{ a + b * c }}` vs `{{multiply (add a b) c}}`
- **String operations**: Built-in filters vs custom implementations
- **Conditional logic**: More powerful `{% if %}` statements
- **Date formatting**: Built-in vs custom helper

### Maintainability

- **Simpler API**: Functions vs trait implementations
- **Better debugging**: Clearer error messages
- **Expressive templates**: More logic in templates, less in code
- **Standard syntax**: Jinja2-like syntax, better known

## Migration Timeline

**Total Duration**: 12-16 days

| Week           | Activities                  | Deliverables                                     |
| -------------- | --------------------------- | ------------------------------------------------ |
| 1 (Day 1)      | Phase 1: Isolate Handlebars | Handlebars moved to subdirectory, all tests pass |
| 1 (Days 2-3)   | Phase 2: Tera Foundation    | Tera setup alongside Handlebars, basic rendering |
| 1 (Days 4-6)   | Phase 3: Core Tera          | Math, defaults, basic functionality working      |
| 2 (Days 7-10)  | Phase 4: Custom Functions   | All helpers migrated to Tera                     |
| 2 (Days 11-13) | Phase 5: Parallel Testing   | Side-by-side validation, feature flag switching  |
| 2 (Day 14)     | Phase 6: Handlebars Removal | Handlebars deleted, Tera fully integrated        |
| 2 (Days 15-16) | Documentation & cleanup     | Migration guide, code cleanup                    |

## Success Criteria

### Functional Requirements

- [ ] Phase 1: Handlebars isolated, all existing tests pass
- [ ] Phase 2: Tera foundation working alongside Handlebars
- [ ] Phase 3: Basic Tera functionality matches Handlebars output
- [ ] Phase 4: All custom helpers successfully migrated to Tera
- [ ] Phase 5: Side-by-side testing shows identical output for both engines
- [ ] Phase 6: Handlebars removed, Tera fully functional
- [ ] 100% test coverage maintained throughout migration
- [ ] CLI commands produce identical output after migration
- [ ] Performance equal or better than Handlebars

### Code Quality Requirements

- [ ] At least 40% reduction in template-related code
- [ ] Simplified helper implementation
- [ ] Better error messages and debugging
- [ ] Comprehensive documentation

### Migration Requirements

- [ ] Zero risk approach: isolate first, then build, then replace
- [ ] All tests pass at each phase before proceeding
- [ ] Side-by-side validation before any replacement
- [ ] Rollback plan at each phase (revert to previous working state)
- [ ] Feature flag for switching between engines during parallel testing

## Safety Checks and Rollback Strategy

### Phase-by-Phase Rollback Plans

- **Phase 1**: If isolation breaks anything, simply move files back
- **Phase 2**: If Tera setup fails, remove tera/ directory, keep handlebars/
- **Phase 3**: If core functionality has issues, disable Tera, continue with Handlebars
- **Phase 4**: If custom functions fail, keep Handlebars as primary, debug Tera separately
- **Phase 5**: If parallel testing fails, investigate without breaking either engine
- **Phase 6**: If removal breaks anything, restore handlebars/ directory immediately

### Critical Validation Points

1. **After Phase 1**: All existing tests must pass
2. **After Phase 3**: Basic templates must produce identical output
3. **After Phase 5**: Side-by-side validation must show 100% match
4. **After Phase 6**: Full test suite must pass after Handlebars removal

## Next Steps

1. **Review and approve this updated migration plan**
2. **Create branch `feature/handlebars-to-tera-migration`**
3. **Begin with Phase 1: Isolate Handlebars (zero risk)**
4. **Run full test suite after each phase**
5. **Only proceed to next phase after complete validation**
6. **Final cleanup and documentation**

## Conclusion

This updated migration plan uses a **zero-risk approach** by:

1. **Isolating existing Handlebars code first** (no functional changes)
2. **Building Tera alongside** Handlebars (parallel development)
3. **Validating side-by-side** before any replacement
4. **Only removing Handlebars** after Tera is fully proven

The migration addresses your core pain points while maintaining complete safety:

- **Eliminates the rabbit hole** of custom helper implementations
- **Provides built-in solutions** like `{{ post | default(value=0) }}`
- **Reduces code complexity** by ~40% in template-related areas
- **Improves template expressiveness** with more powerful syntax
- **Zero risk migration** with rollback at every phase

Given the current 1000+ lines of custom helper code and your frustration with Handlebars limitations, this migration represents a strategic investment in code maintainability and developer productivity with maximum safety.

**Recommendation**: Proceed with this safer, phased migration approach.
