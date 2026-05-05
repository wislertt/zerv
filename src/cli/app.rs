use std::io::{
    IsTerminal,
    Read,
    Write,
};

use clap::{
    CommandFactory,
    Parser,
};

use crate::cli::check::run_check_command;
use crate::cli::flow::run_flow_pipeline;
use crate::cli::llm_help::display_llm_help;
use crate::cli::parser::{
    Cli,
    Commands,
};
use crate::cli::render::run_render;
use crate::cli::version::run_version_pipeline;

pub fn run_with_args<W: Write>(
    args: Vec<String>,
    mut writer: W,
) -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::try_parse_from(args)?;

    crate::logging::init_logging(cli.verbose);

    tracing::debug!("Zerv started with args: {:?}", cli);

    // Handle --llm-help flag
    if cli.llm_help {
        display_llm_help(&mut writer)?;
        return Ok(());
    }

    // Extract stdin content once at the beginning
    let stdin_content = extract_stdin_once()?;

    match cli.command {
        Some(Commands::Version(version_args)) => {
            let output = run_version_pipeline(*version_args, stdin_content.as_deref())?;
            writeln!(writer, "{output}")?;
        }
        Some(Commands::Flow(flow_args)) => {
            let output = run_flow_pipeline(*flow_args, stdin_content.as_deref())?;
            writeln!(writer, "{output}")?;
        }
        Some(Commands::Check(check_args)) => {
            let output = run_check_command(check_args)?;
            writeln!(writer, "{output}")?;
        }
        Some(Commands::Render(render_args)) => {
            let output = run_render(*render_args)?;
            writeln!(writer, "{output}")?;
        }
        None => {
            return Err(Box::new(NoSubcommand));
        }
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

/// Extract stdin content once, regardless of command
/// Returns Ok(Some(String)) if stdin is available, Ok(None) otherwise
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

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = run_with_args(args, std::io::stdout()) {
        if e.downcast_ref::<NoSubcommand>().is_some() {
            let mut cmd = Cli::command();
            cmd.print_long_help().unwrap_or_default();
            std::io::stdout().flush().unwrap_or_default();
            eprintln!();
            std::process::exit(2);
        }
        if let Some(clap_err) = e.downcast_ref::<clap::Error>() {
            clap_err.exit();
        }
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        // Test that run function doesn't panic with valid args
        // We can't easily test the error paths without mocking std::env::args
        // and std::process::exit, so we just ensure it compiles and can be called
        let _test_compile = run; // Ensures function exists and compiles
    }
}
