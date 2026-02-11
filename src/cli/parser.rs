use clap::{
    Parser,
    Subcommand,
};

use crate::cli::check::CheckArgs;
use crate::cli::flow::FlowArgs;
use crate::cli::render::RenderArgs;
use crate::cli::version::VersionArgs;

#[derive(Parser, Debug)]
#[command(name = "zerv")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Dynamic versioning CLI - Generate versions from VCS data with flexible schemas")]
#[command(
    long_about = "Zerv is a dynamic versioning tool that generates version strings from version control \
system (VCS) data using configurable schemas. It supports multiple input sources, output formats, \
and advanced override capabilities for CI/CD workflows.

Use --llm-help to display the comprehensive CLI manual with detailed examples and guidance.

EXAMPLES:
  # Flow - intelligent versioning based on git state
  zerv flow

  # Flow - use specific schema preset
  zerv flow --schema standard

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
  zerv version -C /path/to/repo

  # Render - convert SemVer to PEP440
  zerv render 1.2.3-alpha.1 --output-format pep440

  # Render - use template for custom output
  zerv render 1.2.3 --template 'v{{major}}.{{minor}}'

  # Render - add prefix
  zerv render 1.2.3 --output-prefix release-
"
)]
pub struct Cli {
    /// Use verbose output (enables debug-level logs to stderr).
    /// Use RUST_LOG for fine-grained control (e.g., RUST_LOG=zerv::vcs=debug)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Display comprehensive CLI manual for humans and AI assistants
    #[arg(long = "llm-help", help = "Display comprehensive CLI manual")]
    pub llm_help: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate version from VCS data with configurable schemas and overrides
    #[command(
        long_about = "Generate version strings from version control system data using configurable schemas.
Supports multiple input sources (git, stdin), output formats (semver, pep440, zerv), and VCS overrides
for testing and CI/CD workflows."
    )]
    Version(Box<VersionArgs>),
    /// Generate version with intelligent pre-release management based on Git branch patterns
    #[command(
        long_about = "Generate version strings with automatic pre-release detection based on Git branch patterns.
This command acts as an intelligent wrapper around 'zerv version' that automatically determines
pre-release information from the current Git branch using configurable pattern matching."
    )]
    Flow(Box<FlowArgs>),
    /// Validate version string format compliance
    #[command(
        long_about = "Validate that version strings conform to specific format requirements.
Supports SemVer, PEP440, and other version format validation."
    )]
    Check(CheckArgs),
    /// Render a version string with format conversion and output options
    #[command(
        long_about = "Parse a version string and render it with flexible output options.
Supports format conversion (SemVer ↔ PEP440), normalization, templates, and custom prefixes."
    )]
    Render(Box<RenderArgs>),
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_cli_structure() {
        // Test that CLI can be parsed
        let cli = Cli::try_parse_from(["zerv", "version"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Version(_))));

        let cli = Cli::try_parse_from(["zerv", "flow"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Flow(_))));

        let cli = Cli::try_parse_from(["zerv", "check", "1.0.0"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Check(_))));

        let cli = Cli::try_parse_from(["zerv", "render", "1.2.3"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Render(_))));
    }

    #[test]
    fn test_cli_with_directory() {
        let cli = Cli::try_parse_from(["zerv", "version", "-C", "/tmp"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Version(_))));
        if let Some(Commands::Version(version_args)) = cli.command {
            assert_eq!(version_args.input.directory, Some("/tmp".to_string()));
        }

        let cli = Cli::try_parse_from(["zerv", "flow", "-C", "/tmp"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Flow(_))));
        if let Some(Commands::Flow(flow_args)) = cli.command {
            assert_eq!(flow_args.input.directory, Some("/tmp".to_string()));
        }
    }

    #[rstest]
    #[case(vec!["zerv", "version"], true)]
    #[case(vec!["zerv", "flow"], true)]
    #[case(vec!["zerv", "check", "1.0.0"], true)]
    #[case(vec!["zerv", "render", "1.2.3"], true)]
    #[case(vec!["zerv", "invalid"], false)]
    fn test_cli_parsing(#[case] args: Vec<&str>, #[case] should_succeed: bool) {
        let result = Cli::try_parse_from(args);
        assert_eq!(result.is_ok(), should_succeed);
    }
}
