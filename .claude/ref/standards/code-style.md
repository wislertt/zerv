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

Rustfmt enforces `max_width = 100` for code but can't break string literals.

### Breaking Long Strings

**Regular strings**: Use backslash continuation

```rust
"version --source stdin --tag-version 5.0.0 \
 --input-format semver --output-format semver"
```

**Raw strings (r#"..."#)**: Use `concat!()` macro

```rust
// ❌ WRONG - Backslash is literal in raw strings!
r#"long command \
   continuation"#

// ✅ CORRECT - Use concat!() for raw strings
concat!(
    "version --source stdin ",
    r#"--custom '{"build":"123"}' "#,
    r#"--output-template "{{custom.build}}""#
)
```

**Why this matters**: Raw strings treat backslashes literally, so `\` continuation doesn't work. Always use `concat!()` to join raw string parts.

Check with `/audit` periodically to catch violations.
