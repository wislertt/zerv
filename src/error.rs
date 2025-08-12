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
    /// Command execution failed
    CommandFailed(String),
    /// IO error
    Io(io::Error),
    /// Regex error
    Regex(String),
}

impl std::fmt::Display for ZervError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZervError::VcsNotFound(vcs) => write!(f, "VCS not found: {vcs}"),
            ZervError::NoTagsFound => write!(f, "No tags found matching pattern"),
            ZervError::InvalidFormat(msg) => write!(f, "Invalid version format: {msg}"),
            ZervError::CommandFailed(msg) => write!(f, "Command execution failed: {msg}"),
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

impl PartialEq for ZervError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ZervError::VcsNotFound(a), ZervError::VcsNotFound(b)) => a == b,
            (ZervError::NoTagsFound, ZervError::NoTagsFound) => true,
            (ZervError::InvalidFormat(a), ZervError::InvalidFormat(b)) => a == b,
            (ZervError::CommandFailed(a), ZervError::CommandFailed(b)) => a == b,
            (ZervError::Io(a), ZervError::Io(b)) => {
                a.kind() == b.kind() && a.to_string() == b.to_string()
            }
            (ZervError::Regex(a), ZervError::Regex(b)) => a == b,
            _ => false,
        }
    }
}

/// Result type alias for zerv operations
pub type Result<T> = std::result::Result<T, ZervError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_error_display() {
        assert_eq!(
            ZervError::VcsNotFound("git".to_string()).to_string(),
            "VCS not found: git"
        );
        assert_eq!(
            ZervError::NoTagsFound.to_string(),
            "No tags found matching pattern"
        );
        assert_eq!(
            ZervError::InvalidFormat("bad".to_string()).to_string(),
            "Invalid version format: bad"
        );
        assert_eq!(
            ZervError::CommandFailed("exit 1".to_string()).to_string(),
            "Command execution failed: exit 1"
        );
        assert_eq!(
            ZervError::Regex("invalid".to_string()).to_string(),
            "Regex error: invalid"
        );
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let zerv_err: ZervError = io_err.into();
        assert!(matches!(zerv_err, ZervError::Io(_)));
        assert!(zerv_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_error_source() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let zerv_err = ZervError::Io(io_err);
        assert!(zerv_err.source().is_some());

        assert!(ZervError::NoTagsFound.source().is_none());
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<i32> = Ok(42);
        let err_result: Result<i32> = Err(ZervError::NoTagsFound);

        assert_eq!(ok_result, Ok(42));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_error_equality() {
        // Same variants with same values should be equal
        assert_eq!(
            ZervError::VcsNotFound("git".to_string()),
            ZervError::VcsNotFound("git".to_string())
        );
        assert_eq!(ZervError::NoTagsFound, ZervError::NoTagsFound);
        assert_eq!(
            ZervError::InvalidFormat("bad".to_string()),
            ZervError::InvalidFormat("bad".to_string())
        );
        assert_eq!(
            ZervError::CommandFailed("exit 1".to_string()),
            ZervError::CommandFailed("exit 1".to_string())
        );
        assert_eq!(
            ZervError::Regex("invalid".to_string()),
            ZervError::Regex("invalid".to_string())
        );

        // Same variants with different values should not be equal
        assert_ne!(
            ZervError::VcsNotFound("git".to_string()),
            ZervError::VcsNotFound("hg".to_string())
        );
        assert_ne!(
            ZervError::InvalidFormat("bad".to_string()),
            ZervError::InvalidFormat("worse".to_string())
        );
        assert_ne!(
            ZervError::CommandFailed("exit 1".to_string()),
            ZervError::CommandFailed("exit 2".to_string())
        );
        assert_ne!(
            ZervError::Regex("invalid".to_string()),
            ZervError::Regex("bad".to_string())
        );

        // Different variants should not be equal
        assert_ne!(
            ZervError::NoTagsFound,
            ZervError::VcsNotFound("git".to_string())
        );
        assert_ne!(
            ZervError::InvalidFormat("bad".to_string()),
            ZervError::CommandFailed("bad".to_string())
        );

        // IO errors with same kind and message should be equal
        let io_err1 = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let io_err2 = io::Error::new(io::ErrorKind::NotFound, "file not found");
        assert_eq!(ZervError::Io(io_err1), ZervError::Io(io_err2));

        // IO errors with different kinds should not be equal
        let io_err3 = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let io_err4 = io::Error::new(io::ErrorKind::PermissionDenied, "file not found");
        assert_ne!(ZervError::Io(io_err3), ZervError::Io(io_err4));
    }
}
