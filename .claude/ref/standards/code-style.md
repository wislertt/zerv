# Code Style Standards

## Comment Policy

**Golden Rule**: If a comment just repeats what the code or function name already says, DELETE IT.

**Only comment when**:

- Explaining complex algorithms or non-obvious behavior
- Documenting domain-specific rules (e.g., "PEP440 ordering: dev < alpha < beta < rc")
- Clarifying surprising edge cases

**Never**:

- Restate function names: `/// Converts VCS data` for `fn vcs_data_to_zerv_vars()`
- Obvious comments: `// Initialize repo`, `// Return value`
- Section dividers: `// ======== Tests ========` (use `mod` blocks instead)

## Import Statement Policy

**Always place `use` statements at the top of the file or module, never inside functions.**

Exception: Rare naming conflicts requiring scope limitation (`use X as Y`)

## Test Organization Policy

**Use Rust modules for test organization, not comment-based grouping.**

```rust
// ✅ GOOD - Module-based
mod feature_basic {
    use super::*;
    #[test]
    fn test_something() { }
}

// ❌ BAD - Comment-based
// ============ Feature Basic Tests ============
#[test]
fn test_something() { }
```

## Line Length Policy

**Keep lines reasonably short when adding/updating code.**

Rustfmt enforces `max_width = 100` for code but can't break string literals. For long strings, use `format!()` or string continuation. Check with `/audit-all` periodically.
