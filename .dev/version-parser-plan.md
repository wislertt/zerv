# Version Parser Implementation Plan

## Overview

Implement a comprehensive version parser supporting multiple versioning schemes (SemVer, PEP 440, PVP) with custom pattern support, based on dunamai's proven architecture.

## Architecture

### Core Components

1. **Pattern System** - Regex-based pattern matching with named groups
2. **Style System** - Format-specific serialization (SemVer, PEP440, PVP)
3. **Parser Engine** - Main parsing logic with fallback strategies
4. **Validation Layer** - Format-specific validation rules

### Pattern Hierarchy

```
Pattern (trait)
├── DefaultPattern (v1.2.3-alpha.1+meta)
├── DefaultUnprefixedPattern (1.2.3-alpha.1+meta)
├── CustomPattern (user-defined regex)
└── ArchivePattern (from .git_archival.json)
```

### Style Formats

```
Style (enum)
├── SemVer (1.2.3-alpha.1+build.meta)
├── Pep440 (1!1.2.3a1.post1.dev1+local.meta)
├── Pvp (1.2.3-alpha-1-meta)
└── Custom (user template)
```

## Implementation Phases

### Phase 1: Core Pattern Engine

- Regex pattern compilation and caching
- Named group extraction system
- Pattern validation framework

### Phase 2: Multi-Format Support

- SemVer parser with pre-release and build metadata
- PEP 440 parser with epoch, pre/post/dev releases
- PVP parser with dash-separated components

### Phase 3: Advanced Features

- Custom pattern support
- Archive metadata parsing
- Template-based formatting

### Phase 4: Integration

- VCS integration hooks
- Error handling and fallbacks
- Performance optimization

## Test Cases

### Basic SemVer Tests

```rust
// Core SemVer
assert_parse("1.2.3", Version::new(1, 2, 3));
assert_parse("0.0.0", Version::new(0, 0, 0));
assert_parse("10.20.30", Version::new(10, 20, 30));

// With v prefix
assert_parse("v1.2.3", Version::new(1, 2, 3));
assert_parse("V2.0.0", Version::new(2, 0, 0));

// Pre-release
assert_parse("1.2.3-alpha", Version::new(1, 2, 3).with_stage(Stage::Alpha, None));
assert_parse("1.2.3-alpha.1", Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)));
assert_parse("1.2.3-beta.2", Version::new(1, 2, 3).with_stage(Stage::Beta, Some(2)));
assert_parse("1.2.3-rc.1", Version::new(1, 2, 3).with_stage(Stage::Rc, Some(1)));

// Build metadata
assert_parse("1.2.3+build.1", Version::new(1, 2, 3).with_tagged_metadata("build.1"));
assert_parse("1.2.3-alpha.1+build.meta",
    Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)).with_tagged_metadata("build.meta"));

// Complex combinations
assert_parse("1.2.3-alpha.1+linux.x86_64",
    Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)).with_tagged_metadata("linux.x86_64"));

// Added Complex combinations
assert_parse("1.2.3-alpha.1.post.4.dev.5.x.y.z+linux.x86_64",
    Version::new(.....).....);
```

### PEP 440 Tests

```rust
// Basic PEP 440
assert_parse("1.2.3", Version::new(1, 2, 3));
assert_parse("1.2", Version::new(1, 2, 0)); // Implicit patch

// Epoch
assert_parse("1!1.2.3", Version::new(1, 2, 3).with_epoch(1));
assert_parse("2!0.1.0", Version::new(0, 1, 0).with_epoch(2));

// Pre-releases
assert_parse("1.2.3a1", Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)));
assert_parse("1.2.3alpha1", Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)));
assert_parse("1.2.3b2", Version::new(1, 2, 3).with_stage(Stage::Beta, Some(2)));
assert_parse("1.2.3beta2", Version::new(1, 2, 3).with_stage(Stage::Beta, Some(2)));
assert_parse("1.2.3rc1", Version::new(1, 2, 3).with_stage(Stage::Rc, Some(1)));

// Post releases
assert_parse("1.2.3.post1", Version::new(1, 2, 3).with_post(1));
assert_parse("1.2.3-1", Version::new(1, 2, 3).with_post(1)); // Alternative syntax

// Dev releases
assert_parse("1.2.3.dev1", Version::new(1, 2, 3).with_dev(1));

// Local versions
assert_parse("1.2.3+local.version", Version::new(1, 2, 3).with_tagged_metadata("local.version"));

// Complex PEP 440
assert_parse("1!1.2.3a1.post1.dev1+local.meta",
    Version::new(1, 2, 3)
        .with_epoch(1)
        .with_stage(Stage::Alpha, Some(1))
        .with_post(1)
        .with_dev(1)
        .with_tagged_metadata("local.meta"));


// Added Complex PEP 440
assert_parse("1!1.2.3a1.post1.dev1+local.meta.x.x.x",
    Version::new(.....).....);
```

### PVP Tests

```rust
// Basic PVP
assert_parse("1.2.3", Version::new(1, 2, 3));
assert_parse("0.1", Version::new(0, 1, 0)); // Two components

// With pre-release
assert_parse("1.2.3-alpha", Version::new(1, 2, 3).with_stage(Stage::Alpha, None));
assert_parse("1.2.3-alpha-1", Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)));
assert_parse("1.2.3-beta-2", Version::new(1, 2, 3).with_stage(Stage::Beta, Some(2)));

// With metadata
assert_parse("1.2.3-metadata", Version::new(1, 2, 3).with_tagged_metadata("metadata"));
assert_parse("1.2.3-alpha-1-linux",
    Version::new(1, 2, 3).with_stage(Stage::Alpha, Some(1)).with_tagged_metadata("linux"));
```

### Edge Cases and Error Handling

```rust
// Empty and whitespace
assert_error("");
assert_error("   ");
assert_error("\t\n");

// Invalid formats
assert_error("1");
assert_error("1.2");
assert_error("1.2.3.4.5");
assert_error("a.b.c");
assert_error("1.b.3");
assert_error("1.2.c");

// Invalid characters
assert_error("1.2.3@");
assert_error("1.2.3#invalid");
assert_error("1.2.3 extra");

// Leading zeros (invalid in SemVer)
assert_error("01.2.3");
assert_error("1.02.3");
assert_error("1.2.03");

// Negative numbers
assert_error("-1.2.3");
assert_error("1.-2.3");
assert_error("1.2.-3");

// Overflow
assert_error("4294967296.0.0"); // u32::MAX + 1
assert_error("1.4294967296.0");
assert_error("1.2.4294967296");

// Malformed pre-releases
assert_error("1.2.3-");
assert_error("1.2.3-alpha.");
assert_error("1.2.3-alpha.a");
assert_error("1.2.3-alpha.-1");

// Malformed metadata
assert_error("1.2.3+");
assert_error("1.2.3++double");
```

### Custom Pattern Tests

```rust
// Date-based versions
assert_custom_parse("2023.12.15", r"(?P<major>\d{4})\.(?P<minor>\d{1,2})\.(?P<patch>\d{1,2})",
    Version::new(2023, 12, 15));

// Git describe format
assert_custom_parse("v1.2.3-5-g1234567",
    r"v(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)-(?P<distance>\d+)-g(?P<commit>[a-f0-9]+)",
    Version::new(1, 2, 3).with_distance(5).with_commit("1234567"));

// Ubuntu-style versions
assert_custom_parse("1.2.3-1ubuntu1",
    r"(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)-(?P<revision>\d+)ubuntu(?P<ubuntu>\d+)",
    Version::new(1, 2, 3).with_revision(1).with_tagged_metadata("ubuntu1"));
```

### Archive Metadata Tests

```rust
// Git archival format
let git_archival = r#"{
    "node": "1234567890abcdef",
    "node-date": "2023-12-15T14:25:30+00:00",
    "describe-name": "v1.2.3-5-g1234567",
    "ref-names": "tag: v1.2.3, origin/main"
}"#;
assert_archive_parse(git_archival,
    Version::new(1, 2, 3)
        .with_distance(5)
        .with_commit("1234567")
        .with_timestamp(parse_timestamp("2023-12-15T14:25:30+00:00")));

// Mercurial archival format
let hg_archival = r#"repo: 1234567890abcdef
node: 1234567890abcdef
branch: default
latesttag: v1.2.3
latesttagdistance: 5
changessincelatesttag: 5"#;
assert_archive_parse(hg_archival,
    Version::new(1, 2, 3)
        .with_distance(5)
        .with_commit("1234567890abcdef")
        .with_branch("default"));
```

### Performance Tests

```rust
// Bulk parsing performance
let versions: Vec<&str> = vec![
    "1.2.3", "v2.0.0-alpha.1", "3.1.4-beta.2+build.123",
    "1!1.2.3a1.post1.dev1+local", "2023.12.15-rc.1",
    // ... 1000+ version strings
];

let start = Instant::now();
for version_str in versions {
    let _ = parse_version(version_str);
}
let duration = start.elapsed();
assert!(duration < Duration::from_millis(100)); // Should parse 1000+ versions in <100ms
```

## File Structure

```
src/version/parser/
├── mod.rs              # Public API and parser selection
├── engine.rs           # Core parsing engine
├── patterns/
│   ├── mod.rs          # Pattern trait and registry
│   ├── default.rs      # Default SemVer-like pattern
│   ├── custom.rs       # Custom regex patterns
│   └── archive.rs      # Archive metadata patterns
├── styles/
│   ├── mod.rs          # Style trait and registry
│   ├── semver.rs       # SemVer format parser
│   ├── pep440.rs       # PEP 440 format parser
│   └── pvp.rs          # PVP format parser
└── tests/
    ├── mod.rs          # Test utilities
    ├── semver_tests.rs # SemVer test cases
    ├── pep440_tests.rs # PEP 440 test cases
    ├── pvp_tests.rs    # PVP test cases
    ├── edge_cases.rs   # Error handling tests
    └── performance.rs  # Performance benchmarks
```

## Dependencies

```toml
[dependencies]
regex = "1.10"           # Pattern matching
once_cell = "1.19"       # Regex compilation caching
chrono = "0.4"           # Timestamp parsing
serde = "1.0"            # Archive metadata parsing
serde_json = "1.0"       # JSON archive support

[dev-dependencies]
criterion = "0.5"        # Performance benchmarking
proptest = "1.4"         # Property-based testing
rstest = "0.18"          # Parameterized testing
```

## Success Criteria

1. **Compatibility**: Parse all version formats that dunamai supports
2. **Performance**: Parse 1000+ versions in <100ms
3. **Accuracy**: 100% test coverage with edge cases
4. **Extensibility**: Easy to add new patterns and styles
5. **Error Handling**: Clear error messages for invalid inputs
6. **Memory Safety**: No panics, proper error propagation

## Migration Strategy

1. Keep existing `basic.rs` parser as fallback
2. Implement new parser alongside existing one
3. Add feature flag for new parser
4. Gradually migrate tests and usage
5. Remove old parser once fully validated

## Future Enhancements

- Async parsing for large version lists
- WASM compilation for web usage
- Plugin system for custom formats
- Version comparison and sorting
- Fuzzy version matching
- Version range parsing and matching
