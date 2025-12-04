# Documentation Restructure Plan

**Status**: In Progress
**Priority**: High
**Context**: Ready for first publish, need docs restructuring for human vs LLM audiences

## Goals

1. Create comprehensive README.md that works for both humans and LLMs
2. Create docs/llms.md as symlink to README.md for POC simplicity
3. Focus on getting first publish ready with minimal documentation overhead
4. Ensure all existing information from AUTO.md is preserved and organized
5. Future: Separate into human-focused vs comprehensive docs after user feedback

## Target Files & Structure

### README.md (Human-focused, ~2-minute read)

**Audience**: Developers evaluating/adopting Zerv
**Goal**: Scan, copy, run in 2 minutes

#### README.md Table of Contents

1. **Hero Section** ✅ - Above the fold content
    - **Content**: Clear one-liner, key benefit statement
    - **Purpose**: Immediate value proposition

2. **Quick Start** ✅ - Install and run in 60 seconds
    - **Content**:

        ```bash
        # Install
        cargo install zerv

        # Try automated versioning (current branch determines output)
        zerv flow
        # → 1.0.0 (on main branch with tag v1.0.0)
        # → 1.0.1-rc.1.post.3 (on release branch with pre-release tag)
        # → 1.0.1-beta.1.post.3+develop.3.gf297dd0 (on develop branch)
        # → 1.0.1-alpha.59394.post.1+feature.new.auth.1.g4e9af24 (on feature branch)
        # → 1.0.1-alpha.17015.post.1.dev.1764382150+feature.dirty.work.1.g54c499a (on dirty feature branch)
        ```

    - **Purpose**: Get users running code immediately to see value

3. **Key Features** ✅ - 5 concise bullet points (reduced from 6 to avoid redundancy)
    - **Content**:
        - **zerv version**: Flexible, configurable version generation with full control
        - **zerv flow**: Opinionated, automated pre-release management based on Git branches
        - **Smart Schema System**: Auto-detects clean releases, pre-releases, and build context
        - **Multiple Formats**: SemVer, PEP440 (Python), CalVer, custom schemas
        - **CI/CD Integration**: Complements semantic release with branch-based pre-releases and full override control
    - **Purpose**: Clear distinction between flexible (version) vs automated (flow) approaches

4. **Usage Examples** 🚧 - Copy-paste examples by command
    - **Content**:
        - **zerv flow**: Automated branch-based versions with comprehensive documentation:
            - ✅ **Core Principles**: 4 fundamental design principles
            - ✅ **Version Format Explained**: Structure, components, and variations with examples
            - ✅ **Pre-release Resolution Strategy**: Branch patterns, post-release logic, distance modes
            - **Workflow Examples** (3 real-world scenarios with diagrams):
                - ✅ Trunk-Based Development: Parallel features, nested branches, sync scenarios
                - ✅ GitFlow: develop/feature/hotfix/release branches with proper pre-release mapping
                - ✅ Complex Release Management: Branch abandonment, cascading releases
                - **DIAGRAMS**: Copy mermaid diagrams from `.claude/plan/32-zerv-flow-implementation-plan.md`
            - ✅ **Schema Variants**: 10+ standard schema presets only (no CalVer support)
            - ✅ **Branch Rules**: Configurable pattern matching (default GitFlow) for pre-release automation
            - ✅ **Pre-release Control**: Labels (alpha/beta/rc), numbers, hash-based identification
            - ✅ **Post Mode Options**: Tag distance vs commit distance calculation modes
        - **zerv version**: Manual control with 4 main capability areas:
            - ✅ **Schema Variants**: 20+ presets (standard, calver families) and custom RON schemas
            - ✅ **VCS Overrides**: Override tag version, distance, dirty state, branch, commit data
            - ✅ **Version Bumping**: Field-based bumps (major/minor/patch) and schema-based bumps
            - **Component Overrides**: Fine-grained control over individual version components
        - **zerv check**: Version validation for different formats
        - **Input/Output & Piping**: Shared capabilities for both zerv version and zerv flow:
            - Source options: git, stdin
            - Output formats: semver, pep440, zerv
            - Template customization and prefix support
            - Pipeline chaining examples (e.g., `zerv version --output-format zerv | zerv version --source stdin --schema calver`)
    - **Purpose**: Practical examples organized by command for easy reference

5. **Installation** ⏳ - Simple, cargo-focused
    - **Content**: Cargo install command, manual download link
    - **Purpose**: Quick installation path

6. **Links** ⏳ - Point to comprehensive docs
    - **Content**: Link to docs/llms.md, CLI help command
    - **Purpose**: Path to detailed information

### docs/llms.md (Comprehensive reference)

**Audience**: LLMs for code assistance, advanced users
**Goal**: Complete truth source for all Zerv capabilities

#### POC Simplification

**For POC**: Create `docs/llms.md` as symlink to `README.md` - both files will be identical to avoid maintaining separate documentation for now.

**Future**: Can separate into human-focused vs comprehensive docs once we validate the documentation approach and get user feedback.

## Implementation Approach

1. **README.md Creation** (Primary work)
    - Create comprehensive document that works for both humans and LLMs
    - Include all sections from README.md Table of Contents
    - Remove all "under development" warnings (ready for publish)
    - Include practical examples and workflow diagrams
    - Link to CLI help for detailed reference

2. **docs/llms.md Creation**
    - Create as symlink to README.md: `ln -s ../README.md docs/llms.md`
    - No additional content maintenance needed for POC

3. **Diagram Integration**
    - **IMPORTANT**: Copy mermaid diagrams from `.claude/plan/32-zerv-flow-implementation-plan.md`
    - Include the 3 workflow diagrams: Trunk-Based, GitFlow, Complex Release Management
    - These diagrams are essential for showing how zerv flow handles real Git scenarios

4. **Content Migration**
    - Leverage existing AUTO.md content for CLI reference
    - Preserve all existing information
    - Organize for single-document usability

## Success Criteria

- README.md can be read and understood in under 2 minutes
- All CLI options documented in llms.md (complete coverage)
- Clear separation between quick-start vs comprehensive reference
- Examples work with current implementation
- No information loss from existing documentation
- Both files serve their distinct audiences effectively

## Implementation Notes

### 📝 Documentation Maintenance Process Established

**Corresponding Test Guidelines**:
Every code example in documentation must have a corresponding test with reference comment:

```html
<!-- Corresponding test: tests/integration_tests/flow/docs/quick_start.rs:test_quick_start_documentation_examples -->
```

**Working Process**:

1. **Test First**: Write comprehensive test for documentation example
2. **Document Second**: Add example to documentation with exact output
3. **Add Reference**: Include corresponding test comment for maintainability
4. **Validate**: Ensure examples exactly match test outputs
5. **Coordinated Updates**: Update tests first, then documentation to match

### 🚨 Critical Implementation Patterns & Pitfalls

**BRANCH HASH PATTERNS - AVOID REGEX FALLBACKS**:

❌ **WRONG**: Using `{regex:\\d+}` for branch hashes

```rust
// BAD - This is lazy and unpredictable
"1.0.1-alpha.{regex:\\d+}.post.1+feature.test.1.g{hex:7}"
```

✅ **CORRECT**: Use `expect_branch_hash()` for predictable hash generation

```rust
// GOOD - Use actual hash values from expect_branch_hash()
let feature_test_hash = expect_branch_hash("feature/test", 5, "60124");
assert_command(
    "flow --source stdin --distance 42",
    &format!(
        "1.0.1-alpha.{}.post.42+feature.test.42.g{{hex:7}}",
        feature_test_hash
    ),
);
```

**GIT COMMIT HASH PATTERNS**:

- ✅ **For generated commits**: Use `g{{hex:7}}` in format strings
- ✅ **For manual overrides**: Use exact hash (e.g., `.a1b2c3d` without `g` prefix)

**LOOK AT EXISTING EXAMPLES FIRST**:

Before writing new tests, study these reference files:

1. **Branch Hash Examples**: `tests/integration_tests/flow/docs/quick_start.rs` - Shows proper `expect_branch_hash()` usage
2. **Hash Generation**: `src/cli/flow/test_utils.rs` - Understanding hash calculation
3. **Existing Override Tests**: `tests/integration_tests/flow/docs/override_controls.rs` - Working examples of all patterns
4. **Branch Rules**: `tests/integration_tests/flow/docs/branch_rules.rs` - Shows `release/` branch number extraction vs hash fallback

**COMMON PITFALLS TO AVOID**:

1. **Never use `{regex:\\d+}`** - Always use `expect_branch_hash()` for predictable results
2. **Don't guess hash values** - Run the test first to get actual hash, then update expected value
3. **Don't mix hash patterns** - Consistently use either numeric hashes or `{{hex:7}}` format
4. **Don't assume override behavior** - Test actual behavior, don't rely on assumptions
5. **Always check branch patterns** - `release/42` → `rc.42` (number extraction) vs `release/candidate` → `rc.{hash}` (hash fallback)

**WORKFLOW FOR NEW DOCUMENTATION EXAMPLES**:

1. **Study existing patterns** in `tests/integration_tests/flow/docs/`
2. **Run the actual command** to see real output before writing test
3. **Get the actual hash** using `expect_branch_hash("branch-name", 5, "12345")`
4. **Match the test assertion pattern** exactly from existing working examples
5. **Use realistic hex values** in documentation (e.g., `ga1b2c3d`, `g8f7e6d5`)

**Infrastructure in Place**:

- ✅ **TestScenario**: Chainable test framework with CLI command execution
- ✅ **Pattern Assertion System**: `{hex:7}`, `{timestamp:now}`, `{regex:pattern}` support
- ✅ **Documentation Standards**: Comprehensive guidelines in `.claude/ref/documentation-maintenance.md`
- ✅ **Maintenance Comments**: Professional "Corresponding test:" format
- ✅ **Hash Generation**: `expect_branch_hash()` for predictable branch hash generation

**File Structure**:

```
tests/integration_tests/flow/docs/
├── mod.rs
├── test_utils.rs     # TestScenario implementation
├── quick_start.rs   # Quick Start documentation tests (BEST PATTERN REFERENCE)
├── branch_rules.rs   # Branch pattern and hash examples
├── override_controls.rs  # Override examples with proper hash patterns
└── tmp.rs           # assert_commands functionality (temporary)
```

## Next Steps

### Remaining Implementation Tasks:

1. **Usage Examples** - Organize by command with practical examples and workflow diagrams
2. **Installation Section** - Simple cargo-focused installation instructions
3. **Links Section** - Point to comprehensive docs and CLI help
4. **docs/llms.md** - Create symlink to README.md for comprehensive reference

### Diagram Integration:

- **Copy mermaid diagrams** from `.claude/plan/32-zerv-flow-implementation-plan.md`
- **Include 3 workflow diagrams**: Trunk-Based, GitFlow, Complex Release Management
- **Essential for**: Showing real Git scenarios and zerv flow capabilities

## Notes

- This is a consolidation effort - we're reducing doc surface area while improving coverage
- AUTO.md remains as auto-generated CLI reference
- Test-driven documentation ensures examples always stay accurate
- Future: Consider more scalable documentation approach if needed
