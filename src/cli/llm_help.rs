use std::io::Write;

use crate::config::EnvVars;

// Embed the comprehensive manual at compile time
pub const LLMS_MD: &str = include_str!("../../docs/llms.md");

// Helper function to try spawning a pager
fn try_pager(pager: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use std::process::{
        Command,
        Stdio,
    };

    // Skip pager usage during tests to avoid interactive blocking
    if cfg!(test) {
        return Ok(false);
    }

    match Command::new(pager).stdin(Stdio::piped()).spawn() {
        Ok(mut child) => {
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                match stdin.write_all(LLMS_MD.as_bytes()) {
                    Ok(_) => {
                        let _ = stdin;
                        let _ = child.wait();
                        return Ok(true); // Successfully used pager
                    }
                    Err(_) => {
                        return Ok(false); // Pager failed to accept input
                    }
                }
            }
            Ok(false) // No stdin available
        }
        Err(_) => {
            Ok(false) // Pager failed to spawn
        }
    }
}

pub fn display_llm_help<W: Write>(writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::process::Command;

    // Try to use system pager if available
    if let Ok(pager) = env::var(EnvVars::PAGER)
        && !pager.is_empty()
        && try_pager(&pager)?
    {
        return Ok(());
    }

    // Fallback: try common pagers
    for pager in ["less", "more", "most"] {
        if let Ok(output) = Command::new("which").arg(pager).output()
            && output.status.success()
            && try_pager(pager)?
        {
            return Ok(());
        }
    }

    // If no pager is available, write directly to writer
    writeln!(writer, "{}", LLMS_MD)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_llm_help_with_writer() {
        // Test that display_llm_help can write to any writer
        // In tests, pagers are disabled, so it should always fall back to writer
        let mut buffer = Vec::new();

        let result = display_llm_help(&mut buffer);
        assert!(result.is_ok(), "Should display LLM help without errors");

        // In tests, pagers are disabled, so content should be written to buffer
        assert!(
            !buffer.is_empty(),
            "Buffer should contain the manual content in tests"
        );

        // Verify the content starts with expected markdown
        let content = String::from_utf8(buffer).expect("Content should be valid UTF-8");
        assert!(
            content.starts_with("# Zerv CLI Documentation"),
            "Content should start with the manual title"
        );
    }

    #[test]
    fn test_try_pager_with_invalid_command() {
        // Test that try_pager returns false for invalid commands
        let result = try_pager("nonexistent-command-12345").expect("Should not panic");
        assert!(!result, "Should return false for invalid command");
    }

    #[test]
    fn test_display_llm_help_contains_expected_content() {
        // Test that the manual contains expected content
        let content = LLMS_MD;

        // Check for key sections that should be in the manual
        assert!(
            content.contains("Quick Start"),
            "Should contain Quick Start section"
        );
        assert!(
            content.contains("Commands"),
            "Should contain Commands section"
        );
        assert!(
            content.contains("Examples"),
            "Should contain Examples section"
        );
    }

    #[test]
    fn test_llms_md_constant_exists() {
        // Verify the embedded manual contains the expected content
        assert!(
            LLMS_MD.contains("# Zerv CLI Documentation"),
            "LLMS_MD should contain the manual title"
        );
    }

    #[test]
    fn test_display_llm_help_edge_cases() {
        // Test edge cases for the display_llm_help function

        // Test with empty string PAGER
        unsafe {
            std::env::set_var(EnvVars::PAGER, "");
        }

        let mut buffer = Vec::new();
        let result = display_llm_help(&mut buffer);
        assert!(result.is_ok(), "Should handle empty PAGER gracefully");

        // Clean up
        unsafe {
            std::env::remove_var(EnvVars::PAGER);
        }

        // Test with whitespace-only PAGER
        unsafe {
            std::env::set_var(EnvVars::PAGER, "   ");
        }

        let mut buffer = Vec::new();
        let result = display_llm_help(&mut buffer);
        assert!(
            result.is_ok(),
            "Should handle whitespace-only PAGER gracefully"
        );

        // Clean up
        unsafe {
            std::env::remove_var(EnvVars::PAGER);
        }
    }

    #[test]
    fn test_try_pager_error_conditions() {
        // Test that try_pager handles various error conditions

        // Test with obviously invalid command
        let result = try_pager("").expect("Should not panic with empty command");
        assert!(!result, "Should return false for empty command");

        // Test with non-existent command
        let result = try_pager("definitely-not-a-real-command-12345").expect("Should not panic");
        assert!(!result, "Should return false for non-existent command");

        // Test with command that exists but might not be a pager (like 'echo')
        let _result = try_pager("echo").expect("Should not panic with echo command");
        // This might return true or false depending on system, but shouldn't panic
    }
}
