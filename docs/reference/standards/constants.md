# Constants Usage - MANDATORY

**NEVER use bare string literals for field names, formats, sources, or schema names.**

## Available Constants

```rust
use crate::utils::constants::{fields, formats, sources, schema_names};

// Field names: fields::MAJOR, fields::MINOR, fields::PATCH, etc.
// Format names: formats::SEMVER, formats::PEP440, formats::ZERV
// Source names: sources::GIT, sources::STDIN
// Schema names: schema_names::ZERV_STANDARD, schema_names::ZERV_CALVER
```

## Example

```rust
// ✅ GOOD
use crate::utils::constants::{fields, formats};

match field_name.as_str() {
    fields::MAJOR => handle_major(),
    fields::MINOR => handle_minor(),
    _ => return Err(ZervError::UnknownField(field_name.to_string())),
}

// ❌ BAD
match field_name.as_str() {
    "major" => handle_major(),
    "minor" => handle_minor(),
    _ => return Err(ZervError::UnknownField(field_name.to_string())),
}
```
