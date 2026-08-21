# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

Zerv is a dynamic versioning CLI tool written in Rust that generates versions for any commit from git and other version control systems. It supports multiple version formats (SemVer, PEP440, CalVer) and is designed for CI/CD builds.

---

## 🎯 Critical Rules (Quick Reference)

**MANDATORY - These rules must ALWAYS be followed:**

1. ✅ **Use constants** instead of bare strings (fields, formats, sources, schemas) → Use `crate::utils::constants::*`
    - **For environment variables**: Use `crate::config::EnvVars::*` (e.g., `EnvVars::PAGER`, `EnvVars::RUST_LOG`)
2. ✅ **Use `ZervError`** for custom errors and `io::Error::other()` for IO errors
3. ✅ **Use `get_git_impl()`** for environment-aware Git operations
4. ✅ **Use `should_run_docker_tests()`** for Docker-dependent tests
5. ✅ **Never reuse Git implementations** across different directories
6. ✅ **Include detailed error context** in all error messages
7. ✅ **NO useless comments** - only comment when code cannot explain itself
8. ✅ **Check existing utilities** in `src/test_utils/` before creating new ones
9. ✅ **Place `use` statements at top of file/module** - never inside functions
10. ✅ **Use `mod` blocks for test organization** - not comment dividers
11. ✅ **Use `bake test` for testing** - never create ad-hoc Git repositories or manual test scripts

**NEVER do these:**

1. ❌ Use bare string literals for field/format/source names
2. ❌ Use `unwrap()` or `expect()` in production code
3. ❌ Add comments that just repeat function names or restate code
4. ❌ Create Git fixtures without proper isolation
5. ❌ Skip Docker test gating for Docker-dependent tests
6. ❌ Write generic error messages without context
7. ❌ Duplicate code instead of using existing utilities
8. ❌ Place `use` statements inside functions (except rare naming conflict cases)
9. ❌ Use comment dividers for test grouping (use `mod` blocks instead)
10. ❌ Create manual test scripts or ad-hoc Git repositories for testing

**📋 Run `/audit` to check and fix violations automatically**

---

## 📚 Planning Workflow

### Check Existing Plans First

**BEFORE performing ANY coding task, check `.claude/plan/` for existing plans and context.**

Plans in `.claude/plan/` are temporary working documents with short lifecycle:

- Created when planning new features/refactors
- Used during implementation
- Deleted or archived when task is complete

### Creating New Plans

**CRITICAL: When user asks you to "write a plan" or "create a plan", you MUST:**

1. ✅ **Check existing `.claude/plan/` files** to find the highest number
2. ✅ **Create new plan** as `.claude/plan/XX-descriptive-name.md` (increment XX)
3. ✅ **Do NOT start coding** until the plan is reviewed and approved
4. ✅ **Use ExitPlanMode tool** to present the plan for approval

**Plan Document Structure:**

- **Status**: Planned/In Progress/Completed
- **Priority**: High/Medium/Low
- **Context**: Why this work is needed
- **Goals**: What we want to achieve
- **Implementation Plan**: Detailed steps
- **Testing Strategy**: How to validate
- **Success Criteria**: Definition of done
- **Documentation Updates**: What docs need updating

**Note**: Plans can reference `.claude/ref/` docs, but `.claude/ref/` should NEVER reference specific plans (one-way dependency).

---

## 📖 Essential Commands & Workflows

@.claude/ref/workflows/commands.md
@.claude/ref/workflows/coverage.md

---

## 🏗️ Architecture

### Pipeline Overview

```
Input → VCS Detection → Version Parsing → Transformation → Format Output
```

**For detailed architecture documentation:**

@.claude/ref/architecture/pipeline.md
@.claude/ref/architecture/modules.md
@.claude/ref/architecture/cli.md

---

## 🚨 Code Standards

### Quick Reference

- **Comments**: Only when code can't explain itself
- **Imports**: Always at top of file/module
- **Test organization**: Use `mod` blocks, not comment dividers
- **Line length**: Max 100 chars (rustfmt), use `format!()` for long strings

**For detailed standards:**

@.claude/ref/standards/code-style.md
@.claude/ref/standards/error-handling.md
@.claude/ref/standards/logging.md
@.claude/ref/standards/constants.md

---

## 📝 Documentation Standards

### Test-Driven Documentation

**MANDATORY - All code examples in documentation must have corresponding tests**

1. ✅ **Every code example** in README.md and other docs must be backed by tests
2. ✅ **Add reference comments**: `<!-- Corresponding test: path/to/test.rs:test_function -->`
3. ✅ **Exact matching**: Documentation examples must exactly match test output
4. ✅ **Realistic values**: Use actual commit hashes, not `{hex}` placeholders
5. ✅ **Coordinated updates**: Update tests first, then documentation to match

### Reference Comment Format

```html
<!-- Corresponding test: tests/integration_tests/flow/docs/quick_start.rs:test_quick_start_documentation_examples -->
```

**For detailed documentation maintenance guidelines:**

@.claude/ref/documentation-maintenance.md

**📋 Run `/docs-sync` periodically** (after test renames/refactors, before releases, before large docs edits) to verify docs examples still match their backing tests.

---

## 🧪 Testing Standards

### Quick Reference

- **Environment variables**: `ZERV_TEST_NATIVE_GIT`, `ZERV_TEST_DOCKER`
- **Git operations**: Always use `get_git_impl()`
- **Docker tests**: Always use `should_run_docker_tests()` gating
- **CRITICAL**: For Git-related tests, **ALWAYS use `ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true`**
- **Integration tests**: Prefer `TestCommand::run_with_stdin()` (90% of cases)
- **Docs-site examples**: Tests for `.mdx` page examples live in `tests/integration_tests/docs/`; README-ported examples keep their existing `flow/docs/`, `version/docs/` tests

### Test Cadence (avoid slow full runs)

- **Small/iterative change** → run **only the related test**, not the whole suite. Target the specific test or module:
    - Unit: `cargo +nightly test --lib <module_or_name>`
    - Integration: `cargo +nightly test --test integration -- <filter>` (e.g. `-- config_file`)
    - Add `ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true` only when the test touches Git data.
- **Full `bake -c lint test`** → run **once per milestone** (phase done, feature complete) or as a **single end-of-task check** before handoff/commit, not after every edit. It is slow; do not re-run it for trivial changes.
- **Never re-run the full suite to recover truncated output.** Capture it once to a log + exit code in the same invocation: `bake -c lint test > /tmp/zerv_bake.log 2>&1; echo "EXIT=$?"`, then `grep` the **same** log or read `$?` for the result. Piping through `tail`/`head` drops the summary lines and tempts a wasteful second run — if output is truncated, grep the log you already have. Do NOT invoke `bake` a second time for the same check.
- Lint steps (`cargo +nightly fmt --check`, `cargo +nightly clippy -- -D warnings`) only need re-checking when production code changes, not test-only edits.

**For detailed testing patterns:**

@.claude/ref/testing/overview.md
@.claude/ref/testing/unit-tests.md
@.claude/ref/testing/integration-tests.md

---

## 🚀 CI/CD

@.claude/ref/workflows/cicd.md

---

## 🛠️ Using Claude Code Features

### When to Use What

**Slash Commands** (Simple, predefined workflows):

- `/audit` - Run code quality audit and fix violations
- Use for: Detecting and fixing code quality violations efficiently
- `/docs-sync` - Audit docs examples against their backing tests
- Use for: Periodic docs↔test consistency sweep

**Agents** (Complex, multi-step exploration):

- Codebase exploration: "How does version bumping work across the codebase?"
- Large refactoring: "Migrate all tests to use environment-aware pattern"
- Research tasks: "Find all API endpoints and document them"
- Use for: Open-ended tasks requiring autonomous decision-making

**Direct Questions** (Fastest for simple tasks):

- "Review this function for bugs"
- "Explain how this works"
- "Fix the error in this code"
- Use for: Quick reviews, explanations, single-file changes

**Rule of thumb**:

1. Try direct question first (fastest)
2. Use slash command for standardized workflows
3. Use agent only for complex multi-file exploration/refactoring
