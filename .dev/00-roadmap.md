# Development Roadmap

## Current Status: Version Parsing Complete ✅

**Foundation Built**: Universal version system with PEP440 and SemVer support
**Next Phase**: Git VCS integration for dynamic version generation

## Phase 1: Git VCS Integration 🎯 **(NEXT)**

**Goal**: Enable version generation from Git repository state

### Core VCS Implementation

- [ ] `VCS` trait definition with common interface
- [ ] Git command execution utilities (`run_command` function)
- [ ] Git repository detection and validation
- [ ] Tag discovery and filtering
- [ ] Distance calculation from tag to HEAD
- [ ] Commit hash extraction (short/full)
- [ ] Dirty state detection
- [ ] Branch name extraction

### Integration with Version System

- [ ] Connect Git data to `ZervVars`
- [ ] Pattern matching for tag parsing
- [ ] Version construction from VCS state
- [ ] Error handling for VCS operations

**Success Criteria**: Git VCS integration works with `zerv version --source git`

## Phase 2: Pipeline CLI Interface

**Goal**: Implement unified pipeline architecture for version processing

### CLI Framework

- [ ] Argument parsing with `clap`
- [ ] Pipeline command structure (`version`, `check`)
- [ ] Error handling and user messages
- [ ] RON schema parsing for custom schemas

### Core Commands

- [ ] `zerv version` - Main version processing pipeline
- [ ] `zerv check <version>` - Validate version strings
- [ ] `zerv --help` - Help system

### Pipeline Architecture

- [ ] Input sources (git, stdin, string, tag)
- [ ] Schema transformations (`zerv-default` preset + RON-based custom schemas)
- [ ] Version bumping and state overrides
- [ ] Output formatting (structured formats + templates)

**Success Criteria**: Pipeline CLI with Git source functional

## Phase 3: Output Templates

**Goal**: Enable custom output formatting via templates

### Template Engine

- [ ] Template parsing and validation for `--output-template`
- [ ] Variable substitution system
- [ ] Built-in template presets
- [ ] Template validation and error handling

### Output Format Support

- [ ] Structured formats: `pep440`, `semver`, `pvp`
- [ ] Template-based custom output
- [ ] Format validation

**Success Criteria**: `--output-template` works with variable substitution

## Phase 4: Advanced Features

**Goal**: Production-ready feature set

### Enhanced VCS Support

- [ ] Archive support (`.git_archival.json`)
- [ ] Shallow repository handling
- [ ] Multiple VCS systems (Mercurial, SVN)

### Configuration System

- [ ] Config file support (`zerv.toml`)
- [ ] Environment variable support
- [ ] Project-specific settings

### Version Manipulation

- [ ] Version bumping (`--bump major/minor/patch/alpha/beta/rc`)
- [ ] State overrides (`--set-dirty`, `--set-distance N`, `--set-branch`)
- [ ] Custom patterns for tag parsing

**Success Criteria**: Feature parity with dunamai for Git

## Ideal Stable (Alpha) State

### Target Architecture

```
zerv/
├── src/
│   ├── vcs/
│   │   ├── mod.rs           # VCS trait and detection
│   │   ├── git.rs           # Git implementation
│   │   └── archive.rs       # Archive support
│   ├── cli/
│   │   ├── mod.rs           # CLI framework
│   │   ├── commands.rs      # Command implementations
│   │   └── args.rs          # Argument parsing
│   ├── template/
│   │   ├── mod.rs           # Template engine
│   │   └── presets.rs       # Built-in formats
│   ├── config/
│   │   └── mod.rs           # Configuration system
│   └── version/             # ✅ Already implemented
└── tests/
    ├── integration/         # End-to-end tests
    └── fixtures/            # Test repositories
```

### Core Functionality

**Essential Commands**:

```bash
zerv version                 # Auto-detect Git and generate version
zerv version --source git    # Explicit Git source
zerv check 1.2.3            # Validate version string
zerv version --output-format pep440      # Use specific output format
zerv version --output-format semver --output-prefix  # Add "v" prefix
zerv version --output-template "v{{base}}" # Custom output template
```

**Pipeline Examples**:

```bash
# Simple usage
zerv version

# Advanced pipeline with custom schema
zerv version \
  --source git \
  --bump minor \
  --schema-ron '(core: [VarField("major"), VarField("minor"), VarField("patch")], extra_core: [], build: [])' \
  --output-format pep440

# Template output with default schema
zerv version --output-template "{{major}}.{{minor}}.{{patch}}-{{commit}}"

# Default schema (most common)
zerv version --schema zerv-default --output-format pep440
```

**Key Features**:

- ✅ Universal version format (Zerv)
- ✅ PEP440 and SemVer parsing/formatting
- 🎯 Git repository integration
- 🎯 Template-based custom formats
- 🎯 CLI interface with common commands

### Success Metrics for Alpha

1. **Functionality**: Generate versions from Git repositories
2. **Formats**: Support PEP440, SemVer, and custom templates
3. **Usability**: Simple CLI with sensible defaults
4. **Reliability**: Comprehensive test coverage
5. **Performance**: Fast execution (< 100ms for typical operations)

### Post-Alpha Roadmap

**Beta Features**:

- Multiple VCS support (Mercurial, SVN, Darcs)
- Advanced configuration system
- Version bumping and manipulation
- Archive format support

**1.0 Features**:

- Complete dunamai feature parity
- Performance optimizations
- Comprehensive documentation
- Cross-platform distribution

## Development Strategy

### Incremental Approach

1. Build Git VCS integration first (highest impact)
2. Add minimal CLI for testing
3. Expand with templates and configuration
4. Polish for alpha release

### Risk Mitigation

- Focus on Git only initially (80% of use cases)
- Reuse existing Zerv foundation (proven architecture)
- Maintain high test coverage throughout
- Regular integration testing with real repositories

### Timeline Estimate

- **Phase 1** (Git VCS): 2-3 weeks
- **Phase 2** (Basic CLI): 1 week
- **Phase 3** (Templates): 1-2 weeks
- **Phase 4** (Polish): 1 week

**Total to Alpha**: ~6-8 weeks of focused development

The solid foundation of the version parsing system positions the project well for rapid progress toward a functional alpha release.
