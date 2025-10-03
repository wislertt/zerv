# BumpType Precedence Optimization Plan

## Current State

- `PRECEDENCE_ORDER` array with BumpType instances
- O(n) precedence lookups using `position()` and `std::mem::discriminant`
- O(n) string-to-precedence lookups using `field_name()` matching

## Goal

Optimize to O(1) lookups while maintaining single source of truth:

- String → Index (O(1))
- BumpType → Index (O(1))
- Index → String (O(1))

## Implementation Plan

### 1. Replace PRECEDENCE_ORDER with PRECEDENCE_NAMES

```rust
/// Single source of truth - just list of names
const PRECEDENCE_NAMES: &'static [&'static str] = &[
    bump_types::EPOCH,           // 0
    bump_types::MAJOR,           // 1
    bump_types::MINOR,           // 2
    bump_types::PATCH,           // 3
    bump_types::PRE_RELEASE_LABEL, // 4
    bump_types::PRE_RELEASE_NUM, // 5
    bump_types::POST,            // 6
    bump_types::DEV,             // 7
];
```

### 2. Add O(1) Lookup HashMap

```rust
/// O(1) string -> index lookup map
static NAME_TO_INDEX: Lazy<HashMap<&'static str, usize>> = Lazy::new(|| {
    Self::PRECEDENCE_NAMES
        .iter()
        .enumerate()
        .map(|(i, &name)| (name, i))
        .collect()
});
```

### 3. Update Methods for O(1) Performance

```rust
/// O(1) precedence from BumpType
pub fn precedence(&self) -> usize {
    Self::NAME_TO_INDEX[self.field_name()]
}

/// O(1) precedence from string
pub fn precedence_from_str(component: &str) -> usize {
    Self::NAME_TO_INDEX.get(component).copied().unwrap_or(0)
}

/// O(1) get string from precedence index
pub fn str_from_precedence(index: usize) -> Option<&'static str> {
    Self::PRECEDENCE_NAMES.get(index).copied()
}

/// Create default BumpType from component name string
pub fn default_from_str(component_name: &str) -> BumpType {
    match component_name {
        bump_types::EPOCH => BumpType::Epoch(0),
        bump_types::MAJOR => BumpType::Major(0),
        bump_types::MINOR => BumpType::Minor(0),
        bump_types::PATCH => BumpType::Patch(0),
        bump_types::PRE_RELEASE_LABEL => BumpType::PreReleaseLabel(PreReleaseLabel::Alpha),
        bump_types::PRE_RELEASE_NUM => BumpType::PreReleaseNum(0),
        bump_types::POST => BumpType::Post(0),
        bump_types::DEV => BumpType::Dev(0),
        _ => panic!("Invalid component name: {}", component_name),
    }
}

```

### 4. Dependencies

Add to `Cargo.toml`:

```toml
once_cell = "1.19"
```

### 5. Update Tests

- Update existing precedence tests to work with new implementation
- Add tests for new `str_from_precedence()` method
- Verify O(1) performance characteristics

## Benefits

- **Performance**: O(n) → O(1) for all lookups
- **Memory**: Minimal overhead (single HashMap)
- **Maintainability**: Single source of truth in `PRECEDENCE_NAMES`
- **API**: Same public interface, internal optimization

### 6. Update Reset Logic

```rust
// In reset.rs - simplified reset loop:
for (index, &name) in BumpType::PRECEDENCE_NAMES.iter().enumerate() {
    if index > current_precedence {
        let default_bump = BumpType::default_from_str(name);
        self.reset_component(&default_bump);
    }
}
```

## Files to Modify

- `src/version/zerv/bump/types.rs` - Main implementation
- `src/version/zerv/bump/reset.rs` - Update reset loop
- `Cargo.toml` - Add once_cell dependency

## Testing Strategy

- All existing tests should pass
- Add performance benchmarks if needed
- Verify precedence order remains correct
