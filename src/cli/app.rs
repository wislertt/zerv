use crate::version::pep440::PEP440;
use clap::Command;
use std::io::Write;

pub fn create_app() -> Command {
    Command::new("zerv")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Dynamic versioning CLI")
}

pub fn format_version(version: &PEP440) -> String {
    format!("{version}")
}

pub fn run_with_args<W: Write>(
    args: Vec<String>,
    mut writer: W,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app();
    let _matches = app.try_get_matches_from(args)?;

    let version = PEP440::new(vec![1, 2, 3]);
    let output = format_version(&version);
    writeln!(writer, "{output}")?;
    writeln!(writer, "Debug: {version:?}")?;
    writeln!(writer, "Display: {version:#?}")?;
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
    use rstest::rstest;

    #[rstest]
    #[case(vec![1, 2, 3], "1.2.3")]
    #[case(vec![2, 5, 10], "2.5.10")]
    #[case(vec![0, 0, 1], "0.0.1")]
    fn test_format_version(#[case] release: Vec<u32>, #[case] expected: &str) {
        let version = PEP440::new(release);
        assert_eq!(format_version(&version), expected);
    }

    #[test]
    fn test_create_app() {
        let app = create_app();
        assert_eq!(app.get_name(), "zerv");
    }

    #[test]
    fn test_run_with_args() {
        let mut output = Vec::new();
        let args = vec!["zerv".to_string()];

        run_with_args(args, &mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("1.2.3"));
        assert!(output_str.contains("Debug: PEP440"));
    }

    #[test]
    fn test_run_with_args_invalid_flag() {
        let mut output = Vec::new();
        let args = vec!["zerv".to_string(), "--invalid-flag".to_string()];

        let result = run_with_args(args, &mut output);
        assert!(result.is_err());
    }

    #[test]
    fn test_run() {
        // Test that run function doesn't panic with valid args
        // We can't easily test the error paths without mocking std::env::args
        // and std::process::exit, so we just ensure it compiles and can be called
        let _test_compile = run; // Ensures function exists and compiles
    }
}
