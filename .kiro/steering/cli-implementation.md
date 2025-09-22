---
inclusion: fileMatch
fileMatchPattern: "src/cli/**/*.rs"
---

# CLI Implementation Standards

## Core Commands

### `zerv version [OPTIONS]`

Main version processing pipeline with composable operations.

### `zerv check <version> [OPTIONS]`

Validation-only command for version strings.

## Pipeline Architecture

```
Input → Version Object → Zerv Object → Transform → Output Version Object → Display
```

## Key Implementation Patterns

### Version Command Args Structure

```rust
#[derive(Parser)]
struct VersionArgs {
    version: Option<String>,
    #[arg(long, default_value = "git")]
    source: String,
    #[arg(long, default_value = "zerv-default")]
    schema: String,
    #[arg(long)]
    schema_ron: Option<String>,
    #[arg(long)]
    output_format: Option<String>,
}
```

### State-Based Versioning Tiers

**Tier 1** (Tagged, clean): `major.minor.patch`
**Tier 2** (Distance, clean): `major.minor.patch.post<distance>+branch.<commit>`
**Tier 3** (Dirty): `major.minor.patch.dev<timestamp>+branch.<commit>`

### Format Flag Validation Pattern

```rust
// Error if conflicting format flags used
if args.format.is_some() && (args.input_format.is_some() || args.output_format.is_some()) {
    return Err(ZervError::ConflictingFlags(
        "Cannot use --format with --input-format or --output-format".to_string()
    ));
}
```
