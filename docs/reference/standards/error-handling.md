# Error Handling Standards

## Rules

1. **ALWAYS use `zerv::error::ZervError`** for custom errors
2. **Use `io::Error::other()`** instead of `io::Error::new(io::ErrorKind::Other, ...)`
3. **Include context** in error messages (what failed, where, why)
4. **NEVER use `unwrap()` or `expect()`** in production code (only in tests)

## Examples

```rust
// ✅ GOOD
let file = fs::read_to_string(&path)
    .map_err(|e| ZervError::Io(io::Error::other(
        format!("Failed to read config file at {}: {}", path.display(), e)
    )))?;

// ✅ GOOD - Test code with context
let fixture = GitRepoFixture::tagged("v1.0.0")
    .expect("Failed to create tagged repo - check Docker availability");

// ❌ BAD - Generic error
let file = fs::read_to_string(&path)?;

// ❌ BAD - Old pattern
return Err(ZervError::Io(io::Error::new(io::ErrorKind::Other, "failed")));
```
