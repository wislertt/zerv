use crate::constants::*;
use crate::error::ZervError;
use crate::version::pep440::PEP440;
use crate::version::semver::SemVer;
use clap::Parser;
use std::str::FromStr;

#[derive(Parser)]
pub struct CheckArgs {
    /// Version string to validate
    pub version: String,

    /// Format to validate against
    #[arg(short, long)]
    pub format: Option<String>,
}

pub fn run_check_command(args: CheckArgs) -> Result<(), ZervError> {
    match args.format.as_deref() {
        Some(FORMAT_PEP440) => {
            PEP440::from_str(&args.version)
                .map_err(|_| ZervError::InvalidVersion(args.version.clone()))?;
            println!("✓ Valid PEP440 version");
        }
        Some(FORMAT_SEMVER) => {
            SemVer::from_str(&args.version)
                .map_err(|_| ZervError::InvalidVersion(args.version.clone()))?;
            println!("✓ Valid SemVer version");
        }
        None => {
            // Auto-detect format
            let pep440_valid = PEP440::from_str(&args.version).is_ok();
            let semver_valid = SemVer::from_str(&args.version).is_ok();

            if pep440_valid {
                println!("✓ Valid PEP440 version");
            }
            if semver_valid {
                println!("✓ Valid SemVer version");
            }
            if !pep440_valid && !semver_valid {
                return Err(ZervError::InvalidVersion(args.version));
            }
        }
        Some(format) => {
            eprintln!("Unknown format: {format}");
            eprintln!("Supported formats: {}", SUPPORTED_FORMATS.join(", "));
            return Err(ZervError::UnknownFormat(format.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_check_args_defaults() {
        use clap::Parser;
        let args = CheckArgs::try_parse_from(["zerv", "1.2.3"]).unwrap();
        assert_eq!(args.version, "1.2.3");
        assert!(args.format.is_none());
    }

    #[rstest]
    #[case("1.2.3", Some(FORMAT_PEP440), true)]
    #[case("1.2.3", Some(FORMAT_SEMVER), true)]
    #[case("1.2.3", None, true)]
    #[case("invalid", None, false)]
    #[case("1.2.3", Some("unknown"), false)]
    fn test_run_check_command(
        #[case] version: &str,
        #[case] format: Option<&str>,
        #[case] should_succeed: bool,
    ) {
        let args = CheckArgs {
            version: version.to_string(),
            format: format.map(|s| s.to_string()),
        };
        let result = run_check_command(args);
        assert_eq!(result.is_ok(), should_succeed);
    }

    #[test]
    fn test_run_check_command_unknown_format_error_type() {
        let args = CheckArgs {
            version: "1.2.3".to_string(),
            format: Some("unknown".to_string()),
        };
        let result = run_check_command(args);
        assert!(matches!(result, Err(ZervError::UnknownFormat(_))));
    }
}
