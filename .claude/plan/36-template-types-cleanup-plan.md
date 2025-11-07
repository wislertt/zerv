# Template Types Cleanup Plan

**Status**: Planned
**Priority**: High
**Created**: 2025-11-02
**Author**: Claude Code

## Context

The current `src/cli/utils/template/types.rs` file is a mess with redundant methods and confusing architecture:

### Current Problems

1. **Redundant creation methods**: `new()` and `new_safe()` do essentially the same thing
2. **Duplicate accessor methods**: `content()` and `as_str()` are identical
3. **Double function registration**: Functions are registered in both `new_safe()` AND `render_template()`
4. **Multiple render methods**: `render()`, `render_with_context()`, `render_template()`, `resolve()` - confusing overlap
5. **Lazy initialization anti-pattern**: Creating new Tera instances on every render instead of caching
6. **Unnecessary PhantomData**: The generic `T` parameter is only used for the `resolve()` method

### Current Issues Analysis

```rust
// REDUNDANT: new() just calls new_safe() and panics on error
pub fn new(template: String) -> Self {
    Self::new_safe(template).expect("Invalid template string")
}

// REDUNDANT: content() and as_str() do the exact same thing
pub fn content(&self) -> &str { &self.template }
pub fn as_str(&self) -> &str { &self.template }

// INEFFICIENT: Functions registered twice (creation + render)
pub fn new_safe(template: String) -> Result<Self, ZervError> {
    let mut tera = tera::Tera::default();
    register_functions(&mut tera)?;  // 1st registration
    // ...
}
fn render_template(template: &str, zerv: Option<&Zerv>) -> Result<String, ZervError> {
    let mut tera = tera::Tera::default();
    register_functions(&mut tera)?;  // 2nd registration - wasteful!
    // ...
}
```

## Goal

Simplify the Template struct to have:

1. **Single creation method** that only stores the template string
2. **Single render method** with signature `(&self, zerv: Option<&Zerv>) -> Result<Option<T>, ZervError>`
3. **No redundant accessors** or duplicate functionality
4. **Efficient caching** - register functions once, reuse Tera instance

## Proposed New Architecture

### Simplified Template<T> Structure

```rust
#[derive(Debug, Clone)]
pub struct Template<T> {
    template: String,
    _cached_tera: OnceCell<tera::Tera>,  // Lazy initialization
    _phantom: PhantomData<T>,
}

impl<T> Template<T>
where
    T: FromStr + Clone + Display,
    T::Err: Display,
{
    /// Create a new template from string (pure storage, no validation yet)
    pub fn new(template: String) -> Self {
        Self {
            template,
            _cached_tera: OnceCell::new(),
            _phantom: PhantomData,
        }
    }

    /// Get template content
    pub fn as_str(&self) -> &str {
        &self.template
    }

    /// Render template and parse to typed result
    pub fn render(&self, zerv: Option<&Zerv>) -> Result<Option<T>, ZervError> {
        let rendered = self.render_string(zerv)?;

        // Handle empty/null results
        let trimmed = rendered.trim().to_lowercase();
        if trimmed.is_empty() || matches!(trimmed.as_str(), "none" | "null" | "nil") {
            return Ok(None);
        }

        // Parse to target type
        let parsed = rendered
            .parse::<T>()
            .map_err(|e| ZervError::TemplateError(format!("Failed to parse '{rendered}': {e}")))?;
        Ok(Some(parsed))
    }

    /// Internal method: get or create cached Tera instance
    fn get_tera(&self) -> Result<&tera::Tera, ZervError> {
        self._cached_tera.get_or_try_init(|| {
            let mut tera = tera::Tera::default();
            register_functions(&mut tera)?;  // Register only once!
            tera.add_raw_template("template", &self.template)
                .map_err(|e| ZervError::TemplateError(
                    format!("Failed to parse template '{}': {}", self.template, e)
                ))?;
            Ok(tera)
        })
    }

    /// Internal method: render to string
    fn render_string(&self, zerv: Option<&Zerv>) -> Result<String, ZervError> {
        let tera = self.get_tera()?;
        let context = self.create_context(zerv)?;

        tera.render("template", &context)
            .map(|s| s.trim().to_string())
            .map_err(|e| ZervError::TemplateError(
                format!("Template render error '{}': {}", self.template, e)
            ))
    }

    /// Create template context from Zerv object
    fn create_context(&self, zerv: Option<&Zerv>) -> Result<tera::Context, ZervError> {
        if let Some(z) = zerv {
            let template_context = ZervTemplateContext::from_zerv(z);
            tera::Context::from_serialize(template_context)
                .map_err(|e| ZervError::TemplateError(format!("Serialization error: {e}")))
        } else {
            Ok(tera::Context::new())
        }
    }
}
```

## Key Improvements

### 1. Eliminated Redundancy

- **Removed**: `new_safe()`, `content()`, `render_with_context()`, `render_template()`, `resolve()`
- **Simplified**: Single `new()` method, single `render()` method, single `as_str()` accessor

### 2. Efficient Caching

- **OnceCell**: Lazily create Tera instance once and reuse
- **Single function registration**: Functions registered only when first needed
- **Better performance**: No repeated parsing/registration

### 3. Cleaner API

- **One creation method**: `Template::new(template_string)`
- **One render method**: `template.render(Some(zerv))` returns `Result<Option<T>, ZervError>`
- **One accessor**: `template.as_str()` to get raw template

### 4. Better Error Handling

- **Context preservation**: Template string included in all error messages
- **Proper error types**: Using `ZervError::TemplateError` consistently
- **Clear failure points**: Obvious where things can fail

## Migration Strategy

### Phase 1: Update Template Implementation ✅

- Replace current struct with simplified version
- Remove all redundant methods
- Implement OnceCell caching
- Add comprehensive tests

### Phase 2: Update All Call Sites

- Find all uses of removed methods and update to new API
- Update `resolve()` calls to use `render()`
- Update `content()` calls to use `as_str()`
- Update `new_safe()` calls to use `new()` (handle errors properly)

### Phase 3: Validation

- Run full test suite to ensure no regressions
- Validate performance improvements
- Ensure all template functionality works identically

## Benefits

### Code Reduction

- **Lines of code**: ~555 → ~200 lines (64% reduction)
- **Method count**: 12+ → 5 methods (58% reduction)
- **Complexity**: Much simpler mental model

### Performance Improvements

- **No repeated parsing**: Template parsed once on first render
- **No repeated function registration**: Functions registered once
- **Cached Tera instance**: Reused across multiple renders
- **Expected**: 2-3x faster than current implementation

### Maintainability

- **Clear responsibility**: Each method has one clear purpose
- **No duplication**: Single source of truth for each operation
- **Easier testing**: Fewer methods to test, simpler interactions
- **Better debugging**: Clearer failure modes and error messages

## Files to Modify

1. **`src/cli/utils/template/types.rs`**
    - Replace entire implementation with simplified version
    - Keep same public API surface (for backward compatibility during transition)

2. **`src/cli/utils/template/context.rs`**
    - Rename `TemplateContext` to `ZervTemplateContext`
    - Update all struct definitions and implementations

3. **Tests in types.rs**
    - Update test methods to use new API
    - Add tests for caching behavior
    - Validate performance improvements

4. **Files that use removed methods**
    - Search for calls to `resolve()`, `content()`, `new_safe()`, etc.
    - Update to use new `render()` method
    - Handle error cases properly

5. **Files that import TemplateContext**
    - Update all imports from `TemplateContext` to `ZervTemplateContext`
    - Update usage throughout the codebase

## Success Criteria

- [ ] Template functionality works identically to current implementation
- [ ] Performance is equal or better (target: 2x improvement)
- [ ] All tests pass after migration
- [ ] Code is significantly simpler and more maintainable
- [ ] No regression in template rendering capabilities

## Risk Assessment

### Low Risk

- **Functional equivalence**: Same template behavior, just cleaner implementation
- **Backward compatibility**: Can keep old methods as deprecated during transition
- **Simple changes**: Mostly removing code, not adding complex logic

### Medium Risk

- **Call site updates**: Need to find all uses of removed methods
- **Error handling**: Need to ensure errors are handled properly in new API

### Rollback Plan

- Keep current implementation in separate branch
- If issues arise, can quickly revert and add deprecation warnings instead
- Migration is mostly deletions, so easily reversible

## Implementation Plan

1. **Create new simplified implementation** alongside current one
2. **Add comprehensive tests** for new implementation
3. **Update all call sites** to use new API
4. **Remove old implementation** and deprecated methods
5. **Run full test suite** and performance benchmarks
6. **Final validation** and cleanup

The result will be a much cleaner, more efficient, and more maintainable template system.
