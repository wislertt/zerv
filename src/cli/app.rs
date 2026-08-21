use std::io::{
    IsTerminal,
    Read,
    Write,
};

use clap::{
    CommandFactory,
    FromArgMatches,
};

use crate::cli::check::run_check_command;
use crate::cli::flow::run_flow_pipeline;
use crate::cli::parser::{
    Cli,
    Commands,
};
use crate::cli::render::run_render;
use crate::cli::version::run_version_pipeline;
use crate::config::merge;

pub fn run_with_args<W: Write>(
    args: Vec<String>,
    mut writer: W,
) -> Result<(), Box<dyn std::error::Error>> {
    // Hand-parse to retain ArgMatches — the merge layer reads value_source to
    // tell CLI flags from clap defaults.
    let matches = Cli::command().try_get_matches_from(args)?;
    let cli = Cli::from_arg_matches(&matches)?;

    crate::logging::init_logging(cli.verbose);

    tracing::debug!("Zerv started with args: {:?}", cli);

    let stdin_content = extract_stdin_once()?;

    // Dispatch on the typed command + raw sub_matches (borrowed so the merge
    // layer reads provenance).
    let sub = matches.subcommand();
    // --config-file is global: read once from the top-level matches and forward.
    let config_file: Option<&str> = matches
        .get_one::<String>(crate::utils::constants::arg_ids::CONFIG_FILE)
        .map(String::as_str);
    match (cli.command, sub) {
        (Some(Commands::Version(mut version_args)), Some((_, sub_matches))) => {
            if let Some(file) = &merge::load_for(sub_matches, config_file)? {
                merge::apply_to_version(&mut version_args, sub_matches, file);
            }
            let output = run_version_pipeline(*version_args, stdin_content.as_deref())?;
            writeln!(writer, "{output}")?;
        }
        (Some(Commands::Flow(mut flow_args)), Some((_, sub_matches))) => {
            if let Some(file) = &merge::load_for(sub_matches, config_file)? {
                merge::apply_to_flow(&mut flow_args, sub_matches, file)?;
            }
            let output = run_flow_pipeline(*flow_args, stdin_content.as_deref())?;
            writeln!(writer, "{output}")?;
        }
        (Some(Commands::Check(check_args)), _) => {
            let output = run_check_command(check_args)?;
            writeln!(writer, "{output}")?;
        }
        (Some(Commands::Render(render_args)), _) => {
            let output = run_render(*render_args)?;
            writeln!(writer, "{output}")?;
        }
        (None, _) => {
            return Err(Box::new(NoSubcommand));
        }
        _ => return Err(Box::new(NoSubcommand)),
    }
    Ok(())
}

#[derive(Debug)]
struct NoSubcommand;

impl std::fmt::Display for NoSubcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no subcommand provided")
    }
}

impl std::error::Error for NoSubcommand {}

fn extract_stdin_once() -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Check if stdin is being piped
    if std::io::stdin().is_terminal() {
        return Ok(None);
    }

    let mut input = String::new();
    match std::io::stdin().read_to_string(&mut input) {
        Ok(_) => {
            if input.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(input))
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}

pub fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = run_with_args(args, std::io::stdout()) {
        if e.downcast_ref::<NoSubcommand>().is_some() {
            let mut cmd = Cli::command();
            cmd.print_long_help().unwrap_or_default();
            std::io::stdout().flush().unwrap_or_default();
            eprintln!();
            return 2;
        }
        if let Some(clap_err) = e.downcast_ref::<clap::Error>() {
            clap_err.exit();
        }
        eprintln!("Error: {e}");
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: Vec<&str>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        run_with_args(args, &mut buf)?;
        Ok(buf)
    }

    #[test]
    fn no_subcommand_returns_error() {
        let err = run(vec!["zerv"]).unwrap_err();
        assert!(err.downcast_ref::<NoSubcommand>().is_some());
        assert_eq!(err.to_string(), "no subcommand provided");
    }

    #[test]
    fn unknown_flag_returns_error() {
        let err = run(vec!["zerv", "--bogus"]).unwrap_err();
        assert!(err.downcast_ref::<clap::Error>().is_some());
    }

    #[test]
    fn invalid_subcommand_returns_error() {
        let err = run(vec!["zerv", "nonexistent"]).unwrap_err();
        assert!(err.downcast_ref::<clap::Error>().is_some());
    }

    #[test]
    fn verbose_flag_without_subcommand_returns_error() {
        let err = run(vec!["zerv", "--verbose"]).unwrap_err();
        assert!(err.downcast_ref::<NoSubcommand>().is_some());
    }

    #[test]
    fn no_subcommand_display_message() {
        let err = NoSubcommand;
        assert_eq!(err.to_string(), "no subcommand provided");
    }

    #[test]
    fn no_subcommand_is_error() {
        let err: Box<dyn std::error::Error> = Box::new(NoSubcommand);
        assert!(err.downcast_ref::<NoSubcommand>().is_some());
    }

    #[test]
    fn check_subcommand_dispatches() {
        let buf = run(vec!["zerv", "check", "1.2.3"]).expect("check must validate 1.2.3");
        assert!(!buf.is_empty(), "check must emit output");
    }

    #[test]
    fn render_subcommand_dispatches() {
        let buf = run(vec!["zerv", "render", "1.2.3"]).expect("render must render 1.2.3");
        assert!(!buf.is_empty(), "render must emit output");
    }

    #[test]
    fn version_subcommand_applies_config_file_template() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join("zerv.toml");
        std::fs::write(&cfg, "output_template = \"v{{major}}\"\n").unwrap();
        // --source none avoids stdin/git so the file template reaches output only via the merge layer.
        let args: Vec<String> = [
            "zerv",
            "--config-file",
            cfg.to_str().unwrap(),
            "version",
            "--source",
            "none",
            "--tag-version",
            "v1.2.3",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let mut buf = Vec::new();
        run_with_args(args, &mut buf).expect("version --source none must succeed");
        assert_eq!(
            String::from_utf8(buf).unwrap().trim(),
            "v1",
            "the file's output_template must apply via the merge layer"
        );
    }

    #[test]
    fn flow_subcommand_runs_merge_layer() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join("zerv.toml");
        std::fs::write(&cfg, "").unwrap();
        // --source none keeps flow off stdin; a clean run proves the flow arm wires config.
        let args: Vec<String> = [
            "zerv",
            "--config-file",
            cfg.to_str().unwrap(),
            "flow",
            "--source",
            "none",
            "--schema",
            "standard",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let mut buf = Vec::new();
        run_with_args(args, &mut buf).expect("flow --source none must succeed");
        assert!(
            !buf.is_empty(),
            "flow must emit output after the merge layer applies"
        );
    }

    mod run_exit_code {
        use super::*;

        #[test]
        fn success_returns_zero() {
            let args: Vec<String> = ["zerv", "check", "1.2.3"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            let result = run_with_args(args, Vec::new());
            assert!(result.is_ok());
        }

        #[test]
        fn no_subcommand_returns_two() {
            let args: Vec<String> = vec!["zerv".to_string()];
            let result = run_with_args(args, Vec::new());
            assert!(result.is_err());
            assert!(result.unwrap_err().downcast_ref::<NoSubcommand>().is_some());
        }
    }
}
