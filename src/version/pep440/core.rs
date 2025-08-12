#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreReleaseLabel {
    Alpha,
    Beta,
    Rc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalSegment {
    String(String),
    Integer(u32),
}

impl PreReleaseLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PreReleaseLabel::Alpha => "a", // "alpha", "a"
            PreReleaseLabel::Beta => "b",  // "beta", "b"
            PreReleaseLabel::Rc => "rc",   // "rc", "c", "preview", "pre"
        }
    }
}

#[derive(Debug, Clone)]
pub struct PEP440Version {
    pub epoch: u32,
    pub release: Vec<u32>,
    pub pre_label: Option<PreReleaseLabel>,
    pub pre_number: Option<u32>,
    pub post_label: &'static str,
    pub post_number: Option<u32>,
    pub dev_number: Option<u32>,
    pub local: Option<Vec<LocalSegment>>,
}

impl PEP440Version {
    pub fn new(release: Vec<u32>) -> Self {
        Self {
            epoch: 0,
            release,
            pre_label: None,
            pre_number: None,
            post_label: "post",
            post_number: None,
            dev_number: None,
            local: None,
        }
    }

    pub fn with_epoch(mut self, epoch: u32) -> Self {
        self.epoch = epoch;
        self
    }

    pub fn with_pre_release(mut self, pre_label: PreReleaseLabel, pre_number: Option<u32>) -> Self {
        self.pre_label = Some(pre_label);
        self.pre_number = pre_number;
        self
    }

    pub fn with_post(mut self, post_number: u32) -> Self {
        self.post_number = Some(post_number);
        self
    }

    pub fn with_dev(mut self, dev_number: u32) -> Self {
        self.dev_number = Some(dev_number);
        self
    }

    pub fn with_local(mut self, local: Vec<LocalSegment>) -> Self {
        self.local = Some(local);
        self
    }
}

impl Default for PEP440Version {
    fn default() -> Self {
        Self::new(vec![0, 0, 0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_pep440_version_new() {
        let version = PEP440Version::new(vec![1, 2, 3]);
        assert_eq!(version.epoch, 0);
        assert_eq!(version.release, vec![1, 2, 3]);
        assert!(version.pre_label.is_none());
        assert!(version.pre_number.is_none());
        assert!(version.post_number.is_none());
        assert!(version.dev_number.is_none());
        assert!(version.local.is_none());
    }

    #[test]
    fn test_pep440_version_with_epoch() {
        let version = PEP440Version::new(vec![1, 2, 3]).with_epoch(2);
        assert_eq!(version.epoch, 2);
    }

    #[rstest]
    #[case(PreReleaseLabel::Alpha, Some(1))]
    #[case(PreReleaseLabel::Beta, Some(2))]
    #[case(PreReleaseLabel::Rc, None)]
    fn test_pep440_version_with_pre_release(
        #[case] pre_label: PreReleaseLabel,
        #[case] pre_number: Option<u32>,
    ) {
        let version =
            PEP440Version::new(vec![1, 2, 3]).with_pre_release(pre_label.clone(), pre_number);
        assert_eq!(version.pre_label, Some(pre_label));
        assert_eq!(version.pre_number, pre_number);
    }

    #[test]
    fn test_pep440_version_with_post() {
        let version = PEP440Version::new(vec![1, 2, 3]).with_post(5);
        assert_eq!(version.post_number, Some(5));
    }

    #[test]
    fn test_pep440_version_with_dev() {
        let version = PEP440Version::new(vec![1, 2, 3]).with_dev(42);
        assert_eq!(version.dev_number, Some(42));
    }

    #[test]
    fn test_pep440_version_with_local() {
        let local = vec![
            LocalSegment::String("ubuntu".to_string()),
            LocalSegment::Integer(20),
            LocalSegment::String("04".to_string()),
        ];
        let version = PEP440Version::new(vec![1, 2, 3]).with_local(local.clone());
        assert_eq!(version.local, Some(local));
    }

    #[test]
    fn test_pre_release_label_as_str() {
        assert_eq!(PreReleaseLabel::Alpha.as_str(), "a");
        assert_eq!(PreReleaseLabel::Beta.as_str(), "b");
        assert_eq!(PreReleaseLabel::Rc.as_str(), "rc");
    }

    #[test]
    fn test_pep440_version_default() {
        let version = PEP440Version::default();
        assert_eq!(version.epoch, 0);
        assert_eq!(version.release, vec![0, 0, 0]);
        assert!(version.pre_label.is_none());
    }

    #[test]
    fn test_complex_pep440_version() {
        let version = PEP440Version::new(vec![1, 2, 3])
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(1)
            .with_dev(1)
            .with_local(vec![
                LocalSegment::String("local".to_string()),
                LocalSegment::String("meta".to_string()),
            ]);

        assert_eq!(version.epoch, 2);
        assert_eq!(version.release, vec![1, 2, 3]);
        assert_eq!(version.pre_label, Some(PreReleaseLabel::Alpha));
        assert_eq!(version.pre_number, Some(1));
        assert_eq!(version.post_number, Some(1));
        assert_eq!(version.dev_number, Some(1));
        assert_eq!(
            version.local,
            Some(vec![
                LocalSegment::String("local".to_string()),
                LocalSegment::String("meta".to_string()),
            ])
        );
    }
}
