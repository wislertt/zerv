use crate::cli::check::CheckArgs;
use crate::cli::version::VersionArgs;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zerv")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Dynamic versioning CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate version from VCS data
    Version(VersionArgs),
    /// Validate version string format
    Check(CheckArgs),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_cli_structure() {
        // Test that CLI can be parsed
        let cli = Cli::try_parse_from(["zerv", "version"]).unwrap();
        assert!(matches!(cli.command, Commands::Version(_)));

        let cli = Cli::try_parse_from(["zerv", "check", "1.0.0"]).unwrap();
        assert!(matches!(cli.command, Commands::Check(_)));
    }

    #[rstest]
    #[case(vec!["zerv", "version"], true)]
    #[case(vec!["zerv", "check", "1.0.0"], true)]
    #[case(vec!["zerv", "invalid"], false)]
    fn test_cli_parsing(#[case] args: Vec<&str>, #[case] should_succeed: bool) {
        let result = Cli::try_parse_from(args);
        assert_eq!(result.is_ok(), should_succeed);
    }
}
