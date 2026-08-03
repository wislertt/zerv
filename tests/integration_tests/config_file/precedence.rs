//! Precedence matrix for config resolution (Phase 7).
//!
//! Asserts the full layer order end-to-end through the real binary:
//!
//! ```text
//! CLI flag  >  --config-file  >  discovered zerv.toml  >  builtin default
//! ```
//!
//! The global `~/.config/zerv/config.toml` layer is deferred (D7) and absent
//! from the matrix; the env layer is intentionally absent (D8). `--config-file`
//! and discovery are mutually exclusive at the load layer — an explicit path is
//! read *instead of* walking for `zerv.toml`, so a present-but-overridden
//! discovered file is simply never read.
//!
//! Every cell exercises the same field (`output_template`) over the same stdin
//! version (`1.2.3`, so `{{major}}` → `1`), with a distinct single-letter
//! template per source so the winner is unambiguous:
//!
//! | discovered `zerv.toml` | `--config-file` | CLI flag | winner          |
//! | ---------------------- | --------------- | -------- | --------------- |
//! | —                      | —               | —        | builtin `1.2.3` |
//! | `D`                    | —               | —        | discovered `D1` |
//! | —                      | `C`             | —        | file `C1`       |
//! | —                      | —               | `X`      | CLI `X1`        |
//! | `D`                    | —               | `X`      | CLI `X1`        |
//! | —                      | `C`             | `X`      | CLI `X1`        |
//! | `D`                    | `C`             | —        | file `C1`       |
//! | `D`                    | `C`             | `X`      | CLI `X1`        |
//! | `D`                    | `/dev/null`     | —        | builtin `1.2.3` |
//!
//! Stdin source carries a known version so no git data is read — no Docker
//! gating (mirrors the `config_file/mod.rs` wiring tests).

use std::path::Path;

use super::{
    repo_root,
    repo_with_config,
    stdin_version,
};
use crate::util::TestCommand;

/// Which explicit file source `--config-file` points at, if any.
#[derive(Clone, Copy)]
enum FileSrc {
    /// No `--config-file` flag at all.
    None,
    /// A planted `explicit.toml` whose template is `C{{major}}`.
    Explicit,
    /// `/dev/null` — the empty file disables config (no overrides).
    DevNull,
}

/// Run `zerv version` with a given combination of sources and return trimmed
/// stdout. `discovered` plants a repo-root `zerv.toml` (`D{{major}}`); `file`
/// chooses the `--config-file` target; `cli` adds a CLI `--output-template`
/// (`X{{major}}`).
fn resolve(discovered: bool, file: FileSrc, cli: bool) -> String {
    let dir = if discovered {
        repo_with_config("output_template = \"D{{major}}\"\n")
    } else {
        repo_root()
    };

    let explicit = dir.path().join("explicit.toml");
    if matches!(file, FileSrc::Explicit) {
        std::fs::write(&explicit, "output_template = \"C{{major}}\"\n")
            .expect("write explicit config");
    }

    let mut cmd = TestCommand::new();
    cmd.current_dir(dir.path()).stdin(stdin_version(1, 2, 3));

    match file {
        FileSrc::Explicit => {
            cmd.arg("--config-file").arg(&explicit);
        }
        FileSrc::DevNull => {
            cmd.arg("--config-file").arg(Path::new("/dev/null"));
        }
        FileSrc::None => {}
    }

    let mut args = String::from("version");
    if cli {
        args.push_str(" --output-template \"X{{major}}\"");
    }
    cmd.args_from_str(&args);

    cmd.assert_success().stdout().trim().to_string()
}

#[test]
fn builtin_only_when_no_sources() {
    assert_eq!(resolve(false, FileSrc::None, false), "1.2.3");
}

#[test]
fn discovered_only_wins_over_builtin() {
    assert_eq!(resolve(true, FileSrc::None, false), "D1");
}

#[test]
fn config_file_only_wins_over_builtin() {
    assert_eq!(resolve(false, FileSrc::Explicit, false), "C1");
}

#[test]
fn cli_only_wins_over_builtin() {
    assert_eq!(resolve(false, FileSrc::None, true), "X1");
}

#[test]
fn cli_wins_over_discovered() {
    assert_eq!(resolve(true, FileSrc::None, true), "X1");
}

#[test]
fn cli_wins_over_config_file() {
    assert_eq!(resolve(false, FileSrc::Explicit, true), "X1");
}

#[test]
fn config_file_wins_over_discovered() {
    assert_eq!(resolve(true, FileSrc::Explicit, false), "C1");
}

#[test]
fn cli_wins_over_config_file_and_discovered() {
    assert_eq!(resolve(true, FileSrc::Explicit, true), "X1");
}

#[test]
fn dev_null_disables_discovered_config() {
    assert_eq!(resolve(true, FileSrc::DevNull, false), "1.2.3");
}
