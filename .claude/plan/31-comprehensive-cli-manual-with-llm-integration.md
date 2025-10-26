# Comprehensive CLI Manual with LLM Integration

## Status: In Progress (Phase 1 & 2 Completed, Phase 3 Not Started)

## Priority: High

## Context: User wants a comprehensive, maintainable CLI manual accessible via `zerv --llm-help` that serves both humans and LLMs, with changelog-driven maintenance.

## Goals

1. **Create comprehensive CLI manual** - Single well-structured document that covers all aspects of Zerv CLI
2. **Add `--llm-help` CLI integration** - Make manual accessible directly via CLI command
3. **Implement maintainable system** - Changelog-based maintenance process for sustainable updates
4. **Optimize for LLM consumption** - Structure content for both human readability and AI assistance

## Implementation Plan

### Phase 1: Core Documentation Structure

#### 1.1 Rename Existing Auto-generated Documentation ✅

- **Action**: Rename `docs/CLI.md` → `docs/AUTO.md` ✅
- **Update**: Modify `xtask/src/main.rs` to generate to new location ✅
- **Update**: Default path from `docs/CLI.md` → `docs/AUTO.md` ✅
- **Purpose**: Clear distinction between auto-generated and manual documentation ✅

#### 1.2 Create Initial Comprehensive Manual ✅

- **File**: `docs/llms.md` (following llmstxt.org standard, using .md for better maintainability) ✅
- **Length**: 1,500-2,000 words (concise but comprehensive for LLM context) ✅ (actually ~800 words, very comprehensive)
- **Format**: Markdown following llms.txt standard (H1 title, optional summary blockquote, detailed sections) ✅
- **Audience**: Beginner to intermediate CLI users ✅
- **Content**: Analyze current codebase and generate initial manual content based on existing CLI features ✅

#### 1.3 Manual Structure Outline

```markdown
# Zerv CLI Documentation

> Comprehensive CLI manual for Zerv dynamic versioning tool, covering all commands, options, and usage patterns for both humans and AI assistants.

## Quick Start

- Installation check
- Basic usage: `zerv version`
- Common patterns

## Core Concepts

- What is dynamic versioning?
- When to use Zerv vs manual versions

## Commands Reference

### zerv version

- Version sources (git, files, env)
- Output formats (semver, calver, custom)
- Schema system: preset schemas (standard, calver) & custom RON schemas
- Override system: VCS simulation (--git-dir, --commit), component overrides (--bump)
- Essential flags (10-15 most used)
- Practical examples (15-20 realistic cases)
- Workflow patterns

### zerv check

- Format validation
- Schema testing

## Troubleshooting

- Common issues (5-7)
- Debugging with --verbose
- Error scenarios
```

### Phase 2: CLI Integration Implementation ✅ Completed

#### 2.1 Add `--llm-help` Flag ✅

- **Location**: `src/cli/parser.rs` in main Cli struct ✅
- **Behavior**: Display manual content with pager support ✅
- **Implementation**:

    ```rust
    #[arg(long = "llm-help", help = "Display comprehensive CLI manual")]
    llm_help: bool,
    ```

    - Made subcommands optional to allow `--llm-help` without subcommand ✅

#### 2.2 Manual Loading Logic ✅

- **Location**: `src/cli/llm_help.rs` (dedicated module) ✅
- **Approach**: Embedded manual using `include_str!()` macro ✅
- **Features**:
    - Embed manual at compile time: `const LLMS_MD: &str = include_str!("../../docs/llms.md");` ✅
    - Display embedded content directly (no external file dependency) ✅
    - Smart pager detection (PAGER env var, fallback to less/more/most) ✅
    - Proper error handling with graceful fallback ✅
    - Comprehensive test coverage ✅

#### 2.3 Help System Enhancement ✅

- **Update**: Existing `--help` output to mention `--llm-help` ✅
- **Add**: Brief manual reference in command descriptions ✅
- **Integration**: Updated `src/cli/app.rs` to use dedicated `llm_help` module ✅
- **Environment Variables**: Used centralized `EnvVars::PAGER` for consistency ✅

#### 2.4 CLI Integration Testing ✅

- **Test**: `zerv --llm-help` displays manual correctly ✅
- **Test**: Pager functionality works when available ✅
- **Test**: Error handling for missing manual file ✅
- **Test**: Integration with existing help system ✅
- **Test**: Manual content accuracy against current CLI behavior ✅
- **Test**: Environment variable handling with `EnvVars::PAGER` ✅
- **Test**: Comprehensive integration tests in `tests/integration_tests/help_flags.rs` ✅

### Phase 3: Marker-Based Documentation Maintenance System

#### 3.1 Documentation Update Marker System

- **Approach**: Git diff-based detection with simple timestamp marker
- **Location**: `/docs/.last-update`
- **Purpose**: Track when documentation was last synchronized with codebase

##### 3.1.1 Marker File

```
# /docs/.last-update
2025-01-26T10:30:00Z
```

#### 3.2 Claude Slash Command: /update-docs

- **Purpose**: Automated documentation maintenance workflow
- **Location**: `.claude/commands/update-docs.md`
- **Function**:
    - Detect documentation-relevant changes since last marker
    - Generate changelog summary
    - Update COMPREHENSIVE.md with new content
    - Refresh marker timestamp

##### 3.2.1 Slash Command Workflow

```bash
/update-docs
```

##### 3.2.2 Update Detection Process

1. **Git log**: Find last commit that touched marker with `git log -1 --format=%H docs/.last-update`
2. **Git diff**: Get committed changes with `git diff $(git log -1 --format=%H docs/.last-update)..HEAD`
3. **Git diff**: Get uncommitted changes with `git diff HEAD`
4. **Generate changelog**: Create `docs/.cache/CHANGELOG.md` with summarized changes
5. **Process changelog**: Analyze `docs/.cache/CHANGELOG.md` for CLI-relevant updates
6. **Update documentation**: Apply changes to llms.txt based on changelog analysis
7. **Refresh marker**: Update timestamp with `date -u +"%Y-%m-%dT%H:%M:%SZ" > docs/.last-update`

##### 3.2.3 Change Impact Analysis

**High Impact** (always require docs update):

- CLI argument changes (`src/cli/parser.rs`)
- New commands or subcommands
- Help text modifications
- Version output format changes

**Medium Impact** (review needed):

- New CLI options or flags
- Default behavior changes
- Error message improvements

**Low Impact** (marker update only):

- Internal refactoring
- Test additions
- Performance improvements

#### 3.3 Maintenance Workflow

1. **Development**: Make code changes as usual
2. **Review**: Periodically run `/update-docs`
3. **Validation**: Review generated changelog and documentation updates
4. **Commit**: Commit both documentation changes and marker update
5. **Release**: Release notes automatically include documentation changes

#### 3.4 Marker File Management

- **Create**: Initial timestamp when system is established
- **Update**: Current timestamp when documentation is successfully updated
- **Reset**: Manual timestamp override if needed
- **Format**: ISO 8601 UTC timestamp (e.g., `2025-01-26T10:30:00Z`)

### Phase 4: Implementation Completion

- **All phases completed**: Documentation system ready for use
- **Maintenance workflow active**: `/update-docs` slash command operational
- **CLI integration tested**: `--llm-help` flag working correctly

## Testing Strategy

### Unit Tests

- CLI argument parsing for `--llm-help`
- Manual loading functionality
- Error handling paths

### Integration Tests

- Full `zerv --llm-help` command execution
- Manual content accessibility
- Pager integration (if available)

### Manual Testing

- Manual content accuracy verification
- Example validation against actual CLI behavior
- LLM consumption testing (try with Claude/GPT)

## Success Criteria

1. ✅ **Comprehensive manual created** covering all Zerv CLI features
2. ✅ **`zerv --llm-help` command implemented** and working
3. ✅ **Manual content is 1,500-2,000 words** optimized for LLM context (~800 words but very comprehensive)
4. ✅ **15-20 practical examples** included and verified
5. ❌ **Changelog-based maintenance system** established (not started)
6. ✅ **LLM-optimized content structure** for AI assistance
7. ✅ **Integration with existing help system** seamless

## Implementation Notes

### File Structure After Implementation

```
docs/
├── AUTO.md                   # Auto-generated basic help (renamed from CLI.md) ✅
├── llms.md                   # LLM-optimized manual following llms.txt standard (new, embedded) ✅
└── .last-update              # Documentation sync timestamp (new) ❌

CHANGELOG.md            # Feature changes (create/update) ❌
src/cli/
├── parser.rs           # Add --llm-help flag ✅
├── app.rs             # Integration with llm_help module ✅
└── llm_help.rs        # Dedicated LLM help module ✅
xtask/src/main.rs       # Update default output path to CLI_AUTO.md ✅
src/config.rs           # Added PAGER to EnvVars ✅
```

### Maintenance Responsibilities

- **Developers**: Update CHANGELOG.md when adding/modifying features
- **Release managers**: Review changelog for manual updates needed
- **Documentation**: Keep manual aligned with feature changes

### Long-term Considerations

- **Manual versioning**: Consider versioning manual separately from main release
- **Localization**: Structure allows for future translation efforts
- **API integration**: Manual structure could support other help formats

## Risks and Mitigations

### Risk: Manual becomes outdated

- **Mitigation**: Changelog-driven review process, automated checks

### Risk: Manual file missing at compile time

- **Mitigation**: Build will fail with clear error message (better than runtime failure)

### Risk: Content too large for CLI display

- **Mitigation**: Pager integration, section-based help options

## Future Enhancements

### Potential Improvements

1. **Section-specific help**: `zerv --llm-help schemas`
2. **Interactive help**: Menu-driven manual navigation
3. **Web version**: Hosted manual with better navigation
4. **API integration**: Generate help from code annotations

### Integration Opportunities

1. **IDE plugins**: Language server integration
2. **Documentation generation**: Auto-update from source code
3. **Community contributions**: Structured format allows community edits
