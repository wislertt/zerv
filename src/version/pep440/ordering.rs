use super::core::{LocalSegment, PEP440Version, PreReleaseLabel};
use std::cmp::Ordering;

impl PartialOrd for PEP440Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PEP440Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare epoch first
        match self.epoch.cmp(&other.epoch) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare release versions
        match self.release.cmp(&other.release) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare pre-release (None > Some for pre-release)
        match (&self.pre_label, &other.pre_label) {
            (None, None) => {}
            (None, Some(_)) => return Ordering::Greater,
            (Some(_), None) => return Ordering::Less,
            (Some(self_pre), Some(other_pre)) => match self_pre.cmp(other_pre) {
                Ordering::Equal => match self.pre_number.cmp(&other.pre_number) {
                    Ordering::Equal => {}
                    other => return other,
                },
                other => return other,
            },
        }

        // Compare post-release (None < Some for post-release)
        match self.post_number.cmp(&other.post_number) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare dev-release (None > Some for dev-release)
        match (&self.dev_number, &other.dev_number) {
            (None, None) => {}
            (None, Some(_)) => return Ordering::Greater,
            (Some(_), None) => return Ordering::Less,
            (Some(self_dev), Some(other_dev)) => match self_dev.cmp(other_dev) {
                Ordering::Equal => {}
                other => return other,
            },
        }

        // Local versions are not compared in PEP 440
        Ordering::Equal
    }
}

impl PartialOrd for PreReleaseLabel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreReleaseLabel {
    fn cmp(&self, other: &Self) -> Ordering {
        use PreReleaseLabel::*;
        match (self, other) {
            (Alpha, Alpha) | (Beta, Beta) | (Rc, Rc) => Ordering::Equal,
            (Alpha, Beta) | (Alpha, Rc) => Ordering::Less,
            (Beta, Alpha) => Ordering::Greater,
            (Beta, Rc) => Ordering::Less,
            (Rc, Alpha) | (Rc, Beta) => Ordering::Greater,
        }
    }
}

impl PartialEq for PEP440Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for LocalSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LocalSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (LocalSegment::Integer(a), LocalSegment::Integer(b)) => a.cmp(b),
            (LocalSegment::String(a), LocalSegment::String(b)) => a.cmp(b),
            (LocalSegment::Integer(_), LocalSegment::String(_)) => Ordering::Less,
            (LocalSegment::String(_), LocalSegment::Integer(_)) => Ordering::Greater,
        }
    }
}

impl Eq for PEP440Version {}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(PreReleaseLabel::Alpha, PreReleaseLabel::Beta)]
    #[case(PreReleaseLabel::Alpha, PreReleaseLabel::Rc)]
    #[case(PreReleaseLabel::Beta, PreReleaseLabel::Rc)]
    fn test_pre_release_label_less_than(
        #[case] left: PreReleaseLabel,
        #[case] right: PreReleaseLabel,
    ) {
        assert!(left < right);
    }

    #[rstest]
    #[case(PreReleaseLabel::Alpha, PreReleaseLabel::Alpha)]
    #[case(PreReleaseLabel::Beta, PreReleaseLabel::Beta)]
    #[case(PreReleaseLabel::Rc, PreReleaseLabel::Rc)]
    fn test_pre_release_label_equal(#[case] left: PreReleaseLabel, #[case] right: PreReleaseLabel) {
        assert_eq!(left, right);
    }

    #[rstest]
    #[case(PreReleaseLabel::Beta, PreReleaseLabel::Alpha)]
    #[case(PreReleaseLabel::Rc, PreReleaseLabel::Alpha)]
    #[case(PreReleaseLabel::Rc, PreReleaseLabel::Beta)]
    fn test_pre_release_label_greater_than(
        #[case] left: PreReleaseLabel,
        #[case] right: PreReleaseLabel,
    ) {
        assert!(left > right);
    }

    #[test]
    fn test_basic_version_ordering() {
        let v1 = PEP440Version::new(vec![1, 0, 0]);
        let v2 = PEP440Version::new(vec![2, 0, 0]);
        assert!(v1 < v2);
    }

    #[test]
    fn test_pre_release_ordering() {
        let stable = PEP440Version::new(vec![1, 0, 0]);
        let alpha =
            PEP440Version::new(vec![1, 0, 0]).with_pre_release(PreReleaseLabel::Alpha, Some(1));
        assert!(alpha < stable);
    }

    #[test]
    fn test_post_release_ordering() {
        let base = PEP440Version::new(vec![1, 0, 0]);
        let post = PEP440Version::new(vec![1, 0, 0]).with_post(1);
        assert!(base < post);
    }

    #[test]
    fn test_dev_release_ordering() {
        let stable = PEP440Version::new(vec![1, 0, 0]);
        let dev = PEP440Version::new(vec![1, 0, 0]).with_dev(1);
        assert!(dev < stable);
    }

    #[test]
    fn test_epoch_ordering() {
        let v1 = PEP440Version::new(vec![2, 0, 0]);
        let v2 = PEP440Version::new(vec![1, 0, 0]).with_epoch(1);
        assert!(v1 < v2);
    }

    #[rstest]
    #[case(LocalSegment::Integer(1), LocalSegment::Integer(2))]
    #[case(LocalSegment::String("a".to_string()), LocalSegment::String("b".to_string()))]
    #[case(LocalSegment::Integer(1), LocalSegment::String("a".to_string()))]
    fn test_local_segment_less_than(#[case] left: LocalSegment, #[case] right: LocalSegment) {
        assert!(left < right);
    }

    #[rstest]
    #[case(LocalSegment::Integer(1), LocalSegment::Integer(1))]
    #[case(LocalSegment::String("a".to_string()), LocalSegment::String("a".to_string()))]
    fn test_local_segment_equal(#[case] left: LocalSegment, #[case] right: LocalSegment) {
        assert_eq!(left, right);
    }

    #[rstest]
    #[case(LocalSegment::Integer(2), LocalSegment::Integer(1))]
    #[case(LocalSegment::String("b".to_string()), LocalSegment::String("a".to_string()))]
    #[case(LocalSegment::String("a".to_string()), LocalSegment::Integer(1))]
    fn test_local_segment_greater_than(#[case] left: LocalSegment, #[case] right: LocalSegment) {
        assert!(left > right);
    }

    #[test]
    fn test_local_segment_vec_ordering() {
        let v1 = vec![
            LocalSegment::String("ubuntu".to_string()),
            LocalSegment::Integer(20),
        ];
        let v2 = vec![
            LocalSegment::String("ubuntu".to_string()),
            LocalSegment::Integer(22),
        ];
        assert!(v1 < v2);

        let v3 = vec![
            LocalSegment::Integer(1),
            LocalSegment::String("build".to_string()),
        ];
        let v4 = vec![
            LocalSegment::Integer(1),
            LocalSegment::String("build".to_string()),
            LocalSegment::Integer(2),
        ];
        assert!(v3 < v4);

        let v5 = vec![LocalSegment::Integer(1)];
        let v6 = vec![LocalSegment::String("a".to_string())];
        assert!(v5 < v6);
    }

    #[test]
    fn test_pep440_version_with_local_ordering() {
        // Local versions are ignored in PEP 440 comparison
        let v1 = PEP440Version::new(vec![1, 0, 0]);
        let v2 = PEP440Version::new(vec![1, 0, 0])
            .with_local(vec![LocalSegment::String("build".to_string())]);
        assert_eq!(v1, v2);

        let v3 = PEP440Version::new(vec![1, 0, 0]).with_local(vec![LocalSegment::Integer(1)]);
        let v4 = PEP440Version::new(vec![1, 0, 0]).with_local(vec![LocalSegment::Integer(999)]);
        assert_eq!(v3, v4);
    }

    #[test]
    fn test_complex_pep440_version_ordering() {
        let alpha1 =
            PEP440Version::new(vec![1, 0, 0]).with_pre_release(PreReleaseLabel::Alpha, Some(1));
        let alpha1_post = PEP440Version::new(vec![1, 0, 0])
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(1);
        let alpha2 =
            PEP440Version::new(vec![1, 0, 0]).with_pre_release(PreReleaseLabel::Alpha, Some(2));
        let stable = PEP440Version::new(vec![1, 0, 0]);

        assert!(alpha1 < alpha1_post);
        assert!(alpha1_post < alpha2);
        assert!(alpha2 < stable);
    }

    #[test]
    fn test_pre_release_vs_post_dev_ordering() {
        let pre =
            PEP440Version::new(vec![1, 0, 0]).with_pre_release(PreReleaseLabel::Alpha, Some(1));
        let post = PEP440Version::new(vec![1, 0, 0]).with_post(1);
        let dev = PEP440Version::new(vec![1, 0, 0]).with_dev(1);
        let stable = PEP440Version::new(vec![1, 0, 0]);

        assert!(pre < dev);
        assert!(dev < stable);
        assert!(stable < post);
    }
}
