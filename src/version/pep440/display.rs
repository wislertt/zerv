use super::core::{LocalSegment, PEP440Version};
use std::fmt;

impl fmt::Display for PEP440Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Epoch
        if self.epoch > 0 {
            write!(f, "{}!", self.epoch)?;
        }

        // Release
        let release_str = self
            .release
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(".");
        write!(f, "{release_str}")?;

        // Pre-release
        if let Some(ref pre_label) = self.pre_label {
            write!(f, "{}", pre_label.as_str())?;
            if let Some(pre_number) = self.pre_number {
                write!(f, "{pre_number}")?;
            }
        }

        // Post-release
        if let Some(ref post_label) = self.post_label
            && let Some(post_number) = self.post_number
        {
            write!(f, ".{}{}", post_label.as_str(), post_number)?;
        }

        // Dev-release
        if let Some(dev_number) = self.dev_number {
            write!(f, ".dev{dev_number}")?;
        }

        // Local version
        if let Some(ref local) = self.local {
            write!(f, "+")?;
            let local_str = local
                .iter()
                .map(|segment| match segment {
                    LocalSegment::String(s) => s.clone(),
                    LocalSegment::Integer(i) => i.to_string(),
                })
                .collect::<Vec<_>>()
                .join(".");
            write!(f, "{local_str}")?;
        }

        Ok(())
    }
}

impl fmt::Display for LocalSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalSegment::String(s) => write!(f, "{s}"),
            LocalSegment::Integer(i) => write!(f, "{i}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::pep440::core::PreReleaseLabel;

    #[test]
    fn test_display_simple_version() {
        let version = PEP440Version::new(vec![1, 2, 3]);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_display_complex_version() {
        let version = PEP440Version::new(vec![2025, 12, 31])
            .with_epoch(42)
            .with_pre_release(PreReleaseLabel::Alpha, Some(99))
            .with_post(123)
            .with_dev(456)
            .with_local("deadbeef.abc123");
        assert_eq!(
            version.to_string(),
            "42!2025.12.31a99.post123.dev456+deadbeef.abc123"
        );
    }

    #[test]
    fn test_display_with_local_mixed() {
        let version = PEP440Version::new(vec![1, 0, 0]).with_local("ubuntu.20.04");
        assert_eq!(version.to_string(), "1.0.0+ubuntu.20.4"); // "04" becomes "4"
    }

    #[test]
    fn test_display_pre_release_only() {
        let version =
            PEP440Version::new(vec![1, 2, 3]).with_pre_release(PreReleaseLabel::Beta, Some(2));
        assert_eq!(version.to_string(), "1.2.3b2");
    }

    #[test]
    fn test_display_local_segment() {
        assert_eq!(LocalSegment::String("test".to_string()).to_string(), "test");
        assert_eq!(LocalSegment::Integer(42).to_string(), "42");
    }
}
