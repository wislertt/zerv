use clap::{
    Parser,
    Subcommand,
};

use crate::cli::check::CheckArgs;
use crate::cli::version::VersionArgs;

#[derive(Parser)]
#[command(name = "zerv")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Dynamic versioning CLI - Generate versions from VCS data with flexible schemas")]
#[command(
    long_about = "Zerv is a dynamic versioning tool that generates version strings from version control \
system (VCS) data using configurable schemas. It supports multiple input sources, output formats, \
and advanced override capabilities for CI/CD workflows.

EXAMPLES:
  # Basic version generation from git
  zerv version

  # Generate PEP440 format with custom schema
  zerv version --output-format pep440 --schema calver

  # Override VCS values for testing
  zerv version --tag-version v2.0.0 --distance 5 --dirty true

  # Force clean release state
  zerv version --clean

  # Pipe Zerv RON between commands
  zerv version --output-format zerv | zerv version --source stdin --schema calver

  # Use in different directory
  zerv version -C /path/to/repo"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate version from VCS data with configurable schemas and overrides
    #[command(
        long_about = "Generate version strings from version control system data using configurable schemas.
Supports multiple input sources (git, stdin), output formats (semver, pep440, zerv), and VCS overrides
for testing and CI/CD workflows."
    )]
    Version(Box<VersionArgs>),
    /// Validate version string format compliance
    #[command(
        long_about = "Validate that version strings conform to specific format requirements.
Supports SemVer, PEP440, and other version format validation."
    )]
    Check(CheckArgs),
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_cli_structure() {
        // Test that CLI can be parsed
        let cli = Cli::try_parse_from(["zerv", "version"]).unwrap();
        assert!(matches!(cli.command, Commands::Version(_)));

        let cli = Cli::try_parse_from(["zerv", "check", "1.0.0"]).unwrap();
        assert!(matches!(cli.command, Commands::Check(_)));
    }

    #[test]
    fn test_cli_with_directory() {
        let cli = Cli::try_parse_from(["zerv", "version", "-C", "/tmp"]).unwrap();
        assert!(matches!(cli.command, Commands::Version(_)));
        if let Commands::Version(version_args) = cli.command {
            assert_eq!(version_args.directory, Some("/tmp".to_string()));
        }
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
