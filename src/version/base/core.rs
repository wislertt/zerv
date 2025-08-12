use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum Stage {
    Alpha,
    Beta,
    Rc,
}

#[derive(Debug, Clone)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub stage: Option<Stage>,
    pub revision: Option<u32>,
    pub post: Option<u32>,
    pub dev: Option<u32>,
    pub distance: u32,
    pub commit: Option<String>,
    pub dirty: bool,
    pub tagged_metadata: Option<String>,
    pub epoch: Option<u32>,
    pub branch: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            stage: None,
            revision: None,
            post: None,
            dev: None,
            distance: 0,
            commit: None,
            dirty: false,
            tagged_metadata: None,
            epoch: None,
            branch: None,
            timestamp: None,
        }
    }

    pub fn with_stage(mut self, stage: Stage, revision: Option<u32>) -> Self {
        self.stage = Some(stage);
        self.revision = revision;
        self
    }

    pub fn with_post(mut self, post: u32) -> Self {
        self.post = Some(post);
        self
    }

    pub fn with_dev(mut self, dev: u32) -> Self {
        self.dev = Some(dev);
        self
    }

    pub fn with_distance(mut self, distance: u32) -> Self {
        self.distance = distance;
        self
    }

    pub fn with_commit(mut self, commit: &str) -> Self {
        self.commit = Some(commit.to_string());
        self
    }

    pub fn with_dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub fn with_branch(mut self, branch: &str) -> Self {
        self.branch = Some(branch.to_string());
        self
    }

    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn parse(input: &str) -> Result<Self, crate::error::ZervError> {
        crate::version::parser::parse_version(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(1, 2, 3)]
    #[case(0, 0, 0)]
    #[case(10, 20, 30)]
    fn test_version_new(#[case] major: u32, #[case] minor: u32, #[case] patch: u32) {
        let version = Version::new(major, minor, patch);
        assert_eq!(version.major, major);
        assert_eq!(version.minor, minor);
        assert_eq!(version.patch, patch);
        assert_eq!(version.distance, 0);
        assert!(!version.dirty);
        assert!(version.stage.is_none());
    }

    #[rstest]
    #[case(Stage::Alpha, Some(1))]
    #[case(Stage::Beta, Some(2))]
    #[case(Stage::Rc, None)]
    fn test_version_with_stage(#[case] stage: Stage, #[case] revision: Option<u32>) {
        let version = Version::new(1, 2, 3).with_stage(stage.clone(), revision);

        assert_eq!(version.stage, Some(stage));
        assert_eq!(version.revision, revision);
    }

    #[rstest]
    #[case(0)]
    #[case(5)]
    #[case(100)]
    fn test_version_with_distance(#[case] distance: u32) {
        let version = Version::new(1, 2, 3).with_distance(distance);
        assert_eq!(version.distance, distance);
    }

    #[rstest]
    #[case("abc123")]
    #[case("g29045e8")]
    #[case("1234567890abcdef")]
    fn test_version_with_commit(#[case] commit: &str) {
        let version = Version::new(1, 2, 3).with_commit(commit);
        assert_eq!(version.commit, Some(commit.to_string()));
    }

    #[rstest]
    #[case(true)]
    #[case(false)]
    fn test_version_with_dirty(#[case] dirty: bool) {
        let version = Version::new(1, 2, 3).with_dirty(dirty);
        assert_eq!(version.dirty, dirty);
    }

    #[rstest]
    #[case(0)]
    #[case(5)]
    #[case(42)]
    fn test_version_with_post(#[case] post: u32) {
        let version = Version::new(1, 2, 3).with_post(post);
        assert_eq!(version.post, Some(post));
    }

    #[rstest]
    #[case(0)]
    #[case(10)]
    #[case(999)]
    fn test_version_with_dev(#[case] dev: u32) {
        let version = Version::new(1, 2, 3).with_dev(dev);
        assert_eq!(version.dev, Some(dev));
    }

    #[test]
    fn test_version_with_timestamp() {
        use chrono::TimeZone;
        let timestamp = Utc.with_ymd_and_hms(2023, 12, 15, 14, 25, 30).unwrap();
        let version = Version::new(1, 2, 3).with_timestamp(timestamp);
        assert_eq!(version.timestamp, Some(timestamp));
    }

    #[test]
    fn test_version_with_branch() {
        let version = Version::new(1, 2, 3).with_branch("feature/auth");
        assert_eq!(version.branch, Some("feature/auth".to_string()));
    }

    #[test]
    fn test_version_default() {
        let version = Version::default();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
        assert_eq!(version.distance, 0);
        assert!(!version.dirty);
        assert!(version.stage.is_none());
        assert!(version.post.is_none());
        assert!(version.dev.is_none());
    }

    #[test]
    fn test_version_parse() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_parse_invalid() {
        let result = Version::parse("invalid");
        assert!(result.is_err());
    }
}
