use std::io;

/// Main error type for the zerv library
#[derive(Debug)]
pub enum ZervError {
    /// VCS not found or not available
    VcsNotFound(String),
    /// No tags found matching pattern
    NoTagsFound,
    /// Invalid version format
    InvalidFormat(String),
    /// Invalid version string
    InvalidVersion(String),
    /// Command execution failed
    CommandFailed(String),
    /// IO error
    Io(io::Error),
    /// Regex error
    Regex(String),
    /// Schema parsing error
    SchemaParseError(String),
    /// Unknown schema name
    UnknownSchema(String),
    /// Conflicting schema parameters
    ConflictingSchemas(String),
}

impl std::fmt::Display for ZervError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZervError::VcsNotFound(vcs) => write!(f, "VCS not found: {vcs}"),
            ZervError::NoTagsFound => write!(f, "No tags found matching pattern"),
            ZervError::InvalidFormat(msg) => write!(f, "Invalid version format: {msg}"),
            ZervError::InvalidVersion(msg) => write!(f, "Invalid version: {msg}"),
            ZervError::CommandFailed(msg) => write!(f, "Command execution failed: {msg}"),
            ZervError::Io(err) => write!(f, "IO error: {err}"),
            ZervError::Regex(msg) => write!(f, "Regex error: {msg}"),
            ZervError::SchemaParseError(msg) => write!(f, "Schema parse error: {msg}"),
            ZervError::UnknownSchema(name) => write!(f, "Unknown schema: {name}"),
            ZervError::ConflictingSchemas(msg) => write!(f, "Conflicting schemas: {msg}"),
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
    #[case(ZervError::NoTagsFound, "No tags found matching pattern")]
    #[case(ZervError::InvalidFormat("bad".to_string()), "Invalid version format: bad")]
    #[case(ZervError::InvalidVersion("1.0.0-invalid".to_string()), "Invalid version: 1.0.0-invalid")]
    #[case(ZervError::CommandFailed("exit 1".to_string()), "Command execution failed: exit 1")]
    #[case(ZervError::Regex("invalid".to_string()), "Regex error: invalid")]
    #[case(ZervError::SchemaParseError("bad ron".to_string()), "Schema parse error: bad ron")]
    #[case(ZervError::UnknownSchema("unknown".to_string()), "Unknown schema: unknown")]
    #[case(ZervError::ConflictingSchemas("both provided".to_string()), "Conflicting schemas: both provided")]
    fn test_error_display(#[case] error: ZervError, #[case] expected: &str) {
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

    #[test]
    fn test_debug_trait() {
        let error = ZervError::VcsNotFound("git".to_string());
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("VcsNotFound"));
        assert!(debug_str.contains("git"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = ZervError::NoTagsFound;
        let _: &dyn Error = &error; // Ensure Error trait is implemented
    }
}
