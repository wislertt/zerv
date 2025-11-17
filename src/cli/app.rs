use std::io::{
    IsTerminal,
    Read,
    Write,
};

use clap::Parser;

use crate::cli::check::run_check_command;
use crate::cli::flow::run_flow_pipeline;
use crate::cli::llm_help::display_llm_help;
use crate::cli::parser::{
    Cli,
    Commands,
};
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
            run_check_command(check_args)?;
        }
        None => {
            // No subcommand provided, but --llm-help was not used either
            // This will be handled by clap's default behavior
        }
    }
    Ok(())
}

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
        // Check if it's a clap help/version exit
        if let Some(clap_err) = e.downcast_ref::<clap::Error>() {
            match clap_err.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    print!("{clap_err}");
                    return;
                }
                _ => {}
            }
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
