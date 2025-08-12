# Zerv - Rust Rewrite of Dunamai - Development Plan

**Project Name**: `zerv` (CLI: `zerv`)
**Description**: Dynamic version generation from VCS tags - A fast Rust rewrite of Dunamai

## Project Structure

```
zerv/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── version/
│   │   ├── mod.rs           # Version module
│   │   ├── core.rs          # Version struct and core logic
│   │   ├── parser.rs        # Version string parsing
│   │   └── serializer.rs    # Version formatting/serialization
│   ├── vcs/
│   │   ├── mod.rs           # VCS module
│   │   ├── traits.rs        # VCS trait definitions
│   │   ├── git.rs           # Git implementation
│   │   ├── mercurial.rs     # Mercurial implementation
│   │   ├── darcs.rs         # Darcs implementation
│   │   ├── subversion.rs    # SVN implementation
│   │   ├── bazaar.rs        # Bazaar implementation
│   │   ├── fossil.rs        # Fossil implementation
│   │   └── pijul.rs         # Pijul implementation
│   ├── styles/
│   │   ├── mod.rs           # Style module
│   │   ├── pep440.rs        # PEP 440 style
│   │   ├── semver.rs        # Semantic versioning
│   │   └── pvp.rs           # Haskell PVP style
│   ├── patterns/
│   │   └── mod.rs           # Pattern module
│   ├── template/
│   │   ├── mod.rs           # Template module
│   │   ├── engine.rs        # Template engine wrapper
│   │   └── variables.rs     # Template variable extraction
│   ├── config/
│   │   ├── mod.rs           # Config module
│   │   └── loader.rs        # Config file loading
│   ├── archive/
│   │   ├── mod.rs           # Archive handling
│   │   ├── git.rs           # Git archive metadata
│   │   └── mercurial.rs     # Mercurial archive metadata
│   ├── cli/
│   │   ├── mod.rs           # CLI module
│   │   ├── args.rs          # Argument parsing
│   │   └── commands.rs      # Command implementations
│   ├── error.rs             # Error types and handling
│   └── utils.rs             # Utility functions
├── tests/
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── git_tests.rs
│   │   └── version_tests.rs
│   └── unit/
│       ├── mod.rs
│       ├── version_tests.rs
│       └── vcs_tests.rs
└── examples/
    ├── basic_usage.rs
    └── custom_format.rs
```

## Core Entities

### 1. Version Struct

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub base: String,           // "1.2.3"
    pub stage: Option<String>,  // "alpha", "beta", "rc"
    pub revision: Option<u32>,  // 1, 2, 3...
    pub distance: u32,          // commits since tag
    pub commit: Option<String>, // commit hash
    pub dirty: bool,            // uncommitted changes
    pub tagged_metadata: Option<String>, // metadata from tag
    pub epoch: Option<u32>,     // PEP 440 epoch
    pub branch: Option<String>, // current branch
    pub timestamp: Option<DateTime<Utc>>, // commit timestamp
}
```

### 2. VCS Trait

```rust
pub trait Vcs {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> Result<bool>;
    fn get_version(&self, pattern: &Pattern, options: &VcsOptions) -> Result<Version>;
    fn get_distance(&self, from_tag: &str) -> Result<u32>;
    fn get_commit_hash(&self, short: bool) -> Result<String>;
    fn is_dirty(&self) -> Result<bool>;
    fn get_branch(&self) -> Result<Option<String>>;
}
```

### 3. Style Enum

```rust
#[derive(Debug, Clone, Copy)]
pub enum Style {
    Pep440,
    SemVer,
    Pvp,
    Custom(CustomStyle),
}
```

### 4. Pattern Struct

```rust
#[derive(Debug, Clone)]
pub struct Pattern {
    pub regex: Regex,
    pub prefix: Option<String>,
}
```

**Auto-detection**: The Rust version uses auto-detection by trying multiple built-in patterns in order, rather than requiring users to specify pattern names. This provides better user experience while maintaining flexibility for custom regex patterns when needed.

### 5. Error Handling Strategy (Ripgrep-Style)

**Library Layer** - Specific error types for clean API:

**Implementation**: Manual trait implementations (zero dependencies) with comprehensive error variants, Display/Error traits, From conversions, and full test coverage.

**CLI Layer** - Uses `anyhow` for easy error propagation:

```rust
// CLI functions use anyhow::Result for convenience
fn main() -> anyhow::Result<()> {
    let version = zerv::get_version_from_git(".")?; // ZervError -> anyhow::Error
    println!("{}", version.serialize(Style::SemVer));
    Ok(())
}
```

## Development Phases

### Phase 1: Core Foundation

- [x] Set up Cargo project with dependencies
- [x] Implement `Version` struct with basic methods
- [x] Add basic unit tests for Version struct
- [x] Create ZervError types.
- [ ] Add version parsing from string (simple cases)
- [ ] Implement basic pattern matching (default patterns only)
- [ ] Create basic CLI skeleton (main.rs + lib.rs structure)

**Dependencies:**

```toml
[package]
name = "zerv"
version = "0.1.0"
description = "Dynamic version generation from VCS tags"

[[bin]]
name = "zerv"
path = "src/main.rs"

[dependencies]
# Core library dependencies
regex = "1.0"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }

# CLI-specific dependencies (optional for library users)
anyhow = { version = "1.0", optional = true }  # CLI error handling
clap = { version = "4.0", features = ["derive"], optional = true }
toml = { version = "0.8", optional = true }     # Config file parsing
dirs = { version = "5.0", optional = true }     # Config directory detection
tera = { version = "1.0", optional = true }     # Template engine

[features]
default = ["cli"]
cli = ["anyhow", "clap", "toml", "dirs", "tera"]  # CLI features optional for lib users
```

### Phase 2: Git VCS Implementation

- [ ] Implement `Vcs` trait
- [ ] Add command execution utilities (run_command function)
- [ ] Create basic Git VCS implementation
- [ ] Implement Git tag discovery
- [ ] Add Git distance calculation
- [ ] Add Git dirty detection
- [ ] Add Git-specific tests

### Phase 3: Version Serialization

- [ ] Implement PEP 440 serialization (serialize_pep440)
- [ ] Implement SemVer serialization (serialize_semver)
- [ ] Implement PVP serialization (serialize_pvp)
- [ ] Add style validation functions
- [ ] Add serialization tests
- [ ] Basic Version.serialize() method

### Phase 4: Basic CLI Commands

- [ ] Implement CLI argument parsing with `clap`
- [ ] Set up CLI layer with `anyhow` error handling
- [ ] Add `from git` command (basic functionality)
- [ ] Add `check` command for version validation
- [ ] Add basic output formatting
- [ ] Add error handling and user-friendly messages
- [ ] Add CLI integration tests

### Phase 5: Additional VCS Support

- [ ] Implement Mercurial support
- [ ] Implement Darcs support
- [ ] Implement Subversion support
- [ ] Add `from any` auto-detection
- [ ] Comprehensive VCS testing

### Phase 6: Archive Support

- [ ] Implement Git archive metadata parsing
- [ ] Implement Mercurial archive support
- [ ] Add archive detection logic
- [ ] Test with various archive formats

### Phase 7: Advanced Features

- [ ] Implement template engine integration
- [ ] Add template variable extraction
- [ ] Implement configuration system (TOML files)
- [ ] Add environment variable support
- [ ] Implement override state functionality
- [ ] Implement bump functionality
- [ ] Add branch name escaping
- [ ] Implement remaining VCS systems (Bazaar, Fossil, Pijul)
- [ ] Performance optimizations

### Phase 8: Polish & Documentation

- [ ] Comprehensive documentation
- [ ] Performance benchmarks vs Python version
- [ ] Error message improvements
- [ ] Final testing and bug fixes
- [ ] Release preparation

## Key Implementation Details

### Error Handling Architecture

**Library Functions** (return specific errors):

```rust
// Library API - specific error types
pub fn get_version_from_git(path: &Path) -> Result<Version, ZervError> {
    let output = run_command("git", &["describe", "--tags"], Some(path))?;
    parse_version(&output)
}

fn run_command(cmd: &str, args: &[&str], cwd: Option<&Path>) -> Result<String, ZervError> {
    let mut command = Command::new(cmd);
    command.args(args);

    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    let output = command.output()?; // io::Error -> ZervError::Io

    if !output.status.success() {
        return Err(ZervError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

**CLI Functions** (use anyhow for convenience):

```rust
// CLI layer - anyhow for easy error handling
fn cli_main() -> anyhow::Result<()> {
    let version = zerv::get_version_from_git(".")
        .context("Failed to get version from git")?; // ZervError -> anyhow::Error

    let config = load_config()
        .context("Failed to load config")?; // Any error -> anyhow::Error

    println!("{}", version.serialize(config.style));
    Ok(())
}
```

### Version Serialization

```rust
impl Version {
    pub fn serialize(&self, style: Style, metadata: bool, dirty: bool) -> String {
        match style {
            Style::Pep440 => self.serialize_pep440(metadata, dirty),
            Style::SemVer => self.serialize_semver(metadata, dirty),
            Style::Pvp => self.serialize_pvp(metadata, dirty),
            Style::Custom(format) => self.serialize_custom(&format),
        }
    }
}
```

### Pattern Matching

```rust
impl Pattern {
    pub fn default() -> Self {
        // Equivalent to Python's VERSION_SOURCE_PATTERN
        let pattern = r"(?x)
            ^v((?P<epoch>\d+)!)?
            (?P<base>\d+(\.\d+)*)
            ([-._]?((?P<stage>[a-zA-Z]+)[-._]?(?P<revision>\d+)?))?
            (\+(?P<tagged_metadata>.+))?$";

        Self {
            regex: Regex::new(pattern).unwrap(),
            prefix: Some("v".to_string()),
        }
    }

    pub fn default_unprefixed() -> Self {
        // Matches 1.2.3 format (no v prefix)
        let pattern = r"(?x)
            ^((?P<epoch>\d+)!)?
            (?P<base>\d+(\.\d+)*)
            ([-._]?((?P<stage>[a-zA-Z]+)[-._]?(?P<revision>\d+)?))?
            (\+(?P<tagged_metadata>.+))?$";

        Self {
            regex: Regex::new(pattern).unwrap(),
            prefix: None,
        }
    }

    pub fn from_custom_regex(pattern: &str) -> Result<Self> {
        // Only accept custom regex patterns with named groups
        if !pattern.contains("?P<base>") {
            return Err(ZervError::InvalidFormat(
                "Custom pattern must contain named group ?P<base>".to_string()
            ));
        }

        let regex = Regex::new(pattern)?;
        Ok(Self { regex, prefix: None })
    }

    pub fn try_all_builtin_patterns(tag: &str) -> Option<(Self, VersionComponents)> {
        // Try built-in patterns in order of preference
        let patterns = vec![
            Self::default(),           // v1.2.3
            Self::default_unprefixed(), // 1.2.3
            // Add more common patterns here
        ];

        for pattern in patterns {
            if let Some(components) = pattern.extract_version(tag) {
                return Some((pattern, components));
            }
        }
        None
    }

    pub fn extract_version(&self, tag: &str) -> Option<VersionComponents> {
        self.regex.captures(tag).map(|caps| {
            VersionComponents {
                base: caps.name("base").unwrap().as_str().to_string(),
                stage: caps.name("stage").map(|m| m.as_str().to_string()),
                revision: caps.name("revision").and_then(|m| m.as_str().parse().ok()),
                tagged_metadata: caps.name("tagged_metadata").map(|m| m.as_str().to_string()),
                epoch: caps.name("epoch").and_then(|m| m.as_str().parse().ok()),
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct VersionComponents {
    pub base: String,
    pub stage: Option<String>,
    pub revision: Option<u32>,
    pub tagged_metadata: Option<String>,
    pub epoch: Option<u32>,
}
```

## Testing Strategy

### Unit Tests

- Version parsing and serialization
- Pattern matching
- Style formatting
- Error handling

### Integration Tests

- Full VCS workflows
- CLI command testing
- Archive handling
- Cross-platform compatibility

### Performance Tests

- Benchmark against Python version
- Memory usage comparison
- Command execution overhead

## Success Metrics

1. **Functionality**: 100% feature parity with Python version
2. **Performance**: 2-5x faster than Python version
3. **Safety**: Zero command injection vulnerabilities
4. **Usability**: Drop-in replacement for existing users
5. **Maintainability**: Clean, idiomatic Rust code

## Risk Mitigation

1. **VCS Complexity**: Start with Git only, add others incrementally
2. **Regex Complexity**: Use existing Python regexes as reference
3. **Cross-platform**: Test on Windows, macOS, Linux from early phases
4. **Performance**: Profile early and often

## Project Branding

- **Full Name**: `zerv`
- **CLI Command**: `zerv`
- **Tagline**: "Zerv: Dynamic versioning CLI"
- **Usage Examples**:
    ```bash
    zerv from git
    zerv from any --bump
    zerv check 1.2.3 --style semver
    zerv from git --format "v{base}+{distance}.{commit}"
    ```

This plan provides a structured approach to rewriting Dunamai as `zerv` in Rust while maintaining all functionality and improving performance and security.
