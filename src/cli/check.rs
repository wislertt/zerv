use crate::constants::{
    FORMAT_NAME_PEP440, FORMAT_NAME_SEMVER, FORMAT_PEP440, FORMAT_SEMVER, SUPPORTED_FORMAT_NAMES,
    SUPPORTED_FORMATS,
};
use crate::error::ZervError;
use crate::version::pep440::PEP440;
use crate::version::semver::SemVer;
use clap::Parser;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Parser)]
pub struct CheckArgs {
    /// Version string to validate
    pub version: String,

    /// Format to validate against
    #[arg(short, long)]
    pub format: Option<String>,
}

fn print_format_validation<T: Display>(original: &str, parsed: &T, format_name: &str) {
    if original == parsed.to_string() {
        println!("✓ Valid {format_name} format");
    } else {
        println!("✓ Valid {format_name} format (normalized: {parsed})");
    }
}

pub fn run_check_command(args: CheckArgs) -> Result<(), ZervError> {
    match args.format.as_deref() {
        Some(FORMAT_PEP440) => {
            let parsed = PEP440::from_str(&args.version).map_err(|_| {
                ZervError::InvalidVersion(format!(
                    "{} - Invalid {} format",
                    args.version, FORMAT_NAME_PEP440
                ))
            })?;
            println!("Version: {}", args.version);
            print_format_validation(&args.version, &parsed, FORMAT_NAME_PEP440);
        }
        Some(FORMAT_SEMVER) => {
            let parsed = SemVer::from_str(&args.version).map_err(|_| {
                ZervError::InvalidVersion(format!(
                    "{} - Invalid {} format",
                    args.version, FORMAT_NAME_SEMVER
                ))
            })?;
            println!("Version: {}", args.version);
            print_format_validation(&args.version, &parsed, FORMAT_NAME_SEMVER);
        }
        None => {
            // Auto-detect format
            let pep440_result = PEP440::from_str(&args.version);
            let semver_result = SemVer::from_str(&args.version);

            if pep440_result.is_err() && semver_result.is_err() {
                return Err(ZervError::InvalidVersion(format!(
                    "{} - Invalid for all supported formats ({})",
                    args.version,
                    SUPPORTED_FORMAT_NAMES.join(", ")
                )));
            }

            println!("Version: {}", args.version);

            if let Ok(ref parsed) = pep440_result {
                print_format_validation(&args.version, parsed, FORMAT_NAME_PEP440);
            }
            if let Ok(ref parsed) = semver_result {
                print_format_validation(&args.version, parsed, FORMAT_NAME_SEMVER);
            }
        }
        Some(format) => {
            eprintln!("✗ Unknown format: {format}");
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
    #[case("1.2.3", Some(FORMAT_PEP440))]
    #[case("1.2.3", Some(FORMAT_SEMVER))]
    #[case("1.2.3", None)]
    fn test_run_check_command_success(#[case] version: &str, #[case] format: Option<&str>) {
        let args = CheckArgs {
            version: version.to_string(),
            format: format.map(|s| s.to_string()),
        };
        let result = run_check_command(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_check_command_invalid_version() {
        let args = CheckArgs {
            version: "invalid".to_string(),
            format: None,
        };
        let result = run_check_command(args);
        assert!(matches!(result, Err(ZervError::InvalidVersion(_))));
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
