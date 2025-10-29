# Zerv Flow Implementation Plan

**Status**: In Progress
**Priority**: High
**Context**: Step-by-step implementation plan for the `zerv flow` command based on CLI design in document #33.

## Current Progress Summary (as of Oct 29, 2025)

### ✅ **Completed Work:**

- **Phase 1**: Core CLI Structure - 100% Complete
    - CLI module structure implemented (`src/cli/flow/`)
    - Flow command registered in parser and dispatcher
    - Help system working (`zerv flow --help`)
    - Shared args integration completed
    - Validation system implemented
    - All tests passing (6/6 flow tests)

- **Shared Args Refactoring**: Completed
    - Created `src/cli/common/args/` with InputConfig, OutputConfig, Validation
    - Both version and flow commands now use shared args
    - Eliminated code duplication between commands
    - Comprehensive test coverage (35 tests) for shared components

- **Code Quality Improvements**: Completed
    - Added BoolResolution utility for opposing boolean flags
    - Cleaned up verbose documentation comments
    - Fixed inconsistent module structures
    - Added comprehensive test coverage to common args

### 🔄 **Next Steps:**

- **Phase 2**: Implement branch pattern system and flow-to-version translation logic
- **Phase 3**: Complete pipeline assembly with actual version command integration
- **Phase 4**: Comprehensive testing and documentation

## Goals

1. Implement `zerv flow` command that mirrors `zerv version` structure
2. Add intelligent pre-release management based on Git branch patterns
3. Provide flexible output options (full, pre-release only, base-only)
4. Support configurable post distance calculation (tag vs commit mode)
5. Enable branch pattern detection with RON configuration strings

## Implementation Plan

### ✅ **Phase 1: Core CLI Structure - COMPLETED**

**All Phase 1 tasks have been successfully completed:**

- ✅ CLI structure implemented and working
- ✅ Flow command registered and dispatching correctly
- ✅ Help system functioning (`zerv flow --help` works)
- ✅ Basic validation and error handling in place
- ✅ Shared input/output args integration complete
- ✅ Module structure properly organized (args/mod.rs pattern)

### Phase 2: Flow Logic as Translation Layer

#### Step 1: Create Flow Command Module Structure ✅

- **Files**:
    - `src/cli/flow/mod.rs` (new file - module exports)
    - `src/cli/flow/args/mod.rs` (new file - argument structs)
    - `src/cli/flow/pipeline.rs` (new file - main handler)
- **Tasks**:
    - ✅ Create module structure following existing version command pattern
    - ✅ Define `FlowArgs` struct with `clap::Parser` derive macro
    - ✅ Organize arguments into logical groups (input, output, flow-specific, overrides)
    - ✅ Set up proper module exports and dependencies
    - ✅ Use shared input/output args from `src/cli/common/args/`
    - ✅ Implement consistent module structure with version command
- **Validation**: ✅ Module compiles and imports work correctly

#### Step 2: Implement Flow Command Handler ✅

- **File**: `src/cli/flow/pipeline.rs`
- **Tasks**:
    - ✅ Create `run_flow_pipeline(args: FlowArgs) -> Result<String, ZervError>` function
    - ✅ Follow existing pattern: validation → processing → formatting → return
    - ✅ Set up basic argument validation using `args.validate()` method
    - ✅ Add error handling with `ZervError` and detailed context
    - ✅ Use constants from `crate::utils::constants::*` instead of bare strings
    - ✅ Add placeholder implementation that returns `NotImplemented` error
- **Validation**: ✅ Function compiles and returns proper Result type

#### Step 3: Register Flow Command in CLI Parser ✅

- **Files**: `src/cli/parser.rs`, `src/cli/app.rs`
- **Tasks**:
    - ✅ Add `Flow(FlowArgs)` to `Commands` enum in `parser.rs`
    - ✅ Add command dispatch handling in `app.rs` following existing pattern
    - ✅ Ensure output is written to provided `writer`
    - ✅ Add proper error propagation in dispatch
- **Validation**: ✅ `zerv flow --help` shows all arguments correctly

### Phase 2: Flow Logic as Translation Layer

#### Step 4: Create Branch Pattern System

- **File**: `src/cli/flow/branch_rules.rs` (new file)
- **Tasks**:
    - Define `BranchRule` struct with branch_pattern, pre_release, number, post_mode fields
    - Define `BranchRules` collection struct with RON parsing
    - Implement exact pattern matching (`develop`) and wildcard matching (`release/*`)
    - Add number extraction from branch names and hash-based numbering
    - Implement default branch rules (develop, release/\*)
    - Keep it simple - just pattern matching and rule lookup
- **Validation**: Pattern matching works correctly for branch detection

#### Step 5: Create Flow-to-Version Translation Logic

- **File**: `src/cli/flow/translator.rs` (new file)
- **Tasks**:
    - Create `translate_flow_to_version_args()` function
    - Detect current branch using existing Git operations
    - Apply branch pattern matching to determine pre-release settings
    - Map flow arguments to equivalent `zerv version` arguments
    - Handle manual overrides vs automatic detection
    - Return `VersionArgs` struct for passing to version command
    - Keep logic minimal - just translation, no version calculation
- **Validation**: Translation produces correct `VersionArgs` for all scenarios

#### Step 6: Create Version Command Wrapper

- **File**: `src/cli/flow/wrapper.rs` (new file)
- **Tasks**:
    - Create wrapper to call existing `run_version_pipeline()` function
    - Support multiple version command calls if needed (like piping)
    - Handle conversion between flow output modes and version command options
    - Parse Zerv objects returned from version command for further processing
    - Ensure proper error handling from version command calls
    - Reuse all existing version logic - no duplication
- **Validation**: Wrapper successfully calls version command and returns results

### Phase 3: Pipeline Assembly

#### Step 7: Assemble Flow Pipeline

- **File**: `src/cli/flow/pipeline.rs` (continued)
- **Tasks**:
    - Create `run_flow_pipeline(args: FlowArgs) -> Result<String, ZervError>` function
    - Simple pipeline: validate args → translate to version args → call version command → format output
    - Use existing `run_version_pipeline()` for all heavy lifting
    - Handle output mode translation (`--with-pre-release`, `--base-only`)
    - Keep flow logic minimal - most work delegated to version command
    - Add verbose output showing translation results for debugging
- **Validation**: Complete flow works end-to-end using version command

#### Step 8: Add Argument Validation ✅

- **File**: `src/cli/flow/args/validation.rs`
- **Tasks**:
    - ✅ Implement `validate()` method for `FlowArgs` struct
    - ✅ Add validation for conflicting argument combinations
    - ✅ Validate RON string format for branch rules
    - ✅ Validate post mode values and hash length ranges
    - ✅ Use `ZervError` with detailed messages for validation failures
    - ✅ Follow existing validation patterns from version command
    - ✅ Use shared validation for input/output conflicts
- **Validation**: ✅ All validation scenarios work correctly

### Phase 4: Testing and Documentation

#### Step 9: Add Comprehensive Test Suite

- **Files**:
    - `src/cli/flow/` - Add `#[cfg(test)] mod tests` blocks to each module
    - `tests/cli_flow_test.rs` (new file - integration tests)
- **Tasks**:
    - Unit tests for each component (pattern matching, pre-release resolution, etc.)
    - Integration tests for complete workflows using `rstest`
    - CLI argument parsing tests using `clap::Parser::try_parse_from()`
    - Cross-platform testing with different Git configurations
    - Performance tests for large repositories
    - Test error scenarios and edge cases
- **Validation**: All tests pass and coverage is adequate

#### Step 10: Update Documentation and Help Text

- **Files**:
    - `src/cli/flow/args.rs` - Update doc comments for help text
    - Documentation files (user manual, architecture docs)
- **Tasks**:
    - Add comprehensive doc comments to all argument structs
    - Update CLI help text to be clear and descriptive
    - Add usage examples to help text
    - Update user documentation with flow command section
    - Add practical examples for different workflows (GitFlow, trunk-based, etc.)
- **Validation**: Help text is clear and documentation is complete

## Key Implementation Patterns

### Translation Layer Pattern

- Flow acts as intelligent translation layer to existing `zerv version` command
- Minimal custom logic - most functionality delegated to version command
- Map flow-specific arguments to equivalent version command arguments
- Support multiple version command calls for complex scenarios

### Command Structure Pattern

- Follow existing `version` command modular structure
- Use `clap::Parser` for argument definitions
- Return `Result<String, ZervError>` from pipeline functions
- Register in `Commands` enum and dispatch in `app.rs`

### Reuse Pattern

- Use existing `run_version_pipeline()` for all heavy lifting
- Reuse existing Git operations, version parsing, and formatting
- No duplication of version calculation logic
- Leverage existing error handling and validation patterns

### Testing Pattern

- Include `#[cfg(test)] mod tests` blocks in each file
- Use `rstest` for parameterized tests
- Test CLI parsing with `clap::Parser::try_parse_from()`
- Test translation logic by verifying generated `VersionArgs`
- Follow existing test organization patterns

### Constants Usage Pattern

- Import from `crate::utils::constants::*`
- Use constants instead of bare strings for all field/format/source names
- Follow existing constant usage patterns

## Testing Strategy

### Unit Tests

- Pattern matching logic (exact, wildcard, number extraction)
- Pre-release resolution for all branch types
- Post distance calculation (tag vs commit modes)
- Output formatting for all modes and formats

### Integration Tests

- Complete workflow testing with sample repositories
- CLI argument validation and error handling
- Git environment testing (native git, docker, etc.)
- Cross-platform compatibility

### Test Scenarios

- Trunk-based development workflows
- GitFlow branching strategies
- Feature branch workflows
- Hotfix scenarios
- Release branch management

## Success Criteria

1. **Functional Requirements**:
    - [ ] All CLI arguments work as specified in design #33
    - [ ] Branch pattern detection works for default and custom rules
    - [ ] Pre-release resolution produces correct versions
    - [ ] Post distance calculation works for both modes
    - [ ] Output formatting works for all modes and formats

2. **Non-Functional Requirements**:
    - [ ] Performance comparable to `zerv version`
    - [ ] Consistent error handling and messaging
    - [ ] Proper integration with existing systems
    - [ ] Comprehensive test coverage
    - [ ] Documentation and help text complete

3. **Integration Requirements**:
    - [ ] Works with existing Git environment detection
    - [ ] Compatible with existing configuration system
    - [ ] Maintains consistency with `zerv version` output
    - [ ] Supports all existing output formats

## Documentation Updates

1. **CLI Help Text**: Update help text for all new arguments
2. **User Documentation**: Add `zerv flow` section to user manual
3. **Architecture Documentation**: Update with flow command components
4. **Examples**: Add practical usage examples for different workflows

## Dependencies

### Internal Dependencies

- Existing Git operations (`get_git_impl()`)
- Version parsing and formatting system
- CLI argument parsing infrastructure
- Configuration and error handling systems

### External Dependencies

- RON (for branch rule configuration)
- Existing Rust dependencies (no new ones expected)

## Risk Assessment

### Technical Risks

- **Complex pattern matching**: Mitigated by comprehensive testing
- **Git integration complexity**: Mitigated by reusing existing Git operations
- **Performance impact**: Mitigated by efficient algorithms and caching

### Schedule Risks

- **Component integration complexity**: Mitigated by phased approach
- **Testing coverage**: Mitigated by parallel test development
- **Documentation updates**: Mitigated by starting documentation early

## Next Steps

1. Begin Phase 1 implementation (CLI structure)
2. Set up development branch for `zerv flow` work
3. Create test repositories for different scenarios
4. Review and validate implementation plan with team

---

**This implementation plan provides a structured approach to implementing `zerv flow` while maintaining code quality and ensuring comprehensive testing.**
