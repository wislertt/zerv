# Implementation Plan: Render and Source Improvements

**Status:** Phase 1 Complete, Phase 2 Complete, Phase 3 Complete
**Priority:** Medium
**Created:** 2025-02-11

## Context

From `TODO.md`, there are two related features to implement:

1. **Better `--source` handling** - Smart defaults and `none` source option
2. **New `render` subcommand** - Parse version string and render with full output flexibility

## Goals

1. Smart source detection: default to `stdin` if piped, otherwise `git`
2. Add `--source none` for VCS-independent operation (everything from args/overrides)
3. New `render` subcommand that accepts version string + uses same output options as `version`/`flow`
4. Support format conversion, normalization, templates, schemas, prefixes

## Current State

**Source Constants** (`src/utils/constants.rs`):

- `GIT`, `STDIN` - only two options
- Default is always `GIT` in `InputConfig`

**InputConfig** (`src/cli/common/args/input.rs`):

```rust
#[arg(short = 's', long = "source", default_value = sources::GIT, ...)]
pub source: String,
```

**Good News:** `OutputFormatter::format_output()` already exists and handles all rendering!

- Located in `src/cli/utils/output_formatter.rs`
- Takes `&Zerv`, output format, prefix, template
- Fully decoupled from VCS processing

---

## Progress Tracking

### Phase 1: Better `--source` Handling

- [x] 1.1 Add `NONE` source constant
- [x] 1.2 Change `InputConfig.source` to `Option<String>`
- [x] 1.3 Implement smart source default in `validate()`
- [x] 1.4 Create `none_pipeline.rs` for version
- [x] 1.5 Wire up `none` source in version pipeline
- [x] 1.6 Add tests for smart source default
- [x] 1.7 Add tests for `none` source

**Note:** Flow uses `run_version_pipeline()` internally, so `none` source handling automatically applies to flow. No separate flow implementation needed.

### Phase 2: New `render` Subcommand

**Key insight:** Render reuses existing version parsing infrastructure!

**From git source pattern (`src/pipeline/vcs_data_to_zerv_vars.rs`):**

```rust
let version = VersionObject::parse_with_format(tag_version, input_format)?;
let vars: ZervVars = version.into();  // impl From<VersionObject> for ZervVars
let zerv = Zerv::from_vars(vars)?;
```

**Render follows the exact same pattern:**

```
version_string → VersionObject::parse_with_format() → VersionObject
VersionObject → impl From<VersionObject> for ZervVars → ZervVars
ZervVars → Zerv::from_vars() → Zerv
Zerv → OutputFormatter::format_output() → final output
```

No custom parsing functions needed - everything already exists in `VersionObject`!

- [x] 2.1 Create `src/cli/render/mod.rs` with `RenderArgs`
- [x] 2.2 Create `src/cli/render/pipeline.rs` using existing `VersionObject` + `OutputFormatter`
- [x] 2.3 Add `Render` command to `Commands` enum in `parser.rs`
- [x] 2.4 Wire up render handler in `app.rs`

### Phase 3: Testing

- [x] 3.1 Source tests for version (smart_default.rs, none.rs)
- [x] 3.2 Source tests for flow (sources.rs)
- [x] 3.3 Render unit tests - 83 tests covering:
    - Basic format conversion (SemVer ↔ PEP440)
    - Complex versions (pre-release, post, dev, epoch, build)
    - Template rendering (all available variables)
    - Validation (template + prefix conflict)
    - Invalid input handling
- [x] 3.4 Render integration tests - 67 tests covering:
    - Format conversion (semver → pep440, pep440 → semver)
    - Auto format detection
    - Prefix support
    - Templates (basic, prerelease, object parts, PEP440 extended)
    - Validation errors (template + prefix conflict, template + non-semver format)

### Phase 4: Documentation

- [x] 4.1 Update CLI help text in `parser.rs`
    - Updated render command description (removed "schemas", added "prefixes")
    - Added render examples to main CLI help
    - Added flow examples at top (most important subcommand)
- [ ] 4.2 Add render examples to README.md

---

## Implementation Plan

### Phase 1: Better `--source` Handling

#### 1.1 Add `NONE` Source Constant

**File:** `src/utils/constants.rs`

```rust
pub mod sources {
    pub const GIT: &str = "git";
    pub const STDIN: &str = "stdin";
    pub const NONE: &str = "none";  // NEW
}
```

#### 1.2 Change InputConfig Source to Option

**File:** `src/cli/common/args/input.rs`

```rust
#[arg(short = 's', long = "source",
      value_parser = [sources::GIT, sources::STDIN, sources::NONE],
      help = "Input source: 'git', 'stdin', or 'none'")]
pub source: Option<String>,  // Changed from String, removed default_value
```

Update tests accordingly.

#### 1.3 Implement Smart Source Default

**File:** `src/cli/app.rs`

After extracting `stdin_content`, apply smart default:

```rust
match cli.command {
    Some(Commands::Version(mut args)) => {
        // Apply smart default
        if args.input.source.is_none() {
            args.input.source = if stdin_content.is_some() {
                Some(sources::STDIN.to_string())
            } else {
                Some(sources::GIT.to_string())
            };
        }
        let output = run_version_pipeline(*args, stdin_content.as_deref())?;
        // ...
    }
    // Same for Flow
}
```

#### 1.4-1.5 Handle `none` Source in Version Pipeline

**New file:** `src/cli/version/none_pipeline.rs`

```rust
use crate::cli::version::args::VersionArgs;
use crate::version::zerv::draft::ZervDraft;
use crate::error::ZervError;

pub fn process_none_source(args: &VersionArgs) -> Result<ZervDraft, ZervError> {
    // Create ZervDraft from overrides only, no VCS data
    let mut draft = ZervDraft::new();

    // Apply all override values if provided
    if let Some(ref overrides) = args.main.overrides {
        // Set version components from overrides
        // ...
    }

    Ok(draft)
}
```

**Modify:** `src/cli/version/pipeline.rs`

Add new branch:

```rust
let zerv_draft = match args.input.source.as_str() {
    sources::GIT => super::git_pipeline::process_git_source(&work_dir, &args)?,
    sources::STDIN => super::stdin_pipeline::process_cached_stdin_source(&args, stdin_content)?,
    sources::NONE => super::none_pipeline::process_none_source(&args)?,
    source => return Err(ZervError::UnknownSource(source.to_string())),
};
```

#### 1.6 Flow: No Changes Needed

Flow pipeline calls `run_version_pipeline()` internally, so `none` source handling automatically applies.

**File:** `src/cli/flow/pipeline.rs` (line 24)

```rust
// This call already handles all sources including 'none'
let ron_output = run_version_pipeline(version_args, stdin_content)?;
```

---

### Phase 2: New `render` Subcommand

#### 2.1 Create RenderArgs

**New file:** `src/cli/render/mod.rs`

```rust
use clap::Parser;
use crate::cli::common::args::output::OutputConfig;
use crate::utils::constants::formats;

#[derive(Parser, Debug, Clone)]
pub struct RenderArgs {
    /// Version string to render
    #[arg(required = true, value_name = "VERSION")]
    pub version: String,

    /// Input format (auto-detected if not specified)
    #[arg(short = 'f', long = "input-format",
          default_value = formats::AUTO,
          value_parser = [formats::AUTO, formats::SEMVER, formats::PEP440, formats::ZERV],
          help = "Input format: 'auto', 'semver', 'pep440', or 'zerv'")]
    pub input_format: String,

    /// Output configuration (same as version/flow)
    #[command(flatten)]
    pub output: OutputConfig,
}
```

#### 2.2 Create Render Pipeline

**New file:** `src/cli/render/pipeline.rs`

```rust
use crate::cli::render::RenderArgs;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::error::ZervError;
use crate::version::{VersionObject, Zerv};

pub fn run_render(args: RenderArgs) -> Result<String, ZervError> {
    // 1. Parse version string using existing VersionObject infrastructure
    let version_object = VersionObject::parse_with_format(&args.version, &args.input_format)?;

    // 2. Convert VersionObject to ZervVars (existing impl From<VersionObject> for ZervVars)
    let vars = crate::version::ZervVars::from(version_object);

    // 3. Convert ZervVars to Zerv
    let zerv = Zerv::from_vars(vars)?;

    // 4. Use existing OutputFormatter - no need to reimplement!
    let output = OutputFormatter::format_output(
        &zerv,
        &args.output.output_format,
        args.output.output_prefix.as_deref(),
        &args.output.output_template,
    )?;

    Ok(output)
}
```

**Key insight:** We reuse `VersionObject::parse_with_format()` + `impl From<VersionObject> for ZervVars` + `OutputFormatter::format_output()` - all infrastructure already exists!

#### 2.3-2.4 Wire Up Render Command

**Modify:** `src/cli/parser.rs`

```rust
use crate::cli::render::RenderArgs;

#[derive(Subcommand, Debug)]
pub enum Commands {
    Version(...),
    Flow(...),
    Check(...),
    /// Render a version string with format conversion, templates, and schemas
    #[command(
        long_about = "Parse a version string and render it with flexible output options.
Supports format conversion (SemVer ↔ PEP440), normalization, templates, and schemas."
    )]
    Render(Box<RenderArgs>),
}
```

**Modify:** `src/cli/app.rs`

```rust
use crate::cli::render::run_render;

pub fn run_with_args<W: Write>(...) {
    // ...

    match cli.command {
        Some(Commands::Render(render_args)) => {
            let output = run_render(*render_args)?;
            writeln!(writer, "{output}")?;
        }
        // ...
    }
}
```

---

### Phase 3: Testing Strategy

#### 3.1 Source Tests (COMPLETED)

**Done:** `tests/integration_tests/version/main/sources/smart_default.rs`

- Test with stdin present → defaults to `stdin`
- Test without stdin → defaults to `git`
- Test explicit source overrides smart default
- Test `--source none` works

**Done:** `tests/integration_tests/version/main/sources/none.rs`

- Test `--source none` with `--tag-version`
- Test `--source none` with `--distance`
- Test `--source none` with `--dirty`
- Test `--source none` with all overrides
- Test `--source none` without `--tag-version` defaults to 0.0.0

**Done:** `tests/integration_tests/flow/sources.rs`

- Test flow with `--source stdin`
- Test flow smart default with stdin
- Test flow with `--source none`

#### 3.2 Render Tests

**New:** `tests/integration_tests/render/input_semver.rs`

- Render valid SemVer → various output formats
- Render invalid SemVer (should error)

**New:** `tests/integration_tests/render/input_pep440.rs`

- Render valid PEP440 → various output formats
- Test PEP440 normalization (separators, labels)

**New:** `tests/integration_tests/render/format_conversion.rs`

- SemVer → PEP440
- PEP440 → SemVer
- Round-trip tests

**New:** `tests/integration_tests/render/auto_detect.rs`

- Test auto format detection
- Ambiguous cases

**New:** `tests/integration_tests/render/schema_template.rs`

- Test with schema
- Test with template
- Test with prefix

---

### Phase 4: Documentation Updates

#### 4.1 CLI Help

Update `parser.rs` with render examples.

#### 4.2 README.md

Add render usage section.

---

## Success Criteria

1. ✅ `zerv version` defaults to `stdin` when piped, `git` otherwise
2. ✅ `zerv version --source none` works without VCS
3. ✅ `zerv render "1.2.3rc4" --output-format pep440` outputs normalized version
4. ✅ `zerv render "1.2.3" --schema calver --prefix v` works
5. ✅ Format conversion works (PEP440 ↔ SemVer ↔ Zerv)
6. ✅ All tests pass
7. ✅ Documentation updated

---

## File Summary

### New Files

- `src/cli/version/none_pipeline.rs`
- `src/cli/render/mod.rs`
- `src/cli/render/pipeline.rs`
- `src/cli/render/tests/*.rs`
- `tests/integration_tests/render/*.rs`
- `tests/integration_tests/version/main/sources/none.rs`
- `tests/integration_tests/version/main/sources/smart_default.rs`
- `tests/integration_tests/flow/sources.rs`

### Modified Files

- `src/utils/constants.rs` - Add `sources::NONE`
- `src/cli/common/args/input.rs` - Change source to `Option<String>` + add `apply_smart_source_default()` method + tests
- `src/cli/version/args/mod.rs` - Update `validate()` to accept `stdin_content`
- `src/cli/flow/args/validation.rs` - Update `validate()` to accept `stdin_content`
- `src/cli/version/pipeline.rs` - Handle `none` source
- `src/cli/version/mod.rs` - Export none_pipeline
- `src/cli/parser.rs` - Add `Render` command
- `src/cli/app.rs` - Render handler
- `tests/integration_tests/version/main/sources/mod.rs` - Add none and smart_default modules
- `tests/integration_tests/flow/mod.rs` - Add sources module

---

## Example Usage

```bash
# Smart source default
zerv version < cached.zerv    # Uses stdin
zerv version                  # Uses git

# No source - overrides only
zerv version --source none --tag-version 1.2.3 --distance 5

# Render - format conversion
zerv render "1.2.3-alpha.4" --input-format semver --output-format pep440
# Output: 1.2.3a4

# Render - normalize PEP440
zerv render "1.2.3_RC4" --output-format pep440
# Output: 1.2.3rc4

# Render - with schema
zerv render "1.2.3" --schema calver
# Output: 2025.02.11

# Render - with template
zerv render "1.2.3" --template "v{{major}}.{{minor}}"
# Output: v1.2

# Render - with prefix
zerv render "1.2.3" --prefix "release-"
# Output: release-1.2.3
```
