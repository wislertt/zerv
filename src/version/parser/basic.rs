use crate::error::ZervError;
use crate::version::Version;

pub fn parse_version(input: &str) -> Result<Version, ZervError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(ZervError::InvalidVersion(
            "Empty version string".to_string(),
        ));
    }

    // Parse basic semver: "1.2.3"
    let parts: Vec<&str> = input.split('.').collect();

    if parts.len() != 3 {
        return Err(ZervError::InvalidVersion(format!(
            "Expected format 'major.minor.patch', got '{input}'"
        )));
    }

    let major = parts[0]
        .parse::<u32>()
        .map_err(|_| ZervError::InvalidVersion(format!("Invalid major version: '{}'", parts[0])))?;

    let minor = parts[1]
        .parse::<u32>()
        .map_err(|_| ZervError::InvalidVersion(format!("Invalid minor version: '{}'", parts[1])))?;

    let patch = parts[2]
        .parse::<u32>()
        .map_err(|_| ZervError::InvalidVersion(format!("Invalid patch version: '{}'", parts[2])))?;

    Ok(Version::new(major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_version() {
        let version = parse_version("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_parse_zero_version() {
        let version = parse_version("0.0.0").unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_parse_large_numbers() {
        let version = parse_version("10.20.30").unwrap();
        assert_eq!(version.major, 10);
        assert_eq!(version.minor, 20);
        assert_eq!(version.patch, 30);
    }

    #[test]
    fn test_parse_empty_string() {
        let result = parse_version("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = parse_version("1.2");
        assert!(result.is_err());

        let result = parse_version("1.2.3.4");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_numbers() {
        let result = parse_version("a.2.3");
        assert!(result.is_err());

        let result = parse_version("1.b.3");
        assert!(result.is_err());

        let result = parse_version("1.2.c");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_whitespace() {
        let version = parse_version("  1.2.3  ").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }
}
