# Insights from Archived Documentation

## Key Architectural Decisions (Preserved)

### 1. Universal Version Format (Zerv)

**From**: `zerv-design.md`

- Component-based format system with variable references
- Separation of format template from data values
- Support for timestamp patterns (YYYY, MM, DD, YYYYMMDD)
- Extensible custom variables via HashMap

**Status**: ✅ Fully implemented in `src/version/zerv/`

### 2. Multi-Format Support Strategy

**From**: `key-features.md`, `cli-design.md`

- PEP440 for Python ecosystem (`1.2.3.post7.dev0+g29045e8`)
- SemVer for JavaScript/Node (`1.2.3-post.7+g29045e8`)
- PVP for Haskell (`1.2.3-post-7-g29045e8`)
- Template-based custom formats

**Status**: ✅ PEP440 and SemVer implemented, templates planned

### 3. State-Based Versioning Tiers

**From**: `cli-design.md` - Three-tier system based on repository state

**Status**: 🎯 Architecture ready, needs VCS integration

## Implementation Patterns (Validated)

### 1. Error Handling Strategy

**From**: `planning.md`

- Library layer: Specific error types (`ZervError`)
- CLI layer: `anyhow` for convenience
- No command injection vulnerabilities

**Status**: ✅ Pattern established in current code

### 2. Testing Strategy

**From**: `docker-testing.md`

- Fast local tests without external dependencies
- Docker-based integration tests for real VCS operations

**Status**: ✅ Successfully implemented and working

### 3. Pattern Matching System

**From**: `version-parser-plan.md`

- Auto-detection with fallback patterns
- Custom regex support with named groups
- Built-in patterns for common formats

**Status**: ✅ Implemented in PEP440/SemVer parsers

## Valuable Unfinished Ideas

### 1. Template Variables (Still Relevant)

**From**: `cli-design.md`

**Template Variables for `--output-template`**:

```bash
# Standard variables
{{ major }}, {{ minor }}, {{ patch }}
{{ distance }}, {{ commit }}, {{ branch }}
{{ dirty }}, {{ timestamp }}

# Zerv-specific computed variables
{{ zerv_post }}      # post + distance
{{ zerv_timestamp }} # timestamp if dirty
{{ zerv_branch }}    # escaped branch name
{{ zerv_build }}     # build metadata array
```

**Built-in Output Format Presets**:

```bash
--output-format pep440       # Standard PEP440
--output-format semver       # Standard SemVer
--output-format pvp          # Package Versioning Policy
```

### 2. VCS Integration Patterns

**From**: `dunamai.md`, `planning.md`

**Git Implementation Strategy**:

- Tag discovery with topology ordering
- Distance calculation via `git rev-list --count`
- Dirty detection with optional untracked file handling
- Archive support via `.git_archival.json`
- Branch name extraction and escaping

**Command Execution**:

- Safe command execution with proper error handling
- Environment variable filtering for security
- Cross-platform compatibility

### 3. Advanced Features (Updated for Pipeline)

**From**: `key-features.md`

**Version Bumping (Updated for Pipeline)**:

```bash
zerv version --bump major           # 1.2.3 → 2.0.0
zerv version --bump minor           # 1.2.3 → 1.3.0
zerv version --bump patch           # 1.2.3 → 1.2.4
zerv version --bump alpha           # 1.2.3 → 1.2.4-alpha.1
```

**State Overrides (Updated for Pipeline)**:

```bash
zerv version --set-distance 10      # Override commit distance
zerv version --set-dirty            # Force dirty state
zerv version --set-branch main      # Override branch name
zerv version --set-commit abc123    # Override commit hash
```

## Complementary Tool Strategy

**From**: `key-features.md`

**Zerv + semantic-release Integration**:

- semantic-release: Official release decisions and tagging
- zerv: Continuous version identification for builds
- Perfect complementary workflow for modern CI/CD

## Performance & Security Insights

**From**: `dunamai.md`, `planning.md`

**Performance Targets**:

- Parse 1000+ versions in <100ms
- Minimal VCS command calls
- Compiled regex patterns for speed
- Zero-copy string operations where possible

**Security Considerations**:

- No command injection via safe execution
- Input validation for all patterns
- Minimal attack surface (stateless operation)
- Environment variable sanitization

## Migration Strategy

**From**: `version-parser-plan.md`

**Incremental Development**:

1. Keep existing parsers as fallback during development
2. Feature flags for new implementations
3. Gradual migration with comprehensive testing
4. Remove old code only after full validation

## Configuration System Design

**From**: `cli-design.md`

**Config File Hierarchy**:

1. Project level: `./zerv.toml` (highest priority)
2. User level: `~/.config/zerv.toml`

**Configuration Presets (Distinct Use Cases)**:

```toml
# Custom schemas: Different data structure + standard format output
[schemas]
calver-schema = '(core: [VarTimestamp("YYYY"), VarTimestamp("MM"), VarField("patch")], extra_core: [VarField("pre_release")], build: [])'
git-schema = '(core: [VarField("major"), VarField("minor"), VarField("current_branch"), VarField("distance")], extra_core: [], build: [VarField("current_commit_hash")])'

# Custom templates: Default schema + custom output format
[templates]
my-format = "v{{ major }}.{{ minor }}.{{ patch }}-{{ commit }}"
docker-format = "{{ major }}.{{ minor }}-{{ commit }}"
release-format = "{{ major }}.{{ minor }}.{{ patch }}"
```

**Usage Patterns**:

- **Schema + Standard Format**: `--schema calver-schema --output-format pep440`
- **Default Schema + Template**: `--output-template my-format`
- **Note**: Typically use either schema+format or template alone, but both can be combined for advanced use cases

## Lessons Learned

### 1. Architecture Validation

The universal Zerv format design has proven sound - the implementation validates the original architectural decisions.

### 2. Testing Strategy Success

The Docker-based testing approach solved real development problems and should be maintained.

### 3. Scope Management

The phased approach focusing on Git first (80% of use cases) before expanding to other VCS systems is the right strategy.

### 4. Format Flexibility

The component-based format system provides the flexibility needed for diverse version formats while maintaining type safety.

These archived insights provide a roadmap for implementing the remaining features while validating that the current architecture aligns with the original vision.
