use std::io::Write;

use clap::Parser;

use crate::cli::check::run_check_command;
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

    match cli.command {
        Commands::Version(version_args) => {
            let output = run_version_pipeline(*version_args)?;
            writeln!(writer, "{output}")?;
        }
        Commands::Check(check_args) => {
            run_check_command(check_args)?;
        }
    }
    Ok(())
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
