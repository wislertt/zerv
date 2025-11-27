# Documentation Restructure Plan

**Status**: Planned
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

1. **Hero Section** - Above the fold content
    - **Content**: Clear one-liner, key benefit statement
    - **Purpose**: Immediate value proposition

2. **Quick Start** - Install and run in 60 seconds
    - **Content**:

        ```bash
        # Install
        cargo install zerv

        # Try automated version (current branch determines output)
        zerv flow
        # → 1.2.3-alpha.5 (on feature branch)
        # → 1.3.0-beta.1 (on develop branch)
        # → 1.2.3 (on main branch)
        ```

    - **Purpose**: Get users running code immediately to see value

3. **Key Features** - 6 bullet points maximum
    - **Content**:
        - **zerv version**: Flexible, configurable version generation with full control
        - **zerv flow**: Opinionated, automated pre-release management based on Git branches
        - Smart Schema System: Auto-detects clean releases, pre-releases, and build context
        - Multiple Formats: SemVer, PEP440 (Python), CalVer, custom schemas
        - CI/CD Ready: Override capabilities and pipeline-friendly output formats
        - Zero Config: Works out of the box with sensible defaults
    - **Purpose**: Clear distinction between flexible (version) vs automated (flow) approaches

4. **Usage Examples** - Copy-paste examples by command
    - **Content**:
        - **zerv flow**: Automated branch-based versions with 5 main capability areas:
            - **Workflow Examples** (3 real-world scenarios with diagrams):
                - Trunk-Based Development: Parallel features, nested branches, sync scenarios
                - GitFlow: develop/feature/hotfix/release branches with proper pre-release mapping
                - Complex Release Management: Branch abandonment, cascading releases
                - **DIAGRAMS**: Copy mermaid diagrams from `.claude/plan/32-zerv-flow-implementation-plan.md`
            - **Schema Variants**: 10+ standard schema presets only (no CalVer support)
            - **Branch Rules**: Configurable pattern matching (default GitFlow) for pre-release automation
            - **Pre-release Control**: Labels (alpha/beta/rc), numbers, hash-based identification
            - **Post Mode Options**: Tag distance vs commit distance calculation modes
        - **zerv version**: Manual control with 4 main capability areas:
            - **Schema Variants**: 20+ presets (standard, calver families) and custom RON schemas
            - **VCS Overrides**: Override tag version, distance, dirty state, branch, commit data
            - **Version Bumping**: Field-based bumps (major/minor/patch) and schema-based bumps
            - **Component Overrides**: Fine-grained control over individual version components
        - **zerv check**: Version validation for different formats
        - **Input/Output & Piping**: Shared capabilities for both zerv version and zerv flow:
            - Source options: git, stdin
            - Output formats: semver, pep440, zerv
            - Template customization and prefix support
            - Pipeline chaining examples (e.g., `zerv version --output-format zerv | zerv version --source stdin --schema calver`)
    - **Purpose**: Practical examples organized by command for easy reference

5. **Installation** - Simple, cargo-focused
    - **Content**: Cargo install command, manual download link
    - **Purpose**: Quick installation path

6. **Links** - Point to comprehensive docs
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

## Notes

- This is a consolidation effort - we're reducing doc surface area while improving coverage
- AUTO.md remains as auto-generated CLI reference
- Future: Consider more scalable documentation approach if needed
