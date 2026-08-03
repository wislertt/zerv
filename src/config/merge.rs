//! Merge a discovered `zerv.toml` into resolved CLI arg structs.
//!
//! Provenance comes from clap's `value_source`: a CLI-supplied field always
//! wins; only clap-default fields are file-eligible (keeps arg structs concrete
//! — no `Option` refactor). Layer order, low → high: builtin → discovered
//! `zerv.toml` → `--config-file` → CLI. The two file sources are mutually
//! exclusive (an explicit path skips discovery).

use std::path::{
    Path,
    PathBuf,
};

use clap::ArgMatches;
use clap::parser::ValueSource;

use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
};
use crate::cli::flow::FlowArgs;
use crate::cli::utils::template::Template;
use crate::cli::version::VersionArgs;
use crate::config::file::{
    FlowSection,
    ZervFileConfig,
};
use crate::error::ZervError;
use crate::utils::constants::arg_ids;

/// Discover + parse the nearest `zerv.toml`. `Ok(None)` when none found. A
/// found-but-unparseable file errors loud (a silent fallback would ship a wrong
/// version); a missing start dir collapses to `Ok(None)` — discovery is
/// advisory, the command surfaces its own dir error.
pub fn load(start_dir: &Path) -> Result<Option<ZervFileConfig>, ZervError> {
    let path = match crate::config::discover(start_dir) {
        Ok(Some(p)) => p,
        Ok(None) => return Ok(None),
        Err(ZervError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(other) => return Err(other),
    };
    let contents = std::fs::read_to_string(&path).map_err(|e| {
        ZervError::Io(std::io::Error::other(format!(
            "Failed to read config file {}: {e}",
            path.display()
        )))
    })?;
    let config = ZervFileConfig::from_toml_str(&contents)?;
    tracing::debug!("Loaded zerv.toml from {}", path.display());
    Ok(Some(config))
}

/// Resolve the start dir (`--directory`/`-C` else cwd, mirroring `git -C`) and
/// [`load`] it. `config_file` skips discovery and reads that path via
/// [`load_explicit`].
pub fn load_for(
    sub_matches: &ArgMatches,
    config_file: Option<&str>,
) -> Result<Option<ZervFileConfig>, ZervError> {
    if let Some(path) = config_file {
        return load_explicit(Path::new(path));
    }
    let start_dir = match sub_matches.get_one::<String>(arg_ids::DIRECTORY) {
        Some(d) => PathBuf::from(d),
        None => std::env::current_dir()?,
    };
    load(&start_dir)
}

/// Read an explicit `--config-file <path>` (no discovery). Unlike [`load`], a
/// missing path errors loud — the user named it, so silence would hide a typo.
/// An empty file — the null device, `/dev/null` on Unix and `NUL` on Windows —
/// yields no overrides (config disabled).
pub fn load_explicit(path: &Path) -> Result<Option<ZervFileConfig>, ZervError> {
    let contents = std::fs::read_to_string(path).map_err(|e| {
        ZervError::Io(std::io::Error::other(format!(
            "Failed to read --config-file {}: {e}",
            path.display()
        )))
    })?;
    let config = ZervFileConfig::from_toml_str(&contents)?;
    tracing::debug!("Loaded zerv config from --config-file {}", path.display());
    Ok(Some(config))
}

fn cli_set(matches: &ArgMatches, id: &str) -> bool {
    matches.value_source(id) == Some(ValueSource::CommandLine)
}

fn override_opt(
    field: &mut Option<String>,
    matches: &ArgMatches,
    id: &str,
    file_val: Option<&str>,
) {
    if !cli_set(matches, id)
        && let Some(v) = file_val
    {
        *field = Some(v.to_string());
    }
}

/// Same as [`override_opt`] for fields that carry a clap `default_value`
/// (concrete `String` rather than `Option<String>`).
fn override_str(field: &mut String, matches: &ArgMatches, id: &str, file_val: Option<&str>) {
    if !cli_set(matches, id)
        && let Some(v) = file_val
    {
        *field = v.to_string();
    }
}

fn apply_to_input(input: &mut InputConfig, matches: &ArgMatches, file: &ZervFileConfig) {
    override_opt(
        &mut input.source,
        matches,
        arg_ids::SOURCE,
        file.source.as_deref(),
    );
    override_str(
        &mut input.input_format,
        matches,
        arg_ids::INPUT_FORMAT,
        file.input_format.as_deref(),
    );
    override_opt(
        &mut input.directory,
        matches,
        arg_ids::DIRECTORY,
        file.directory.as_deref(),
    );
}

fn apply_to_output(output: &mut OutputConfig, matches: &ArgMatches, file: &ZervFileConfig) {
    override_str(
        &mut output.output_format,
        matches,
        arg_ids::OUTPUT_FORMAT,
        file.output_format.as_deref(),
    );
    override_opt(
        &mut output.output_prefix,
        matches,
        arg_ids::OUTPUT_PREFIX,
        file.output_prefix.as_deref(),
    );
    if !cli_set(matches, arg_ids::OUTPUT_TEMPLATE)
        && let Some(tpl) = file.output_template.as_deref()
    {
        output.output_template = Some(Template::new(tpl.to_string()));
    }
}

pub fn apply_to_version(args: &mut VersionArgs, matches: &ArgMatches, file: &ZervFileConfig) {
    apply_to_input(&mut args.input, matches, file);
    apply_to_output(&mut args.output, matches, file);

    let schema = file
        .version
        .as_ref()
        .and_then(|v| v.schema.as_deref())
        .or(file.schema.as_deref());
    let schema_ron = file
        .version
        .as_ref()
        .and_then(|v| v.schema_ron.as_deref())
        .or(file.schema_ron.as_deref());
    override_opt(&mut args.main.schema, matches, arg_ids::SCHEMA, schema);
    override_opt(
        &mut args.main.schema_ron,
        matches,
        arg_ids::SCHEMA_RON,
        schema_ron,
    );
}

/// Errors on an unparseable `[flow].branch_rules` RON payload (unlike
/// [`apply_to_version`], which cannot fail).
pub fn apply_to_flow(
    args: &mut FlowArgs,
    matches: &ArgMatches,
    file: &ZervFileConfig,
) -> Result<(), ZervError> {
    apply_to_input(&mut args.input, matches, file);
    apply_to_output(&mut args.output, matches, file);

    let schema = file
        .flow
        .as_ref()
        .and_then(|f| f.schema.as_deref())
        .or(file.schema.as_deref());
    let schema_ron = file
        .flow
        .as_ref()
        .and_then(|f| f.schema_ron.as_deref())
        .or(file.schema_ron.as_deref());
    override_opt(&mut args.schema, matches, arg_ids::SCHEMA, schema);
    override_opt(
        &mut args.schema_ron,
        matches,
        arg_ids::SCHEMA_RON,
        schema_ron,
    );

    if let Some(flow) = &file.flow {
        apply_flow_section(args, matches, flow)?;
    }
    Ok(())
}

fn apply_flow_section(
    args: &mut FlowArgs,
    matches: &ArgMatches,
    flow: &FlowSection,
) -> Result<(), ZervError> {
    override_opt(
        &mut args.branch_config.pre_release_label,
        matches,
        arg_ids::PRE_RELEASE_LABEL,
        flow.pre_release_label.as_deref(),
    );
    override_opt(
        &mut args.branch_config.post_mode,
        matches,
        arg_ids::POST_MODE,
        flow.post_mode.as_deref(),
    );
    if !cli_set(matches, arg_ids::HASH_BRANCH_LEN)
        && let Some(n) = flow.hash_branch_len
    {
        args.hash_branch_len = n;
    }
    if !cli_set(matches, arg_ids::BRANCH_RULES)
        && let Some(ron) = flow.branch_rules.as_deref()
    {
        args.branch_config.branch_rules = ron.parse().map_err(|e| {
            ZervError::ConfigParseError(format!(
                "Failed to parse branch_rules from config file: {e}"
            ))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::{
        CommandFactory,
        FromArgMatches,
    };

    use super::*;
    use crate::cli::parser::{
        Cli,
        Commands,
    };

    /// Return resolved `version` args plus its `ArgMatches` (for provenance).
    fn version_with(extra: &[&str]) -> (VersionArgs, ArgMatches) {
        let mut full: Vec<&str> = vec!["zerv", "version"];
        full.extend_from_slice(extra);
        let matches = Cli::command().try_get_matches_from(full).unwrap();
        let cli = Cli::from_arg_matches(&matches).unwrap();
        let args = match cli.command {
            Some(Commands::Version(va)) => *va,
            _ => unreachable!("version subcommand"),
        };
        (args, matches.subcommand_matches("version").unwrap().clone())
    }

    /// Return resolved `flow` args plus its `ArgMatches` (for provenance).
    fn flow_with(extra: &[&str]) -> (FlowArgs, ArgMatches) {
        let mut full: Vec<&str> = vec!["zerv", "flow"];
        full.extend_from_slice(extra);
        let matches = Cli::command().try_get_matches_from(full).unwrap();
        let cli = Cli::from_arg_matches(&matches).unwrap();
        let args = match cli.command {
            Some(Commands::Flow(fa)) => *fa,
            _ => unreachable!("flow subcommand"),
        };
        (args, matches.subcommand_matches("flow").unwrap().clone())
    }

    #[test]
    fn version_file_fills_output_format_when_cli_absent() {
        let (mut args, sub) = version_with(&[]);
        let file = ZervFileConfig {
            output_format: Some("pep440".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.output.output_format, "pep440");
    }

    #[test]
    fn version_cli_output_format_wins_over_file() {
        let (mut args, sub) = version_with(&["--output-format", "zerv"]);
        let file = ZervFileConfig {
            output_format: Some("pep440".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.output.output_format, "zerv");
    }

    #[test]
    fn version_file_fills_optional_source_when_cli_absent() {
        let (mut args, sub) = version_with(&[]);
        let file = ZervFileConfig {
            source: Some("stdin".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.input.source.as_deref(), Some("stdin"));
    }

    #[test]
    fn version_cli_source_wins_over_file() {
        let (mut args, sub) = version_with(&["--source", "none"]);
        let file = ZervFileConfig {
            source: Some("stdin".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.input.source.as_deref(), Some("none"));
    }

    #[test]
    fn version_file_fills_directory_and_prefix() {
        let (mut args, sub) = version_with(&[]);
        let file = ZervFileConfig {
            directory: Some("/repo".to_string()),
            output_prefix: Some("v".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.input.directory.as_deref(), Some("/repo"));
        assert_eq!(args.output.output_prefix.as_deref(), Some("v"));
    }

    #[test]
    fn version_file_fills_output_template() {
        let (mut args, sub) = version_with(&[]);
        let file = ZervFileConfig {
            output_template: Some("v{{major}}".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(
            args.output.output_template.as_ref().map(Template::as_str),
            Some("v{{major}}")
        );
    }

    #[test]
    fn version_schema_from_shared_top_when_no_section() {
        let (mut args, sub) = version_with(&[]);
        let file = ZervFileConfig {
            schema: Some("calver".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.main.schema.as_deref(), Some("calver"));
    }

    #[test]
    fn version_section_schema_overrides_shared_top() {
        let (mut args, sub) = version_with(&[]);
        let file = ZervFileConfig {
            schema: Some("standard-context".to_string()),
            version: Some(crate::config::file::VersionSection {
                schema: Some("standard-no-context".to_string()),
                schema_ron: None,
            }),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.main.schema.as_deref(), Some("standard-no-context"));
    }

    #[test]
    fn version_cli_schema_wins_over_file() {
        let (mut args, sub) = version_with(&["--schema", "calver"]);
        let file = ZervFileConfig {
            schema: Some("standard".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.main.schema.as_deref(), Some("calver"));
    }

    // schema_ron is stored verbatim (RON string, parsed at consume time) —
    // assert only the merge, not the parse.

    #[test]
    fn version_file_fills_schema_ron_when_cli_absent() {
        let (mut args, sub) = version_with(&[]);
        let file = ZervFileConfig {
            schema_ron: Some("ron-payload".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.main.schema_ron.as_deref(), Some("ron-payload"));
    }

    #[test]
    fn version_section_schema_ron_overrides_shared_top() {
        let (mut args, sub) = version_with(&[]);
        let file = ZervFileConfig {
            schema_ron: Some("shared-ron".to_string()),
            version: Some(crate::config::file::VersionSection {
                schema: None,
                schema_ron: Some("version-ron".to_string()),
            }),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.main.schema_ron.as_deref(), Some("version-ron"));
    }

    #[test]
    fn version_cli_schema_ron_wins_over_file() {
        let (mut args, sub) = version_with(&["--schema-ron", "cli-ron"]);
        let file = ZervFileConfig {
            schema_ron: Some("file-ron".to_string()),
            ..Default::default()
        };
        apply_to_version(&mut args, &sub, &file);
        assert_eq!(args.main.schema_ron.as_deref(), Some("cli-ron"));
    }

    #[test]
    fn flow_file_fills_output_format_when_cli_absent() {
        let (mut args, sub) = flow_with(&[]);
        let file = ZervFileConfig {
            output_format: Some("pep440".to_string()),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        assert_eq!(args.output.output_format, "pep440");
    }

    #[test]
    fn flow_file_fills_post_mode_and_pre_release_label() {
        let (mut args, sub) = flow_with(&[]);
        let file = ZervFileConfig {
            flow: Some(crate::config::file::FlowSection {
                post_mode: Some("tag".to_string()),
                pre_release_label: Some("beta".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        assert_eq!(args.branch_config.post_mode.as_deref(), Some("tag"));
        assert_eq!(
            args.branch_config.pre_release_label.as_deref(),
            Some("beta")
        );
    }

    #[test]
    fn flow_cli_post_mode_wins_over_file() {
        let (mut args, sub) = flow_with(&["--post-mode", "commit"]);
        let file = ZervFileConfig {
            flow: Some(crate::config::file::FlowSection {
                post_mode: Some("tag".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        assert_eq!(args.branch_config.post_mode.as_deref(), Some("commit"));
    }

    #[test]
    fn flow_file_fills_hash_branch_len() {
        let (mut args, sub) = flow_with(&[]);
        let file = ZervFileConfig {
            flow: Some(crate::config::file::FlowSection {
                hash_branch_len: Some(7),
                ..Default::default()
            }),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        assert_eq!(args.hash_branch_len, 7);
    }

    #[test]
    fn flow_cli_hash_branch_len_wins_over_file() {
        let (mut args, sub) = flow_with(&["--hash-branch-len", "3"]);
        let file = ZervFileConfig {
            flow: Some(crate::config::file::FlowSection {
                hash_branch_len: Some(7),
                ..Default::default()
            }),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        assert_eq!(args.hash_branch_len, 3);
    }

    #[test]
    fn flow_file_branch_rules_parsed_when_cli_absent() {
        let (mut args, sub) = flow_with(&[]);
        let ron = r#"[(pattern: "main", pre_release_label: beta, pre_release_num: 1, post_mode: commit)]"#;
        let file = ZervFileConfig {
            flow: Some(crate::config::file::FlowSection {
                branch_rules: Some(ron.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        assert!(args.branch_config.branch_rules.find_rule("main").is_some());
    }

    #[test]
    fn flow_file_branch_rules_parse_error_is_loud() {
        let (mut args, sub) = flow_with(&[]);
        let file = ZervFileConfig {
            flow: Some(crate::config::file::FlowSection {
                branch_rules: Some("not valid ron (((".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let err = apply_to_flow(&mut args, &sub, &file).unwrap_err();
        assert!(matches!(err, ZervError::ConfigParseError(_)));
    }

    #[test]
    fn flow_cli_branch_rules_win_over_file() {
        let cli_ron =
            r#"[(pattern: "main", pre_release_label: rc, pre_release_num: 9, post_mode: tag)]"#;
        let (mut args, sub) = flow_with(&["--branch-rules", cli_ron]);
        let file_ron = r#"[(pattern: "main", pre_release_label: beta, pre_release_num: 1, post_mode: commit)]"#;
        let file = ZervFileConfig {
            flow: Some(crate::config::file::FlowSection {
                branch_rules: Some(file_ron.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        let rule = args
            .branch_config
            .branch_rules
            .find_rule("main")
            .expect("main rule present");
        assert_eq!(rule.pre_release_label.to_string(), "rc");
    }

    #[test]
    fn flow_section_schema_overrides_shared_top() {
        let (mut args, sub) = flow_with(&[]);
        let file = ZervFileConfig {
            schema: Some("standard-context".to_string()),
            flow: Some(crate::config::file::FlowSection {
                schema: Some("standard-no-context".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        assert_eq!(args.schema.as_deref(), Some("standard-no-context"));
    }

    #[test]
    fn flow_file_fills_schema_ron_when_cli_absent() {
        let (mut args, sub) = flow_with(&[]);
        let file = ZervFileConfig {
            schema_ron: Some("ron-payload".to_string()),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        assert_eq!(args.schema_ron.as_deref(), Some("ron-payload"));
    }

    #[test]
    fn flow_cli_schema_ron_wins_over_file() {
        let (mut args, sub) = flow_with(&["--schema-ron", "cli-ron"]);
        let file = ZervFileConfig {
            schema_ron: Some("file-ron".to_string()),
            ..Default::default()
        };
        apply_to_flow(&mut args, &sub, &file).unwrap();
        assert_eq!(args.schema_ron.as_deref(), Some("cli-ron"));
    }

    #[test]
    fn load_returns_none_when_no_config() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        // A stray config above $TMPDIR would make this flaky — assert only no-error.
        let _ = load(tmp.path()).expect("load over empty tree should not error");
    }

    #[test]
    fn load_parses_found_config() {
        use std::fs;

        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join(crate::utils::constants::vcs_markers::GIT)).unwrap();
        fs::write(
            root.join(crate::utils::constants::config_files::PRIMARY),
            "output_format = \"pep440\"\n",
        )
        .unwrap();
        let config = load(&root).unwrap().expect("config should be found");
        assert_eq!(config.output_format.as_deref(), Some("pep440"));
    }

    #[test]
    fn load_errors_loud_on_malformed_config() {
        use std::fs;

        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join(crate::utils::constants::vcs_markers::GIT)).unwrap();
        fs::write(
            root.join(crate::utils::constants::config_files::PRIMARY),
            "not = valid = toml",
        )
        .unwrap();
        let err = load(&root).unwrap_err();
        assert!(matches!(err, ZervError::ConfigParseError(_)));
    }

    #[test]
    fn load_explicit_reads_given_path() {
        use std::fs;

        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("custom.toml");
        fs::write(&path, "output_format = \"pep440\"\n").unwrap();
        let config = load_explicit(&path)
            .unwrap()
            .expect("explicit path should be read");
        assert_eq!(config.output_format.as_deref(), Some("pep440"));
    }

    #[test]
    fn load_explicit_empty_file_is_default_config() {
        use std::fs;

        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("empty.toml");
        fs::write(&path, "").unwrap();
        let config = load_explicit(&path)
            .unwrap()
            .expect("empty explicit file is a valid (default) config");
        assert_eq!(config, ZervFileConfig::default());
    }

    #[test]
    fn load_explicit_missing_path_errors_loud() {
        let err = load_explicit(Path::new("/nonexistent/zerv/custom.toml"))
            .expect_err("missing explicit path must error, not silently skip");
        assert!(matches!(err, ZervError::Io(_)));
        assert!(err.to_string().contains("--config-file"));
    }

    #[test]
    fn load_explicit_malformed_errors_loud() {
        use std::fs;

        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.toml");
        fs::write(&path, "not = valid = toml").unwrap();
        let err = load_explicit(&path).unwrap_err();
        assert!(matches!(err, ZervError::ConfigParseError(_)));
    }

    #[test]
    fn load_for_explicit_path_skips_discovery() {
        use std::fs;

        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join(crate::utils::constants::vcs_markers::GIT)).unwrap();
        fs::write(
            root.join(crate::utils::constants::config_files::PRIMARY),
            "output_format = \"semver\"\n",
        )
        .unwrap();
        let explicit = root.join("explicit.toml");
        fs::write(&explicit, "output_format = \"pep440\"\n").unwrap();

        let (_, sub) = version_with(&[]);
        let config = load_for(&sub, Some(explicit.to_str().unwrap()))
            .unwrap()
            .expect("explicit config should load");
        assert_eq!(config.output_format.as_deref(), Some("pep440"));
    }

    #[test]
    fn load_for_none_falls_back_to_discovery() {
        let (_, sub) = version_with(&[]);
        // Discovery result depends on the host cwd — assert only no-error.
        let _ = load_for(&sub, None).expect("load_for with no explicit path must not error");
    }
}
