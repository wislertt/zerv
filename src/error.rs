use std::io;

/// Main error type for the zerv library
#[derive(Debug)]
pub enum ZervError {
    // VCS errors
    /// VCS not found or not available
    VcsNotFound(String),
    /// No tags found matching pattern
    NoTagsFound,
    /// Command execution failed
    CommandFailed(String),

    // Version errors
    /// Invalid version format
    InvalidFormat(String),
    /// Invalid version string
    InvalidVersion(String),

    // Schema errors
    /// Schema parsing error
    SchemaParseError(String),
    /// Unknown schema name
    UnknownSchema(String),
    /// Conflicting schema parameters
    ConflictingSchemas(String),

    // CLI errors
    /// Unknown format specified
    UnknownFormat(String),
    /// Stdin input error
    StdinError(String),
    /// Unknown source specified
    UnknownSource(String),
    /// Conflicting CLI options
    ConflictingOptions(String),

    // System errors
    /// IO error
    Io(io::Error),
    /// Regex error
    Regex(String),
}

impl std::fmt::Display for ZervError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // VCS errors
            ZervError::VcsNotFound(vcs) => write!(f, "VCS not found: {vcs}"),
            ZervError::NoTagsFound => write!(f, "No version tags found in git repository"),
            ZervError::CommandFailed(msg) => write!(f, "Command execution failed: {msg}"),

            // Version errors
            ZervError::InvalidFormat(msg) => write!(f, "Invalid version format: {msg}"),
            ZervError::InvalidVersion(msg) => write!(f, "Invalid version: {msg}"),

            // Schema errors
            ZervError::SchemaParseError(msg) => write!(f, "Schema parse error: {msg}"),
            ZervError::UnknownSchema(name) => write!(f, "Unknown schema: {name}"),
            ZervError::ConflictingSchemas(msg) => write!(f, "Conflicting schemas: {msg}"),

            // CLI errors
            ZervError::UnknownFormat(format) => write!(f, "Unknown format: {format}"),
            ZervError::StdinError(msg) => write!(f, "Stdin error: {msg}"),
            ZervError::UnknownSource(source) => write!(f, "Unknown source: {source}"),
            ZervError::ConflictingOptions(msg) => write!(f, "Conflicting options: {msg}"),

            // System errors
            ZervError::Io(err) => write!(f, "IO error: {err}"),
            ZervError::Regex(msg) => write!(f, "Regex error: {msg}"),
        }
    }
}

impl std::error::Error for ZervError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ZervError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ZervError {
    fn from(err: io::Error) -> Self {
        ZervError::Io(err)
    }
}

/// Convert string errors to ZervError
impl From<String> for ZervError {
    fn from(err: String) -> Self {
        // For string errors, use a generic format error
        ZervError::InvalidFormat(err)
    }
}

impl PartialEq for ZervError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ZervError::VcsNotFound(a), ZervError::VcsNotFound(b)) => a == b,
            (ZervError::NoTagsFound, ZervError::NoTagsFound) => true,
            (ZervError::InvalidFormat(a), ZervError::InvalidFormat(b)) => a == b,
            (ZervError::InvalidVersion(a), ZervError::InvalidVersion(b)) => a == b,
            (ZervError::CommandFailed(a), ZervError::CommandFailed(b)) => a == b,
            (ZervError::Io(a), ZervError::Io(b)) => {
                a.kind() == b.kind() && a.to_string() == b.to_string()
            }
            (ZervError::Regex(a), ZervError::Regex(b)) => a == b,
            (ZervError::SchemaParseError(a), ZervError::SchemaParseError(b)) => a == b,
            (ZervError::UnknownSchema(a), ZervError::UnknownSchema(b)) => a == b,
            (ZervError::ConflictingSchemas(a), ZervError::ConflictingSchemas(b)) => a == b,
            (ZervError::UnknownFormat(a), ZervError::UnknownFormat(b)) => a == b,
            (ZervError::StdinError(a), ZervError::StdinError(b)) => a == b,
            (ZervError::UnknownSource(a), ZervError::UnknownSource(b)) => a == b,
            (ZervError::ConflictingOptions(a), ZervError::ConflictingOptions(b)) => a == b,
            _ => false,
        }
    }
}

/// Result type alias for zerv operations
pub type Result<T> = std::result::Result<T, ZervError>;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::error::Error;

    #[rstest]
    #[case(ZervError::VcsNotFound("git".to_string()), "VCS not found: git")]
    #[case(ZervError::NoTagsFound, "No version tags found in git repository")]
    #[case(ZervError::InvalidFormat("bad".to_string()), "Invalid version format: bad")]
    #[case(ZervError::InvalidVersion("1.0.0-invalid".to_string()), "Invalid version: 1.0.0-invalid")]
    #[case(ZervError::CommandFailed("exit 1".to_string()), "Command execution failed: exit 1")]
    #[case(ZervError::Regex("invalid".to_string()), "Regex error: invalid")]
    #[case(ZervError::SchemaParseError("bad ron".to_string()), "Schema parse error: bad ron")]
    #[case(ZervError::UnknownSchema("unknown".to_string()), "Unknown schema: unknown")]
    #[case(ZervError::ConflictingSchemas("both provided".to_string()), "Conflicting schemas: both provided")]
    #[case(ZervError::UnknownFormat("unknown".to_string()), "Unknown format: unknown")]
    #[case(ZervError::StdinError("no input".to_string()), "Stdin error: no input")]
    #[case(ZervError::UnknownSource("unknown".to_string()), "Unknown source: unknown")]
    #[case(ZervError::ConflictingOptions("--clean with --dirty".to_string()), "Conflicting options: --clean with --dirty")]
    fn test_error_display(#[case] error: ZervError, #[case] expected: &str) {
        assert_eq!(error.to_string(), expected);
    }

    /// Test specific error messages that match requirements exactly
    #[rstest]
    #[case(ZervError::NoTagsFound, "No version tags found in git repository")]
    #[case(ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()), "VCS not found: Not in a git repository (--source git)")]
    #[case(ZervError::CommandFailed("No commits found in git repository".to_string()), "Command execution failed: No commits found in git repository")]
    #[case(ZervError::CommandFailed("Git command not found. Please install git and try again.".to_string()), "Command execution failed: Git command not found. Please install git and try again.")]
    #[case(ZervError::CommandFailed("Permission denied accessing git repository".to_string()), "Command execution failed: Permission denied accessing git repository")]
    fn test_requirement_specific_error_messages(#[case] error: ZervError, #[case] expected: &str) {
        assert_eq!(error.to_string(), expected);
    }

    #[rstest]
    #[case(io::ErrorKind::NotFound, "file not found")]
    #[case(io::ErrorKind::PermissionDenied, "access denied")]
    #[case(io::ErrorKind::ConnectionRefused, "connection refused")]
    #[case(io::ErrorKind::TimedOut, "timed out")]
    fn test_io_error_conversion(#[case] kind: io::ErrorKind, #[case] message: &str) {
        let io_err = io::Error::new(kind, message);
        let zerv_err: ZervError = io_err.into();
        assert!(matches!(zerv_err, ZervError::Io(_)));
        assert!(zerv_err.to_string().contains(message));
    }

    #[rstest]
    #[case(ZervError::Io(io::Error::new(io::ErrorKind::NotFound, "test")), true)]
    #[case(ZervError::VcsNotFound("git".to_string()), false)]
    #[case(ZervError::NoTagsFound, false)]
    #[case(ZervError::InvalidFormat("bad".to_string()), false)]
    #[case(ZervError::InvalidVersion("bad".to_string()), false)]
    #[case(ZervError::CommandFailed("exit 1".to_string()), false)]
    #[case(ZervError::Regex("invalid".to_string()), false)]
    #[case(ZervError::SchemaParseError("bad".to_string()), false)]
    #[case(ZervError::UnknownSchema("unknown".to_string()), false)]
    #[case(ZervError::ConflictingSchemas("conflict".to_string()), false)]
    #[case(ZervError::UnknownFormat("unknown".to_string()), false)]
    #[case(ZervError::StdinError("no input".to_string()), false)]
    #[case(ZervError::UnknownSource("unknown".to_string()), false)]
    #[case(ZervError::ConflictingOptions("conflict".to_string()), false)]
    fn test_error_source(#[case] error: ZervError, #[case] has_source: bool) {
        assert_eq!(error.source().is_some(), has_source);
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<i32> = Ok(42);
        let err_result: Result<i32> = Err(ZervError::NoTagsFound);

        assert_eq!(ok_result, Ok(42));
        assert!(err_result.is_err());
    }

    #[rstest]
    #[case(
        ZervError::VcsNotFound("git".to_string()),
        ZervError::VcsNotFound("git".to_string()),
        true
    )]
    #[case(
        ZervError::VcsNotFound("git".to_string()),
        ZervError::VcsNotFound("hg".to_string()),
        false
    )]
    #[case(ZervError::NoTagsFound, ZervError::NoTagsFound, true)]
    #[case(
        ZervError::InvalidFormat("bad".to_string()),
        ZervError::InvalidFormat("bad".to_string()),
        true
    )]
    #[case(
        ZervError::InvalidFormat("bad".to_string()),
        ZervError::InvalidFormat("worse".to_string()),
        false
    )]
    #[case(
        ZervError::InvalidVersion("1.0".to_string()),
        ZervError::InvalidVersion("1.0".to_string()),
        true
    )]
    #[case(
        ZervError::CommandFailed("exit 1".to_string()),
        ZervError::CommandFailed("exit 1".to_string()),
        true
    )]
    #[case(
        ZervError::Regex("invalid".to_string()),
        ZervError::Regex("invalid".to_string()),
        true
    )]
    #[case(
        ZervError::SchemaParseError("bad".to_string()),
        ZervError::SchemaParseError("bad".to_string()),
        true
    )]
    #[case(
        ZervError::UnknownSchema("unknown".to_string()),
        ZervError::UnknownSchema("unknown".to_string()),
        true
    )]
    #[case(
        ZervError::ConflictingSchemas("conflict".to_string()),
        ZervError::ConflictingSchemas("conflict".to_string()),
        true
    )]
    #[case(
        ZervError::UnknownFormat("unknown".to_string()),
        ZervError::UnknownFormat("unknown".to_string()),
        true
    )]
    #[case(
        ZervError::StdinError("no input".to_string()),
        ZervError::StdinError("no input".to_string()),
        true
    )]
    #[case(
        ZervError::UnknownSource("git".to_string()),
        ZervError::UnknownSource("git".to_string()),
        true
    )]
    #[case(
        ZervError::ConflictingOptions("--clean with --dirty".to_string()),
        ZervError::ConflictingOptions("--clean with --dirty".to_string()),
        true
    )]
    #[case(
        ZervError::NoTagsFound,
        ZervError::VcsNotFound("git".to_string()),
        false
    )]
    fn test_error_equality(
        #[case] error1: ZervError,
        #[case] error2: ZervError,
        #[case] should_equal: bool,
    ) {
        assert_eq!(error1 == error2, should_equal);
    }

    #[test]
    fn test_io_error_equality() {
        // IO errors with same kind and message should be equal
        let io_err1 = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let io_err2 = io::Error::new(io::ErrorKind::NotFound, "file not found");
        assert_eq!(ZervError::Io(io_err1), ZervError::Io(io_err2));

        // IO errors with different kinds should not be equal
        let io_err3 = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let io_err4 = io::Error::new(io::ErrorKind::PermissionDenied, "file not found");
        assert_ne!(ZervError::Io(io_err3), ZervError::Io(io_err4));
    }

    /// Test equality with updated source-aware error messages
    #[rstest]
    #[case(
        ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()),
        ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()),
        true
    )]
    #[case(
        ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()),
        ZervError::VcsNotFound("VCS not found".to_string()),
        false
    )]
    #[case(
        ZervError::CommandFailed("No commits found in git repository".to_string()),
        ZervError::CommandFailed("No commits found in git repository".to_string()),
        true
    )]
    #[case(
        ZervError::CommandFailed("Git command not found. Please install git and try again.".to_string()),
        ZervError::CommandFailed("Git command not found. Please install git and try again.".to_string()),
        true
    )]
    #[case(
        ZervError::CommandFailed("Permission denied accessing git repository".to_string()),
        ZervError::CommandFailed("Permission denied accessing git repository".to_string()),
        true
    )]
    #[case(
        ZervError::CommandFailed("No commits found in git repository".to_string()),
        ZervError::CommandFailed("Git command failed".to_string()),
        false
    )]
    fn test_error_equality_with_updated_messages(
        #[case] error1: ZervError,
        #[case] error2: ZervError,
        #[case] should_equal: bool,
    ) {
        assert_eq!(error1 == error2, should_equal);
        assert_eq!(error2 == error1, should_equal); // Test symmetry
    }

    #[test]
    fn test_debug_trait() {
        let error = ZervError::VcsNotFound("git".to_string());
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("VcsNotFound"));
        assert!(debug_str.contains("git"));
    }

    /// Test debug output for all error variants to ensure helpful development information
    #[rstest]
    #[case(ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()), "VcsNotFound", "Not in a git repository")]
    #[case(ZervError::NoTagsFound, "NoTagsFound", "")]
    #[case(ZervError::CommandFailed("No commits found in git repository".to_string()), "CommandFailed", "No commits found")]
    #[case(ZervError::InvalidFormat("bad format".to_string()), "InvalidFormat", "bad format")]
    #[case(ZervError::InvalidVersion("1.0.0-invalid".to_string()), "InvalidVersion", "1.0.0-invalid")]
    #[case(ZervError::SchemaParseError("bad ron".to_string()), "SchemaParseError", "bad ron")]
    #[case(ZervError::UnknownSchema("unknown".to_string()), "UnknownSchema", "unknown")]
    #[case(ZervError::ConflictingSchemas("both provided".to_string()), "ConflictingSchemas", "both provided")]
    #[case(ZervError::UnknownFormat("unknown".to_string()), "UnknownFormat", "unknown")]
    #[case(ZervError::Regex("invalid regex".to_string()), "Regex", "invalid regex")]
    fn test_debug_output_helpful_for_development(
        #[case] error: ZervError,
        #[case] variant_name: &str,
        #[case] content: &str,
    ) {
        let debug_str = format!("{error:?}");
        assert!(
            debug_str.contains(variant_name),
            "Debug output should contain variant name: {debug_str}"
        );
        if !content.is_empty() {
            assert!(
                debug_str.contains(content),
                "Debug output should contain error content: {debug_str}"
            );
        }
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = ZervError::NoTagsFound;
        let _: &dyn Error = &error; // Ensure Error trait is implemented
    }

    /// Test conversion from String to ZervError
    #[rstest]
    #[case(
        "Some format error",
        ZervError::InvalidFormat("Some format error".to_string())
    )]
    fn test_string_to_zerv_error_conversion(#[case] input: &str, #[case] expected: ZervError) {
        let result: ZervError = input.to_string().into();
        assert_eq!(result, expected);
    }

    /// Test that error message consistency is maintained across all variants
    #[test]
    fn test_error_message_consistency() {
        // Test that source-aware messages include source information
        let vcs_error =
            ZervError::VcsNotFound("Not in a git repository (--source git)".to_string());
        assert!(vcs_error.to_string().contains("git repository"));
        assert!(vcs_error.to_string().contains("--source git"));

        // Test that NoTagsFound includes source information
        let no_tags_error = ZervError::NoTagsFound;
        assert!(no_tags_error.to_string().contains("git repository"));

        // Test that command failed errors are clear and actionable
        let git_not_found = ZervError::CommandFailed(
            "Git command not found. Please install git and try again.".to_string(),
        );
        assert!(git_not_found.to_string().contains("install git"));

        let permission_denied =
            ZervError::CommandFailed("Permission denied accessing git repository".to_string());
        assert!(permission_denied.to_string().contains("Permission denied"));
        assert!(permission_denied.to_string().contains("git repository"));

        let no_commits = ZervError::CommandFailed("No commits found in git repository".to_string());
        assert!(no_commits.to_string().contains("commits"));
        assert!(no_commits.to_string().contains("git repository"));
    }

    /// Comprehensive validation of error message consistency across all error types
    /// This test validates requirements 5.1, 5.2, and 5.3 from the error handling improvements spec
    #[test]
    fn test_comprehensive_error_message_validation() {
        // Test 1: All VCS-related errors should be source-aware (Requirement 5.1)
        let vcs_errors = vec![
            ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()),
            ZervError::NoTagsFound,
            ZervError::CommandFailed("No commits found in git repository".to_string()),
            ZervError::CommandFailed(
                "Git command not found. Please install git and try again.".to_string(),
            ),
            ZervError::CommandFailed("Permission denied accessing git repository".to_string()),
        ];

        for error in &vcs_errors {
            let error_msg = error.to_string();
            assert!(
                error_msg.contains("git") || error_msg.contains("Git"),
                "VCS error should be source-aware and mention git: {error_msg}"
            );
        }

        // Test 2: Error messages should use consistent terminology (Requirement 5.2)
        let terminology_tests = vec![
            (ZervError::NoTagsFound, "git repository"),
            (
                ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()),
                "git repository",
            ),
            (
                ZervError::CommandFailed("No commits found in git repository".to_string()),
                "git repository",
            ),
            (
                ZervError::CommandFailed("Permission denied accessing git repository".to_string()),
                "git repository",
            ),
        ];

        for (error, expected_term) in terminology_tests {
            assert!(
                error.to_string().contains(expected_term),
                "Error should use consistent terminology '{expected_term}': {error}"
            );
        }

        // Test 3: Actionable guidance should be provided where appropriate (Requirement 5.3)
        let actionable_errors = vec![
            (
                ZervError::CommandFailed(
                    "Git command not found. Please install git and try again.".to_string(),
                ),
                "install git",
            ),
            (
                ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()),
                "--source git",
            ),
        ];

        for (error, guidance) in actionable_errors {
            assert!(
                error.to_string().contains(guidance),
                "Error should provide actionable guidance '{guidance}': {error}"
            );
        }

        // Test 4: No generic error messages should be used for VCS operations
        let generic_patterns = vec!["IO error", "VCS data", "Command execution failed"];

        for error in &vcs_errors {
            let error_msg = error.to_string();
            for pattern in &generic_patterns {
                if pattern == &"Command execution failed" {
                    // This is acceptable as a prefix for CommandFailed errors
                    continue;
                }
                assert!(
                    !error_msg.contains(pattern),
                    "Error should not contain generic pattern '{pattern}': {error_msg}"
                );
            }
        }

        // Test 5: Error messages should be extensible for future VCS types (Requirement 5.2)
        // Test that the error structure supports different sources
        let future_vcs_error =
            ZervError::VcsNotFound("Not in a mercurial repository (--source hg)".to_string());
        assert!(future_vcs_error.to_string().contains("mercurial"));
        assert!(future_vcs_error.to_string().contains("--source hg"));
    }

    /// Test that all error messages follow consistent formatting patterns
    #[test]
    fn test_error_message_formatting_consistency() {
        let test_cases = vec![
            // VCS errors should mention the specific source
            (
                ZervError::VcsNotFound("Not in a git repository (--source git)".to_string()),
                vec!["git", "--source"],
            ),
            (ZervError::NoTagsFound, vec!["git repository"]),
            // Command errors should be clear and specific
            (
                ZervError::CommandFailed(
                    "Git command not found. Please install git and try again.".to_string(),
                ),
                vec!["Git command", "install"],
            ),
            (
                ZervError::CommandFailed("Permission denied accessing git repository".to_string()),
                vec!["Permission denied", "git repository"],
            ),
            (
                ZervError::CommandFailed("No commits found in git repository".to_string()),
                vec!["commits", "git repository"],
            ),
            // Format errors should be descriptive
            (
                ZervError::InvalidFormat("bad format".to_string()),
                vec!["Invalid version format"],
            ),
            (
                ZervError::InvalidVersion("1.0.0-invalid".to_string()),
                vec!["Invalid version"],
            ),
            // Schema errors should be clear
            (
                ZervError::UnknownSchema("unknown".to_string()),
                vec!["Unknown schema"],
            ),
            (
                ZervError::SchemaParseError("bad ron".to_string()),
                vec!["Schema parse error"],
            ),
        ];

        for (error, required_terms) in test_cases {
            let error_msg = error.to_string();
            for term in required_terms {
                assert!(
                    error_msg.contains(term),
                    "Error message should contain '{term}': {error_msg}"
                );
            }
        }
    }

    /// Test that error types are used appropriately (not mixing IO errors with VCS errors)
    #[test]
    fn test_error_type_usage_correctness() {
        // VCS-related errors should use VcsNotFound or CommandFailed, not Io
        let vcs_error =
            ZervError::VcsNotFound("Not in a git repository (--source git)".to_string());
        assert!(matches!(vcs_error, ZervError::VcsNotFound(_)));

        // No tags should use NoTagsFound, not Io
        let no_tags = ZervError::NoTagsFound;
        assert!(matches!(no_tags, ZervError::NoTagsFound));

        // Git command failures should use CommandFailed, not Io
        let cmd_failed = ZervError::CommandFailed("Git command failed".to_string());
        assert!(matches!(cmd_failed, ZervError::CommandFailed(_)));

        // Only actual IO operations should use Io
        let io_error = ZervError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        assert!(matches!(io_error, ZervError::Io(_)));
    }

    /// Test comprehensive error handling for all new error variants
    #[test]
    fn test_new_error_variants_comprehensive() {
        // Test UnknownSource error
        let unknown_source = ZervError::UnknownSource("hg".to_string());
        assert_eq!(unknown_source.to_string(), "Unknown source: hg");

        // Test ConflictingOptions error
        let conflicting =
            ZervError::ConflictingOptions("--clean conflicts with --dirty".to_string());
        assert_eq!(
            conflicting.to_string(),
            "Conflicting options: --clean conflicts with --dirty"
        );

        // Test StdinError error
        let stdin_error = ZervError::StdinError("No input provided via stdin".to_string());
        assert_eq!(
            stdin_error.to_string(),
            "Stdin error: No input provided via stdin"
        );
    }

    /// Test that all error messages provide actionable guidance where appropriate
    #[test]
    fn test_actionable_error_guidance() {
        // Test that VCS errors provide source context
        let vcs_error =
            ZervError::VcsNotFound("Not in a git repository (--source git)".to_string());
        let msg = vcs_error.to_string();
        assert!(
            msg.contains("--source git"),
            "Should indicate which source was attempted"
        );

        // Test that git command errors provide installation guidance
        let git_not_found = ZervError::CommandFailed(
            "Git command not found. Please install git and try again.".to_string(),
        );
        let msg = git_not_found.to_string();
        assert!(
            msg.contains("install git"),
            "Should provide installation guidance"
        );

        // Test that permission errors are clear
        let permission_error =
            ZervError::CommandFailed("Permission denied accessing git repository".to_string());
        let msg = permission_error.to_string();
        assert!(
            msg.contains("Permission denied"),
            "Should clearly indicate permission issue"
        );
        assert!(
            msg.contains("git repository"),
            "Should specify what was being accessed"
        );
    }

    /// Test error message consistency for enhanced git error handling
    #[test]
    fn test_enhanced_git_error_messages() {
        // Test network error messages
        let network_error = ZervError::CommandFailed(
            "Network error accessing git repository. Check your internet connection.".to_string(),
        );
        let msg = network_error.to_string();
        assert!(msg.contains("Network error"));
        assert!(msg.contains("git repository"));
        assert!(msg.contains("Check your internet connection"));

        // Test authentication error messages
        let auth_error = ZervError::CommandFailed(
            "Authentication failed accessing git repository. Check your credentials.".to_string(),
        );
        let msg = auth_error.to_string();
        assert!(msg.contains("Authentication failed"));
        assert!(msg.contains("git repository"));
        assert!(msg.contains("Check your credentials"));

        // Test corruption error messages
        let corruption_error = ZervError::CommandFailed(
            "Git repository appears to be corrupted. Try running 'git fsck' to diagnose."
                .to_string(),
        );
        let msg = corruption_error.to_string();
        assert!(msg.contains("corrupted"));
        assert!(msg.contains("git fsck"));
    }
}
