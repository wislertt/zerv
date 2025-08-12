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

        // Compare release versions (normalize trailing zeros)
        match compare_release_versions(&self.release, &other.release) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare pre-release (None > Some for pre-release)
        match (&self.pre_label, &other.pre_label) {
            (None, None) => {}
            (None, Some(_)) => return Ordering::Greater,
            (Some(_), None) => return Ordering::Less,
            (Some(self_pre), Some(other_pre)) => match self_pre.cmp(other_pre) {
                Ordering::Equal => {
                    // Handle implicit vs explicit 0: None (implicit) < Some(0) (explicit) < Some(n) where n > 0
                    match (self.pre_number, other.pre_number) {
                        (None, None) => {}
                        (None, Some(0)) => return Ordering::Less, // implicit 0 < explicit 0
                        (None, Some(_)) => return Ordering::Less, // implicit 0 < explicit n
                        (Some(0), None) => return Ordering::Greater, // explicit 0 > implicit 0
                        (Some(_), None) => return Ordering::Greater, // explicit n > implicit 0
                        (Some(a), Some(b)) => match a.cmp(&b) {
                            Ordering::Equal => {}
                            other => return other,
                        },
                    }
                }
                other => return other,
            },
        }

        // Compare post-release (None < Some for post-release)
        match (&self.post_label, &other.post_label) {
            (None, None) => {}
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(_), Some(_)) => {
                // Handle implicit vs explicit 0: None (implicit) < Some(0) (explicit) < Some(n) where n > 0
                match (self.post_number, other.post_number) {
                    (None, None) => {}
                    (None, Some(0)) => return Ordering::Less, // implicit 0 < explicit 0
                    (None, Some(_)) => return Ordering::Less, // implicit 0 < explicit n
                    (Some(0), None) => return Ordering::Greater, // explicit 0 > implicit 0
                    (Some(_), None) => return Ordering::Greater, // explicit n > implicit 0
                    (Some(a), Some(b)) => match a.cmp(&b) {
                        Ordering::Equal => {}
                        other => return other,
                    },
                }
            }
        }

        // Compare dev-release (None > Some for dev-release)
        match (&self.dev_label, &other.dev_label) {
            (None, None) => {}
            (None, Some(_)) => return Ordering::Greater,
            (Some(_), None) => return Ordering::Less,
            (Some(_), Some(_)) => {
                // Handle implicit vs explicit 0: None (implicit) < Some(0) (explicit) < Some(n) where n > 0
                match (self.dev_number, other.dev_number) {
                    (None, None) => {}
                    (None, Some(0)) => return Ordering::Less, // implicit 0 < explicit 0
                    (None, Some(_)) => return Ordering::Less, // implicit 0 < explicit n
                    (Some(0), None) => return Ordering::Greater, // explicit 0 > implicit 0
                    (Some(_), None) => return Ordering::Greater, // explicit n > implicit 0
                    (Some(a), Some(b)) => match a.cmp(&b) {
                        Ordering::Equal => {}
                        other => return other,
                    },
                }
            }
        }

        // Compare local versions (None < Some)
        match (&self.local, &other.local) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(self_local), Some(other_local)) => self_local.cmp(other_local),
        }
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

// Helper function to compare release versions with trailing zero normalization
fn compare_release_versions(left: &[u32], right: &[u32]) -> Ordering {
    let left_len = left.len();
    let right_len = right.len();
    let max_len = left_len.max(right_len);

    for i in 0..max_len {
        let left_val = left.get(i).copied().unwrap_or(0);
        let right_val = right.get(i).copied().unwrap_or(0);

        match left_val.cmp(&right_val) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    Ordering::Equal
}

impl Eq for PEP440Version {}

#[cfg(test)]
mod tests {
    use super::super::core::{DevLabel, PostLabel};
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
        assert!(left <= right);
        assert!(right > left);
        assert!(right >= left);
    }

    #[rstest]
    #[case(PreReleaseLabel::Alpha, PreReleaseLabel::Alpha)]
    #[case(PreReleaseLabel::Beta, PreReleaseLabel::Beta)]
    #[case(PreReleaseLabel::Rc, PreReleaseLabel::Rc)]
    fn test_pre_release_label_equal(#[case] left: PreReleaseLabel, #[case] right: PreReleaseLabel) {
        assert_eq!(left, right);
    }

    #[rstest]
    #[case(PostLabel::Post, PostLabel::Post)]
    fn test_post_label_equal(#[case] left: PostLabel, #[case] right: PostLabel) {
        assert_eq!(left, right);
    }

    #[rstest]
    #[case(DevLabel::Dev, DevLabel::Dev)]
    fn test_dev_label_equal(#[case] left: DevLabel, #[case] right: DevLabel) {
        assert_eq!(left, right);
    }

    #[rstest]
    #[case("1", "1")]
    #[case("a", "a")]
    #[case("0", "0")]
    #[case("build", "build")]
    #[case("999", "999")]
    #[case("z", "z")]
    fn test_local_segment_equality(#[case] left: &str, #[case] right: &str) {
        use super::super::parser::parse_local_segments;
        let left_local = &parse_local_segments(left)[0];
        let right_local = &parse_local_segments(right)[0];
        assert_eq!(left_local, right_local);
    }

    #[rstest]
    #[case("ubuntu.20", "ubuntu.22")]
    #[case("1.build", "1.build.2")]
    #[case("1", "a")]
    #[case("a", "b")]
    #[case("1", "2")]
    #[case("0", "1")]
    #[case("9", "10")]
    #[case("a", "z")]
    #[case("build", "release")]
    #[case("1.0", "1.1")]
    #[case("alpha", "beta")]
    #[case("dev", "prod")]
    #[case("0.build", "1.build")]
    #[case("build.0", "build.1")]
    #[case("1.2.3", "1.2.4")]
    #[case("a.b.c", "a.b.d")]
    fn test_local_segment_ordering_less_than(#[case] left: &str, #[case] right: &str) {
        use super::super::parser::parse_local_segments;
        let left_local = parse_local_segments(left);
        let right_local = parse_local_segments(right);
        assert!(left_local < right_local);
        assert!(left_local <= right_local);
        assert!(right_local > left_local);
        assert!(right_local >= left_local);
    }

    #[rstest]
    // Reflexivity: a == a
    #[case("1.0.0")]
    #[case("1!2.3.4a5.post6.dev7+local.8")]
    #[case("0.0.0")]
    #[case(
        "4294967295!4294967295.4294967295.4294967295a4294967295.post4294967295.dev4294967295+4294967295"
    )]
    fn test_ordering_reflexivity(#[case] version_str: &str) {
        let version: PEP440Version = version_str.parse().unwrap();
        // Reflexivity: a version should equal itself
        assert_eq!(version.cmp(&version), std::cmp::Ordering::Equal);
    }

    #[rstest]
    // Transitivity: if a < b and b < c, then a < c
    #[case("1.0.0a1", "1.0.0a2", "1.0.0a3")]
    #[case("1.0.0.dev1", "1.0.0", "1.0.0.post1")]
    #[case("1.0.0+a", "1.0.0+b", "1.0.0+c")]
    #[case("0!1.0.0", "1!0.0.0", "2!0.0.0")]
    #[case("1!0.0.0", "1!0.0.1", "1!0.1.0")]
    #[case("1.0.0", "1!0.0.0", "2!0.0.0")]
    fn test_ordering_transitivity(#[case] a: &str, #[case] b: &str, #[case] c: &str) {
        let version_a: PEP440Version = a.parse().unwrap();
        let version_b: PEP440Version = b.parse().unwrap();
        let version_c: PEP440Version = c.parse().unwrap();

        assert!(version_a < version_b);
        assert!(version_b < version_c);
        assert!(version_a < version_c); // transitivity
    }

    #[rstest]
    // Normal case
    #[case("1.0.0", "2.0.0")]
    #[case("1.0.0", "1.0.0.post1")]
    #[case("1.0.0.dev1", "1.0.0")]
    #[case("1.0.0a1", "1.0.0")]
    #[case("1.0.0+a", "1.0.0+b")]
    #[case("1.0.0.dev1", "1.0.0.post1")]
    // Epoch ordering
    #[case("2.0.0", "1!1.0.0")] // no epoch < epoch 1
    #[case("999.999.999", "1!0.0.0")] // large version < epoch 1
    #[case("1!0.0.0", "2!0.0.0")] // epoch 1 < epoch 2
    #[case("1!999.999.999", "2!0.0.0")] // epoch 1 large < epoch 2 small
    #[case("0!1.0.0", "1!0.0.0")] // explicit epoch 0 < epoch 1
    #[case("4294967294!1.0.0", "4294967295!0.0.0")] // max-1 epoch < max epoch
    // Local version ordering
    #[case("1.0.0", "1.0.0+build")] // no local < with local
    #[case("1.0.0+1", "1.0.0+999")] // single segment numeric
    #[case("1.0.0+a", "1.0.0+z")] // single segment string
    #[case("1.0.0+ubuntu.20", "1.0.0+ubuntu.22")] // multi-segment numeric
    #[case("1.0.0+build.1", "1.0.0+build.2")] // multi-segment mixed
    #[case("1.0.0+dev.alpha", "1.0.0+dev.beta")] // multi-segment string
    #[case("1.0.0+1.2.3", "1.0.0+1.2.4")] // multi-segment all numeric
    #[case("1.0.0+a.b.c", "1.0.0+a.b.d")] // multi-segment all string
    #[case("1.0.0+build", "1.0.0+build.1")] // fewer segments < more segments
    #[case("1.0.0+1.build", "1.0.0+2.build")] // numeric first segment
    #[case("1.0.0+1", "1.0.0+a")] // numeric < string
    #[case("1.0.0+1.alpha", "1.0.0+1.beta")] // same numeric, different string
    // Local segment count edge cases
    #[case("1.0.0+1", "1.0.0+1.2")] // single vs multi numeric
    #[case("1.0.0+a", "1.0.0+a.b")] // single vs multi string
    #[case("1.0.0+1.2", "1.0.0+1.2.0")] // shorter vs longer with trailing zero
    #[case("1.0.0+build", "1.0.0+build.0")] // string vs string with zero
    #[case("1.0.0+1", "1.0.0+1.0")] // single vs double with zero
    // Complex pre/post/dev release ordering
    #[case("1.0.0a1", "1.0.0a1.post1")] // alpha < alpha.post
    #[case("1.0.0a1.dev1", "1.0.0a1")] // alpha.dev < alpha
    #[case("1.0.0a1.dev1", "1.0.0a1.post1")] // alpha.dev < alpha.post
    #[case("1.0.0a1.post1", "1.0.0a2")] // alpha.post < alpha2
    #[case("1.0.0a1", "1.0.0b1")] // alpha < beta
    #[case("1.0.0b1", "1.0.0b1.post1")] // beta < beta.post
    #[case("1.0.0b1.dev1", "1.0.0b1")] // beta.dev < beta
    #[case("1.0.0b1.dev1", "1.0.0b1.post1")] // beta.dev < beta.post
    #[case("1.0.0b1.post1", "1.0.0b2")] // beta.post < beta2
    #[case("1.0.0b1", "1.0.0rc1")] // beta < rc
    #[case("1.0.0rc1", "1.0.0rc1.post1")] // rc < rc.post
    #[case("1.0.0rc1.dev1", "1.0.0rc1")] // rc.dev < rc
    #[case("1.0.0rc1.dev1", "1.0.0rc1.post1")] // rc.dev < rc.post
    #[case("1.0.0rc1.post1", "1.0.0rc2")] // rc.post < rc2
    #[case("1.0.0rc1", "1.0.0")] // rc < stable
    #[case("1.0.0a", "1.0.0a0")] // alpha < alpha0
    #[case("1.0.0a0", "1.0.0a1")] // alpha0 < alpha1
    #[case("1.0.0a", "1.0.0a1")] // alpha < alpha1
    #[case("1.0.0a1", "1.0.0.dev1")] // alpha < stable.dev
    #[case("1.0.0b1", "1.0.0.dev1")] // beta < stable.dev
    #[case("1.0.0rc1", "1.0.0.dev1")] // rc < stable.dev
    #[case("1.0.0.dev1", "1.0.0")] // stable.dev < stable
    #[case("1.0.0", "1.0.0.post1")] // stable < stable.post
    #[case("1.0.0.post1.dev1", "1.0.0.post1")] // post.dev < post
    #[case("1.0.0.post1.dev1", "1.0.0.post2")] // post.dev < post2
    // Boundary value edge cases
    #[case("0.0.0", "0.0.1")] // minimum version
    #[case("4294967295.0.0", "4294967295.0.1")] // u32::MAX in release
    #[case("1.0.0a4294967295", "1.0.0b1")] // u32::MAX in pre-release number
    #[case("1.0.0.post4294967295", "1.0.1")] // u32::MAX in post-release
    #[case("1.0.0.dev4294967295", "1.0.0")] // u32::MAX in dev-release
    #[case("4294967295!0.0.0", "4294967295!0.0.1")] // u32::MAX in epoch
    #[case("1.0.0a0", "1.0.0a1")] // zero pre-release number
    #[case("1.0.0.post0", "1.0.0.post1")] // zero post-release number
    #[case("1.0.0.dev0", "1.0.0.dev1")] // zero dev-release number
    #[case("1.0.0+0", "1.0.0+1")] // zero local segment
    #[case("1.0.0+4294967295", "1.0.0+a")] // u32::MAX local vs string
    #[case("1.0.0+z", "1.0.0+z.0")] // string vs string with zero
    // Local version edge cases
    #[case("1.0.0+a", "1.0.0+b")] // single character local segments
    #[case("1.0.0+0", "1.0.0+9")] // single digit local segments
    #[case(
        "1.0.0+a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z",
        "1.0.0+a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z.1"
    )] // very long local segments
    #[case("1.0.0+0.a", "1.0.0+0.b")] // mixed numeric and string
    #[case("1.0.0+a.0", "1.0.0+a.1")] // mixed string and numeric
    fn test_version_ordering_less_than(#[case] left: &str, #[case] right: &str) {
        let left_version: PEP440Version = left.parse().unwrap();
        let right_version: PEP440Version = right.parse().unwrap();
        assert!(left_version < right_version);
        assert!(left_version <= right_version);
        assert!(right_version > left_version);
        assert!(right_version >= left_version);
    }

    // Comprehensive equality tests
    #[rstest]
    // Release format equality
    #[case("1", "1.0")]
    #[case("1", "1.0.0")]
    #[case("1.0", "1.0.0")]
    #[case("1.2", "1.2.0")]
    #[case("1.2.3", "1.2.3.0")]
    #[case("2", "2.0.0.0")]
    // Epoch equality
    #[case("0!1.0.0", "1.0.0")]
    #[case("0!1", "1")]
    #[case("0!1.2", "1.2")]
    #[case("0!2.3.4", "2.3.4")]
    // Prerelease equality
    #[case("1.0.0a1", "1.0.0a1")]
    #[case("1a", "1.0a")]
    #[case("1a", "1.0.0a")]
    #[case("1.2b3", "1.2.0b3")]
    #[case("2rc1", "2.0rc1")]
    // Post release equality
    #[case("1.post1", "1.0.post1")]
    #[case("1.post1", "1.0.0.post1")]
    #[case("1.2.post5", "1.2.0.post5")]
    #[case("2.post", "2.0.post")]
    // Dev release equality
    #[case("1.dev1", "1.0.dev1")]
    #[case("1.dev1", "1.0.0.dev1")]
    #[case("1.2.dev3", "1.2.0.dev3")]
    #[case("2.dev", "2.0.dev")]
    // Complex combination equality
    #[case("1a1.post2.dev3", "1.0a1.post2.dev3")]
    #[case("1a1.post2.dev3", "1.0.0a1.post2.dev3")]
    #[case("2b.post.dev1", "2.0b.post.dev1")]
    #[case("1!1a.post1", "1!1.0a.post1")]
    #[case("0!1.2rc3.dev", "1.2.0rc3.dev")]
    // Local version equality
    #[case("1+build", "1.0+build")]
    #[case("1+build", "1.0.0+build")]
    #[case("1.2+local.123", "1.2.0+local.123")]
    #[case("0!1+ubuntu.20.04", "1+ubuntu.20.4")] // epoch + local + leading zeros
    #[case("1.0.0+build", "1.0.0+build")]
    #[case("1.0.0+ubuntu.20.04", "1.0.0+ubuntu.20.4")] // leading zeros
    #[case("1.0.0+1.2.3", "1.0.0+1.2.3")]
    #[case("1.0.0+a.b.c", "1.0.0+a.b.c")]
    // Local equality
    #[case("1.0.0+0", "1.0.0+00")] // zero with leading zeros
    #[case("1.0.0+0.build", "1.0.0+00.build")] // zero first segment with leading zeros
    #[case("1.0.0+build.0", "1.0.0+build.00")] // zero after string with leading zeros
    #[case("1.0.0+0.0.0", "1.0.0+00.00.00")] // all zeros with leading zeros
    // Local version with leading zero
    #[case("1.0.0+01", "1.0.0+1")] // leading zeros in numeric segments
    #[case("1.0.0+ubuntu.020", "1.0.0+ubuntu.20")] // leading zeros in mixed segments
    #[case("1.0.0+1.02.003", "1.0.0+1.2.3")] // multiple leading zeros
    #[case("1.0.0+build.01", "1.0.0+build.1")] // leading zero in string-numeric mix
    fn test_pep440_version_equality(#[case] left: &str, #[case] right: &str) {
        let left_version: PEP440Version = left.parse().unwrap();
        let right_version: PEP440Version = right.parse().unwrap();
        assert_eq!(left_version, right_version);
    }
}
