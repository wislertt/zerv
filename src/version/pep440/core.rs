use super::utils::LocalSegment;
use crate::version::zerv::PreReleaseLabel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostLabel {
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevLabel {
    Dev,
}

impl PreReleaseLabel {
    pub fn as_str(&self) -> &'static str {
        crate::version::pep440::utils::pre_release_label_to_pep440_string(self)
    }
}

impl PostLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PostLabel::Post => "post", // "post", "rev", "r"
        }
    }
}

impl DevLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DevLabel::Dev => "dev",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PEP440 {
    pub epoch: u32,
    pub release: Vec<u32>,
    pub pre_label: Option<PreReleaseLabel>,
    pub pre_number: Option<u32>,
    pub post_label: Option<PostLabel>,
    pub post_number: Option<u32>,
    pub dev_label: Option<DevLabel>,
    pub dev_number: Option<u32>,
    pub local: Option<Vec<LocalSegment>>,
}

impl PEP440 {
    pub fn new(release: Vec<u32>) -> Self {
        Self {
            epoch: 0,
            release,
            pre_label: None,
            pre_number: None,
            post_label: None,
            post_number: None,
            dev_label: None,
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

    pub fn with_post(mut self, post_number: Option<u32>) -> Self {
        self.post_label = Some(PostLabel::Post);
        self.post_number = post_number;
        self
    }

    pub fn with_dev(mut self, dev_number: Option<u32>) -> Self {
        self.dev_label = Some(DevLabel::Dev);
        self.dev_number = dev_number;
        self
    }

    pub fn with_local(mut self, local: &str) -> Self {
        use crate::version::pep440::parser::parse_local_segments;
        self.local = Some(parse_local_segments(local));
        self
    }

    /// Normalize the version by setting implicit 0 values for dev, post, and pre-release numbers
    pub fn normalize(mut self) -> Self {
        self.normalize_implicit_numbers();
        self.normalize_local_segments();
        self
    }

    fn normalize_implicit_numbers(&mut self) {
        if self.pre_label.is_some() && self.pre_number.is_none() {
            self.pre_number = Some(0);
        }
        if self.post_label.is_some() && self.post_number.is_none() {
            self.post_number = Some(0);
        }
        if self.dev_label.is_some() && self.dev_number.is_none() {
            self.dev_number = Some(0);
        }
    }

    fn normalize_local_segments(&mut self) {
        if let Some(ref mut local_segments) = self.local {
            for segment in local_segments {
                Self::normalize_local_segment(segment);
            }
        }
    }

    fn normalize_local_segment(segment: &mut LocalSegment) {
        if let LocalSegment::Str(s) = segment {
            let lowercase = s.to_lowercase();
            if let Ok(num) = lowercase.parse::<u32>() {
                *segment = LocalSegment::UInt(num);
            } else {
                *s = lowercase;
            }
        }
    }
}

impl Default for PEP440 {
    fn default() -> Self {
        Self::new(vec![0, 0, 0])
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_pep440_version_new() {
        let version = PEP440::new(vec![1, 2, 3]);
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
        let version = PEP440::new(vec![1, 2, 3]).with_epoch(2);
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
        let version = PEP440::new(vec![1, 2, 3]).with_pre_release(pre_label.clone(), pre_number);
        assert_eq!(version.pre_label, Some(pre_label));
        assert_eq!(version.pre_number, pre_number);
    }

    #[test]
    fn test_pep440_version_with_post() {
        let version = PEP440::new(vec![1, 2, 3]).with_post(Some(5));
        assert_eq!(version.post_number, Some(5));
    }

    #[test]
    fn test_pep440_version_with_dev() {
        let version = PEP440::new(vec![1, 2, 3]).with_dev(Some(42));
        assert_eq!(version.dev_number, Some(42));
    }

    #[test]
    fn test_pep440_version_with_local() {
        let version = PEP440::new(vec![1, 2, 3]).with_local("ubuntu.20.04");
        let expected = vec![
            LocalSegment::Str("ubuntu".to_string()),
            LocalSegment::UInt(20),
            LocalSegment::UInt(4), // "04" becomes integer 4
        ];
        assert_eq!(version.local, Some(expected));
    }

    #[test]
    fn test_pre_release_label_as_str() {
        assert_eq!(PreReleaseLabel::Alpha.as_str(), "a");
        assert_eq!(PreReleaseLabel::Beta.as_str(), "b");
        assert_eq!(PreReleaseLabel::Rc.as_str(), "rc");
    }

    #[test]
    fn test_post_release_label_as_str() {
        assert_eq!(PostLabel::Post.as_str(), "post");
    }

    #[test]
    fn test_pep440_version_default() {
        let version = PEP440::default();
        assert_eq!(version.epoch, 0);
        assert_eq!(version.release, vec![0, 0, 0]);
        assert!(version.pre_label.is_none());
    }

    #[test]
    fn test_pep440_version_with_pre_release_none_number() {
        let version = PEP440::new(vec![1, 2, 3]).with_pre_release(PreReleaseLabel::Alpha, None);
        assert_eq!(version.pre_label, Some(PreReleaseLabel::Alpha));
        assert_eq!(version.pre_number, None);
    }

    #[test]
    fn test_pep440_version_with_dev_none() {
        let version = PEP440::new(vec![1, 2, 3]).with_dev(None);
        assert_eq!(version.dev_number, None);
    }

    #[test]
    fn test_pep440_version_with_post_none() {
        let version = PEP440::new(vec![1, 2, 3]).with_post(None);
        assert_eq!(version.post_number, None);
        assert_eq!(version.post_label, Some(PostLabel::Post));
    }

    #[test]
    fn test_complex_pep440_version() {
        let version = PEP440::new(vec![1, 2, 3])
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(Some(1))
            .with_dev(Some(1))
            .with_local("local.meta");

        assert_eq!(version.epoch, 2);
        assert_eq!(version.release, vec![1, 2, 3]);
        assert_eq!(version.pre_label, Some(PreReleaseLabel::Alpha));
        assert_eq!(version.pre_number, Some(1));
        assert_eq!(version.post_number, Some(1));
        assert_eq!(version.dev_number, Some(1));
        assert_eq!(
            version.local,
            Some(vec![
                LocalSegment::Str("local".to_string()),
                LocalSegment::Str("meta".to_string()),
            ])
        );
    }

    // Edge case tests
    #[test]
    fn test_empty_release_vector() {
        let version = PEP440::new(vec![]);
        assert_eq!(version.release, Vec::<u32>::new());
    }

    #[test]
    fn test_single_element_release() {
        let version = PEP440::new(vec![1]);
        assert_eq!(version.release, vec![1]);
    }

    #[test]
    fn test_max_values() {
        let version = PEP440::new(vec![u32::MAX])
            .with_epoch(u32::MAX)
            .with_pre_release(PreReleaseLabel::Alpha, Some(u32::MAX))
            .with_post(Some(u32::MAX))
            .with_dev(Some(u32::MAX));

        assert_eq!(version.epoch, u32::MAX);
        assert_eq!(version.release, vec![u32::MAX]);
        assert_eq!(version.pre_number, Some(u32::MAX));
        assert_eq!(version.post_number, Some(u32::MAX));
        assert_eq!(version.dev_number, Some(u32::MAX));
    }

    #[test]
    fn test_zero_values() {
        let version = PEP440::new(vec![0])
            .with_epoch(0)
            .with_pre_release(PreReleaseLabel::Alpha, Some(0))
            .with_post(Some(0))
            .with_dev(Some(0));

        assert_eq!(version.epoch, 0);
        assert_eq!(version.pre_number, Some(0));
        assert_eq!(version.post_number, Some(0));
        assert_eq!(version.dev_number, Some(0));
    }

    #[test]
    fn test_method_chaining_overwrite() {
        let version = PEP440::new(vec![1])
            .with_epoch(1)
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_pre_release(PreReleaseLabel::Beta, Some(2));

        assert_eq!(version.epoch, 2);
        assert_eq!(version.pre_label, Some(PreReleaseLabel::Beta));
        assert_eq!(version.pre_number, Some(2));
    }

    #[rstest]
    #[case(PreReleaseLabel::Alpha)]
    #[case(PreReleaseLabel::Beta)]
    #[case(PreReleaseLabel::Rc)]
    fn test_implicit_vs_explicit_zero_equality(#[case] pre_label: PreReleaseLabel) {
        let base = || PEP440::new(vec![1, 0, 0]);

        // Individual components
        assert_eq!(
            base().with_pre_release(pre_label.clone(), None),
            base().with_pre_release(pre_label.clone(), Some(0))
        );
        assert_eq!(base().with_post(None), base().with_post(Some(0)));
        assert_eq!(base().with_dev(None), base().with_dev(Some(0)));

        // 2-component combinations
        assert_eq!(
            base()
                .with_pre_release(pre_label.clone(), None)
                .with_post(None),
            base()
                .with_pre_release(pre_label.clone(), Some(0))
                .with_post(Some(0))
        );
        assert_eq!(
            base().with_post(None).with_dev(None),
            base().with_post(Some(0)).with_dev(Some(0))
        );
        assert_eq!(
            base()
                .with_pre_release(pre_label.clone(), None)
                .with_dev(None),
            base()
                .with_pre_release(pre_label.clone(), Some(0))
                .with_dev(Some(0))
        );

        // All combinations
        assert_eq!(
            base()
                .with_pre_release(pre_label.clone(), None)
                .with_post(None)
                .with_dev(None),
            base()
                .with_pre_release(pre_label.clone(), Some(0))
                .with_post(Some(0))
                .with_dev(Some(0))
        );
    }

    #[test]
    fn test_post_none_after_some() {
        let version = PEP440::new(vec![1]).with_post(Some(1)).with_post(None);

        assert_eq!(version.post_number, None);
        assert_eq!(version.post_label, Some(PostLabel::Post));
    }

    #[test]
    fn test_local_empty_string() {
        let version = PEP440::new(vec![1]).with_local("");
        assert!(version.local.is_some());
    }

    #[test]
    fn test_normalize_pre_release() {
        let version = PEP440::new(vec![1, 2, 3]).with_pre_release(PreReleaseLabel::Alpha, None);
        let normalized = version.normalize();
        assert_eq!(normalized.pre_number, Some(0));
    }

    #[test]
    fn test_normalize_post_release() {
        let version = PEP440::new(vec![1, 2, 3]).with_post(None);
        let normalized = version.normalize();
        assert_eq!(normalized.post_number, Some(0));
    }

    #[test]
    fn test_normalize_dev_release() {
        let version = PEP440::new(vec![1, 2, 3]).with_dev(None);
        let normalized = version.normalize();
        assert_eq!(normalized.dev_number, Some(0));
    }

    #[test]
    fn test_normalize_all_implicit() {
        let version = PEP440::new(vec![1, 2, 3])
            .with_pre_release(PreReleaseLabel::Beta, None)
            .with_post(None)
            .with_dev(None);
        let normalized = version.normalize();
        assert_eq!(normalized.pre_number, Some(0));
        assert_eq!(normalized.post_number, Some(0));
        assert_eq!(normalized.dev_number, Some(0));
    }

    #[test]
    fn test_normalize_preserves_explicit_values() {
        let version = PEP440::new(vec![1, 2, 3])
            .with_pre_release(PreReleaseLabel::Alpha, Some(5))
            .with_post(Some(3))
            .with_dev(Some(7));
        let normalized = version.normalize();
        assert_eq!(normalized.pre_number, Some(5));
        assert_eq!(normalized.post_number, Some(3));
        assert_eq!(normalized.dev_number, Some(7));
    }

    #[test]
    fn test_normalize_no_labels() {
        let version = PEP440::new(vec![1, 2, 3]);
        let normalized = version.normalize();
        assert_eq!(normalized.pre_number, None);
        assert_eq!(normalized.post_number, None);
        assert_eq!(normalized.dev_number, None);
    }

    #[test]
    fn test_long_release_vector() {
        let long_release = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let version = PEP440::new(long_release.clone());
        assert_eq!(version.release, long_release);
    }

    // Round-trip tests for normalized strings (should get same string back)
    #[rstest]
    // Different release formats
    #[case("1")]
    #[case("1.2")]
    #[case("1.2.3")]
    #[case("1.2.3.4")]
    //
    #[case("1.0")]
    #[case("1.2.0")]
    #[case("1.2.3.0")]
    // Prerelease variations
    #[case("1.0.0a1")]
    #[case("1.0.0b2")]
    #[case("1.0.0rc3")]
    #[case("1.0.0a0")]
    #[case("1.0.0b0")]
    #[case("1.0.0rc0")]
    #[case("1a1")]
    #[case("1.2a0")]
    // Post release variations
    #[case("1.0.0.post4")]
    #[case("1.0.0.post0")]
    #[case("1.post1")]
    #[case("1.2.post0")]
    // Dev release variations
    #[case("1.0.0.dev5")]
    #[case("1.0.0.dev0")]
    #[case("1.dev1")]
    #[case("1.2.dev0")]
    // Local versions
    #[case("1.0.0+local")]
    #[case("1+build")]
    #[case("1.2+local.123")]
    // Epoch versions
    #[case("5!1.2.3")]
    #[case("2!1")]
    #[case("1!1.2")]
    // Combinations with different release formats
    #[case("1a1.post2")]
    #[case("1.2b3.dev4")]
    #[case("1.post1.dev2")]
    #[case("1a0.post0")]
    #[case("1.2rc0.dev0")]
    #[case("1.post0.dev1")]
    #[case("1a1.post2.dev3")]
    #[case("1.2b0.post0.dev4")]
    // Complex combinations
    #[case("1.0.0a1.post2.dev3+build.123")]
    #[case("2!1.2a0.post0.dev0+local")]
    #[case("1!1a1.dev2+build.456")]
    fn test_roundtrip_normalized_strings(#[case] input: &str) {
        let parsed: PEP440 = input.parse().unwrap();
        let output = parsed.to_string();
        assert_eq!(input, output, "Normalized string should remain unchanged");
    }

    // Round-trip tests for unnormalized strings (should get expected normalized version)
    #[rstest]
    // Prerelease normalization with different release formats
    #[case("1alpha1", "1a1")]
    #[case("1.2ALPHA1", "1.2a1")]
    #[case("1.0.0alpha", "1.0.0a0")]
    #[case("1ALPHA", "1a0")]
    #[case("1.2beta2", "1.2b2")]
    #[case("1.0.0BETA2", "1.0.0b2")]
    #[case("1beta", "1b0")]
    #[case("1.2BETA", "1.2b0")]
    #[case("1c3", "1rc3")]
    #[case("1.2c", "1.2rc0")]
    #[case("1.0.0preview4", "1.0.0rc4")]
    #[case("1preview", "1rc0")]
    #[case("1.2pre5", "1.2rc5")]
    #[case("1pre", "1rc0")]
    #[case("1-a1", "1a1")]
    #[case("1.2-a", "1.2a0")]
    #[case("1_beta2", "1b2")]
    #[case("1.2_beta", "1.2b0")]
    #[case("1.0.0.rc3", "1.0.0rc3")]
    #[case("1.rc", "1rc0")]
    // Post release normalization with different release formats
    #[case("1-1", "1.post1")]
    #[case("1.2-2", "1.2.post2")]
    #[case("1.0.0.rev2", "1.0.0.post2")]
    #[case("1rev", "1.post0")]
    #[case("1.2.r3", "1.2.post3")]
    #[case("1r", "1.post0")]
    // Dev release normalization with different release formats
    #[case("1-dev1", "1.dev1")]
    #[case("1.2-dev", "1.2.dev0")]
    #[case("1_dev2", "1.dev2")]
    #[case("1.0.0_dev", "1.0.0.dev0")]
    // Local version normalization
    #[case("1+ubuntu-20-04", "1+ubuntu.20.4")]
    #[case("1.2+ubuntu_20_04", "1.2+ubuntu.20.4")]
    // Complex combinations with different release formats
    #[case("1alpha1-1-dev2", "1a1.post1.dev2")]
    #[case("1.2beta-rev-dev", "1.2b0.post0.dev0")]
    #[case("1c1.r2_dev3", "1rc1.post2.dev3")]
    #[case("1.2pre.post1-dev", "1.2rc0.post1.dev0")]
    // Mixed separators and formats
    #[case("1_alpha_1.post_2-dev_3", "1a1.post2.dev3")]
    #[case("1.2-beta-2_r_3.dev.4", "1.2b2.post3.dev4")]
    #[case("1.0.0a1.post2.dev3+BUILD.123", "1.0.0a1.post2.dev3+build.123")]
    #[case("1.0.0a1.post2.dev3+build-123", "1.0.0a1.post2.dev3+build.123")]
    #[case("1.0.0a1.post2.dev3+build_123", "1.0.0a1.post2.dev3+build.123")]
    #[case("1.2.3a.post.dev", "1.2.3a0.post0.dev0")]
    #[case("1.2.3a.rev.dev", "1.2.3a0.post0.dev0")]
    #[case("1.2.3a.r.dev", "1.2.3a0.post0.dev0")]
    #[case("1.2.3c.post.dev", "1.2.3rc0.post0.dev0")]
    #[case("1.2.3pre.post.dev", "1.2.3rc0.post0.dev0")]
    #[case("1.2.3preview.post.dev", "1.2.3rc0.post0.dev0")]
    fn test_roundtrip_unnormalized_strings(#[case] input: &str, #[case] expected: &str) {
        let parsed: PEP440 = input.parse().unwrap();
        let normalized = parsed;
        let output = normalized.to_string();
        assert_eq!(
            expected, output,
            "Unnormalized string should be normalized to expected form"
        );

        // Also verify the normalized version parses to the same object
        let reparsed: PEP440 = output.parse().unwrap();
        assert_eq!(
            normalized, reparsed,
            "Normalized version should parse to same object"
        );
    }
}
