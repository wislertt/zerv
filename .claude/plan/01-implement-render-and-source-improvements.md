# Implementation Plan: Render and Source Improvements

**Status:** Planned
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
- [ ] 1.7 Add tests for `none` source

**Note:** Flow uses `run_version_pipeline()` internally, so `none` source handling automatically applies to flow. No separate flow implementation needed.

### Phase 2: New `render` Subcommand

- [ ] 2.1 Create `src/cli/render/mod.rs` with `RenderArgs`
- [ ] 2.2 Create `src/cli/render/parser.rs` for format conversion
    - [ ] 2.2.1 `parse_semver_to_zerv()` function
    - [ ] 2.2.2 `parse_pep440_to_zerv()` function
    - [ ] 2.2.3 `detect_and_parse()` auto-detection function
- [ ] 2.3 Create `src/cli/render/pipeline.rs` using `OutputFormatter`
- [ ] 2.4 Add `Render` command to `Commands` enum in `parser.rs`
- [ ] 2.5 Wire up render handler in `app.rs`

### Phase 3: Testing

- [ ] 3.1 Source tests for version
- [ ] 3.2 Source tests for flow
- [ ] 3.3 Render tests - SemVer
- [ ] 3.4 Render tests - PEP440
- [ ] 3.5 Render tests - format conversion
- [ ] 3.6 Render tests - auto-detect
- [ ] 3.7 Render tests - schema and template

### Phase 4: Documentation

- [ ] 4.1 Update CLI help text in `parser.rs`
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

#### 2.2 Create Format Parser

**New file:** `src/cli/render/parser.rs`

```rust
use crate::error::ZervError;
use crate::utils::constants;
use crate::version::zerv::core::Zerv;
use crate::version::semver::core::SemVer;
use crate::version::pep440::core::PEP440;

pub fn parse_to_zerv(version: &str, format: &str) -> Result<Zerv, ZervError> {
    match format {
        constants::formats::AUTO => detect_and_parse(version),
        constants::formats::SEMVER => parse_semver_to_zerv(version),
        constants::formats::PEP440 => parse_pep440_to_zerv(version),
        constants::formats::ZERV => version.parse(),
        _ => Err(ZervError::UnsupportedFormat(format.to_string())),
    }
}

fn parse_semver_to_zerv(version: &str) -> Result<Zerv, ZervError> {
    let semver: SemVer = version.parse()?;
    // Convert SemVer to Zerv
    // Map: major/minor/patch, pre-release -> pre_release, build -> discard (no equivalent)
    Ok(/* converted Zerv */)
}

fn parse_pep440_to_zerv(version: &str) -> Result<Zerv, ZervError> {
    let pep440: PEP440 = version.parse()?;
    // Convert PEP440 to Zerv
    // Map: release, epoch, pre, post, dev, local
    Ok(/* converted Zerv */)
}

fn detect_and_parse(version: &str) -> Result<Zerv, ZervError> {
    // Try each format in order
    // Return first successful parse
}
```

#### 2.3 Create Render Pipeline

**New file:** `src/cli/render/pipeline.rs`

```rust
use crate::cli::render::RenderArgs;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::cli::render::parser::parse_to_zerv;
use crate::error::ZervError;

pub fn run_render(args: RenderArgs) -> Result<String, ZervError> {
    // 1. Parse input version to Zerv
    let zerv = parse_to_zerv(&args.version, &args.input_format)?;

    // 2. Use existing OutputFormatter - no need to reimplement!
    let output = OutputFormatter::format_output(
        &zerv,
        &args.output.output_format,
        args.output.output_prefix.as_deref(),
        &args.output.output_template,
    )?;

    Ok(output)
}
```

**Key insight:** We reuse `OutputFormatter::format_output()` directly - all rendering logic already exists!

#### 2.4-2.5 Wire Up Render Command

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

#### 3.1 Source Tests

**New:** `src/cli/version/tests/source_default.rs`

- Test with stdin present → defaults to `stdin`
- Test without stdin → defaults to `git`

**New:** `src/cli/version/tests/none_source.rs`

- Test `--source none` uses only overrides
- Test VCS is skipped

**Note:** Flow tests are covered by existing integration tests since flow uses `run_version_pipeline()` internally.

#### 3.2 Render Tests

**New:** `src/cli/render/tests/render_semver.rs`

- Parse valid SemVer
- Parse invalid SemVer (should error)

**New:** `src/cli/render/tests/render_pep440.rs`

- Parse valid PEP440
- Test normalization (separators, labels)

**New:** `src/cli/render/tests/format_conversion.rs`

- SemVer → PEP440
- PEP440 → SemVer
- Round-trip tests

**New:** `src/cli/render/tests/auto_detect.rs`

- Test format detection
- Ambiguous cases

**New:** `src/cli/render/tests/schema_template.rs`

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
- `src/cli/render/parser.rs`
- `src/cli/render/pipeline.rs`
- `src/cli/render/tests/*.rs`
- `src/cli/version/tests/source_default.rs`
- `src/cli/version/tests/none_source.rs`
- `src/cli/flow/tests/source_default.rs`

### Modified Files

- `src/utils/constants.rs` - Add `sources::NONE`
- `src/cli/common/args/input.rs` - Change source to `Option<String>` + add `apply_smart_source_default()` method
- `src/cli/version/args/mod.rs` - Update `validate()` to accept `stdin_content`
- `src/cli/flow/args/validation.rs` - Update `validate()` to accept `stdin_content`
- `src/cli/parser.rs` - Add `Render` command
- `src/cli/version/pipeline.rs` - Handle `none` source
- `src/cli/app.rs` - Render handler
- `tests/integration_tests/help_flags.rs` - Update expected help text for `none` source
- `src/cli/flow/pipeline.rs` - Handle `none` source

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
