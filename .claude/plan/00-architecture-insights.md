# Architecture Insights - Key Decisions

## Universal Version Format (Zerv)

- **Component-based format system** with variable references
- **Separation of format template from data values**
- **Support for timestamp patterns** (YYYY, MM, DD, YYYYMMDD)
- **Extensible custom variables** via HashMap
- **Status**: ✅ Fully implemented in `src/version/zerv/`

## State-Based Versioning Tiers

**Three-tier system based on repository state:**

- **Tier 1** (Tagged, clean): `major.minor.patch`
- **Tier 2** (Distance, clean): `major.minor.patch.post<distance>+branch.<commit>`
- **Tier 3** (Dirty): `major.minor.patch.dev<timestamp>+branch.<commit>`

## Multi-Format Support Strategy

- **PEP440** for Python ecosystem (`1.2.3.post7.dev0+g29045e8`)
- **SemVer** for JavaScript/Node (`1.2.3-post.7+g29045e8`)
- **Template-based** custom formats
- **Status**: ✅ PEP440 and SemVer implemented

## Error Handling Strategy

- **Library layer**: Specific error types (`ZervError`)
- **CLI layer**: User-friendly messages
- **Use `io::Error::other()` instead of `io::Error::new(io::ErrorKind::Other, ...)`**
- **Status**: ✅ Pattern established

## Testing Strategy

- **Local**: Docker Git for isolation
- **CI**: Native Git for platform testing
- **Atomic operations** using single Docker commands
- **Status**: ✅ Successfully implemented

## Performance Targets

- **Parse 1000+ versions in <100ms**
- **Minimal VCS command calls**
- **Compiled regex patterns for speed**
- **Zero-copy string operations where possible**

## Configuration System Design (Future)

```toml
# Custom schemas: Different data structure + standard format output
[schemas]
calver-schema = '(core: [VarTimestamp("YYYY"), VarTimestamp("MM"), VarField("patch")], ...)'

# Custom templates: Default schema + custom output format
[templates]
my-format = "v{{ major }}.{{ minor }}.{{ patch }}-{{ commit }}"
```

## Complementary Tool Strategy

- **semantic-release**: Official release decisions and tagging
- **zerv**: Continuous version identification for builds
- **Perfect complementary workflow** for modern CI/CD
