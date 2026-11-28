# Pipeline Architecture

## High-Level Flow

```
Input → VCS Detection → Version Parsing → Transformation → Format Output
```

## Detailed CLI Pipeline

```
Input → Version Object → Zerv Object → Transform → Output Version Object → Display
```

## State-Based Versioning Tiers

- **Tier 1** (Tagged, clean): `major.minor.patch`
- **Tier 2** (Distance, clean): `major.minor.patch.post<distance>+branch.<commit>`
- **Tier 3** (Dirty): `major.minor.patch.dev<timestamp>+branch.<commit>`

## Test Infrastructure

- **Environment-aware Git testing**: Uses `DockerGit` locally, `NativeGit` in CI
- **`GitOperations` trait**: Unified interface for both implementations
- **`GitRepoFixture`**: Creates isolated test repositories with specific states
- **`TestDir`**: Temporary directory management with automatic cleanup

## Performance Standards

- Parse 1000+ versions in <100ms
- Minimal VCS command calls (batch when possible)
- Use compiled regex patterns for speed
- Zero-copy string operations where possible
