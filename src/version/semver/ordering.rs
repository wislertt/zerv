use super::core::{PreReleaseIdentifier, SemVer};
use std::cmp::Ordering;

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major.minor.patch first
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| {
                // Pre-release versions have lower precedence than normal versions
                match (&self.pre_release, &other.pre_release) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Greater, // stable > pre-release
                    (Some(_), None) => Ordering::Less,    // pre-release < stable
                    (Some(self_pre), Some(other_pre)) => {
                        compare_pre_release_identifiers(self_pre, other_pre)
                    }
                }
            })
        // Build metadata MUST be ignored when determining version precedence
    }
}

impl PartialEq for SemVer {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for SemVer {}

impl PartialOrd for PreReleaseIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreReleaseIdentifier {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PreReleaseIdentifier::Integer(a), PreReleaseIdentifier::Integer(b)) => a.cmp(b),
            (PreReleaseIdentifier::String(a), PreReleaseIdentifier::String(b)) => a.cmp(b),
            (PreReleaseIdentifier::Integer(_), PreReleaseIdentifier::String(_)) => Ordering::Less,
            (PreReleaseIdentifier::String(_), PreReleaseIdentifier::Integer(_)) => {
                Ordering::Greater
            }
        }
    }
}

fn compare_pre_release_identifiers(
    left: &[PreReleaseIdentifier],
    right: &[PreReleaseIdentifier],
) -> Ordering {
    let min_len = left.len().min(right.len());

    for i in 0..min_len {
        match left[i].cmp(&right[i]) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    // If all compared identifiers are equal, the version with fewer identifiers has lower precedence
    left.len().cmp(&right.len())
}

#[cfg(test)]
mod tests {
    use super::super::core::BuildMetadata;
    use super::*;
    use rstest::rstest;

    mod basic_ordering {
        use super::*;

        #[test]
        fn test_major_version_ordering() {
            let v1 = SemVer::new(1, 0, 0);
            let v2 = SemVer::new(2, 0, 0);
            assert!(v1 < v2);
            assert!(v2 > v1);
        }

        #[test]
        fn test_minor_version_ordering() {
            let v1 = SemVer::new(1, 0, 0);
            let v2 = SemVer::new(1, 1, 0);
            assert!(v1 < v2);
            assert!(v2 > v1);
        }

        #[test]
        fn test_patch_version_ordering() {
            let v1 = SemVer::new(1, 0, 0);
            let v2 = SemVer::new(1, 0, 1);
            assert!(v1 < v2);
            assert!(v2 > v1);
        }

        #[test]
        fn test_equal_versions() {
            let v1 = SemVer::new(1, 2, 3);
            let v2 = SemVer::new(1, 2, 3);
            assert_eq!(v1, v2);
            assert!(v1 <= v2);
            assert!(v1 >= v2);
        }

        #[test]
        fn test_reflexivity() {
            let version = SemVer::new(1, 2, 3);
            assert_eq!(version.cmp(&version), Ordering::Equal);
        }
    }

    mod pre_release_ordering {
        use super::*;

        #[test]
        fn test_stable_vs_pre_release() {
            let stable = SemVer::new(1, 0, 0);
            let pre_release = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("alpha".to_string())]);

            assert!(pre_release < stable);
            assert!(stable > pre_release);
        }

        #[test]
        fn test_pre_release_string_ordering() {
            let alpha = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("alpha".to_string())]);
            let beta = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("beta".to_string())]);

            assert!(alpha < beta);
            assert!(beta > alpha);
        }

        #[test]
        fn test_pre_release_integer_ordering() {
            let v1 = SemVer::new(1, 0, 0).with_pre_release(vec![PreReleaseIdentifier::Integer(1)]);
            let v2 = SemVer::new(1, 0, 0).with_pre_release(vec![PreReleaseIdentifier::Integer(2)]);

            assert!(v1 < v2);
            assert!(v2 > v1);
        }

        #[test]
        fn test_pre_release_mixed_type_ordering() {
            let integer =
                SemVer::new(1, 0, 0).with_pre_release(vec![PreReleaseIdentifier::Integer(1)]);
            let string = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("alpha".to_string())]);

            assert!(integer < string); // integers < strings
            assert!(string > integer);
        }

        #[test]
        fn test_pre_release_length_ordering() {
            let shorter = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("alpha".to_string())]);
            let longer = SemVer::new(1, 0, 0).with_pre_release(vec![
                PreReleaseIdentifier::String("alpha".to_string()),
                PreReleaseIdentifier::Integer(1),
            ]);

            assert!(shorter < longer); // fewer identifiers < more identifiers
            assert!(longer > shorter);
        }

        #[test]
        fn test_complex_pre_release_ordering() {
            let v1 = SemVer::new(1, 0, 0).with_pre_release(vec![
                PreReleaseIdentifier::String("alpha".to_string()),
                PreReleaseIdentifier::Integer(1),
            ]);
            let v2 = SemVer::new(1, 0, 0).with_pre_release(vec![
                PreReleaseIdentifier::String("alpha".to_string()),
                PreReleaseIdentifier::Integer(2),
            ]);

            assert!(v1 < v2);
            assert!(v2 > v1);
        }
    }

    mod build_metadata_ignored {
        use super::*;

        #[test]
        fn test_build_metadata_ignored_in_comparison() {
            let v1 = SemVer::new(1, 0, 0)
                .with_build_metadata(vec![BuildMetadata::String("build1".to_string())]);
            let v2 = SemVer::new(1, 0, 0)
                .with_build_metadata(vec![BuildMetadata::String("build2".to_string())]);

            assert_eq!(v1, v2); // build metadata is ignored
        }

        #[test]
        fn test_build_metadata_with_pre_release() {
            let v1 = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("alpha".to_string())])
                .with_build_metadata(vec![BuildMetadata::String("build1".to_string())]);
            let v2 = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("alpha".to_string())])
                .with_build_metadata(vec![BuildMetadata::String("build2".to_string())]);

            assert_eq!(v1, v2); // build metadata is ignored
        }

        #[test]
        fn test_no_build_metadata_vs_with_build_metadata() {
            let v1 = SemVer::new(1, 0, 0);
            let v2 = SemVer::new(1, 0, 0)
                .with_build_metadata(vec![BuildMetadata::String("build".to_string())]);

            assert_eq!(v1, v2); // build metadata is ignored
        }
    }

    mod identifier_ordering {
        use super::*;

        #[test]
        fn test_integer_identifier_ordering() {
            let id1 = PreReleaseIdentifier::Integer(1);
            let id2 = PreReleaseIdentifier::Integer(2);
            assert!(id1 < id2);
            assert!(id2 > id1);
        }

        #[test]
        fn test_string_identifier_ordering() {
            let id1 = PreReleaseIdentifier::String("alpha".to_string());
            let id2 = PreReleaseIdentifier::String("beta".to_string());
            assert!(id1 < id2);
            assert!(id2 > id1);
        }

        #[test]
        fn test_mixed_identifier_ordering() {
            let integer = PreReleaseIdentifier::Integer(999);
            let string = PreReleaseIdentifier::String("a".to_string());
            assert!(integer < string); // integers always < strings
            assert!(string > integer);
        }

        #[test]
        fn test_identifier_equality() {
            let id1 = PreReleaseIdentifier::String("alpha".to_string());
            let id2 = PreReleaseIdentifier::String("alpha".to_string());
            assert_eq!(id1, id2);

            let id3 = PreReleaseIdentifier::Integer(42);
            let id4 = PreReleaseIdentifier::Integer(42);
            assert_eq!(id3, id4);
        }
    }

    mod semver_version_equality {
        use super::*;

        #[rstest]
        #[case("1.0.0", "1.0.0")]
        #[case("1.2.3", "1.2.3")]
        #[case("0.0.0", "0.0.0")]
        #[case("1.0.0-alpha", "1.0.0-alpha")]
        #[case("1.0.0-alpha.1", "1.0.0-alpha.1")]
        #[case("1.0.0-alpha.beta", "1.0.0-alpha.beta")]
        #[case("1.0.0+build", "1.0.0+build")]
        #[case("1.0.0+build.1", "1.0.0+build.1")]
        #[case("1.0.0-alpha+build", "1.0.0-alpha+build")]
        #[case("1.0.0-alpha.1+build.1", "1.0.0-alpha.1+build.1")]
        #[case("1.0.0+build1", "1.0.0+build2")] // build metadata ignored
        #[case("1.0.0-alpha+build1", "1.0.0-alpha+build2")] // build metadata ignored
        #[case("1.0.0-alpha+build1", "1.0.0-alpha+Build2")] // build metadata ignored
        #[case("1.0.0-alpha+build1", "1.0.0-alpha+BUILD2")] // build metadata ignored
        fn test_semver_version_equality(#[case] left: &str, #[case] right: &str) {
            let left_version: SemVer = left.parse().unwrap();
            let right_version: SemVer = right.parse().unwrap();
            assert_eq!(left_version, right_version);
        }

        // #[rstest]
        // #[case("1.0.0-alpha", "1.0.0-ALPHA")]
        // #[case("1.0.0-Alpha", "1.0.0-alpha")]
        // fn test_case_sensitivity_inequality(#[case] left: &str, #[case] right: &str) {
        //     let left_version: SemVer = left.parse().unwrap();
        //     let right_version: SemVer = right.parse().unwrap();
        //     assert_ne!(left_version, right_version); // case sensitive
        // }
    }

    mod comprehensive_ordering {
        use super::*;

        #[rstest]
        #[case("1.0.0", "2.0.0")]
        #[case("2.0.0", "2.1.0")]
        #[case("2.1.0", "2.1.1")]
        #[case("1.0.0-alpha", "1.0.0")]
        #[case("1.0.0-alpha", "1.0.0-alpha.1")]
        #[case("1.0.0-alpha.1", "1.0.0-alpha.beta")]
        #[case("1.0.0-alpha.beta", "1.0.0-beta")]
        #[case("1.0.0-beta", "1.0.0-beta.2")]
        #[case("1.0.0-beta.2", "1.0.0-beta.11")]
        #[case("1.0.0-beta.11", "1.0.0-rc.1")]
        #[case("1.0.0-rc.1", "1.0.0")]
        #[case("1.0.0-ALPHA", "1.0.0-alpha")]
        #[case("1.0.0-Alpha", "1.0.0-alpha")]
        fn test_semver_spec_examples(#[case] left: &str, #[case] right: &str) {
            let left_version: SemVer = left.parse().unwrap();
            let right_version: SemVer = right.parse().unwrap();
            assert!(left_version < right_version, "{left} should be < {right}");
            assert!(left_version <= right_version, "{left} should be <= {right}");
            assert!(right_version > left_version, "{left} should be > {right}");
            assert!(right_version >= left_version, "{left} should be >= {right}");
        }

        #[test]
        fn test_transitivity() {
            let v1 = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("alpha".to_string())]);
            let v2 = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("beta".to_string())]);
            let v3 = SemVer::new(1, 0, 0);

            assert!(v1 < v2);
            assert!(v2 < v3);
            assert!(v1 < v3); // transitivity
        }

        #[test]
        fn test_antisymmetry() {
            let v1 = SemVer::new(1, 0, 0);
            let v2 = SemVer::new(2, 0, 0);

            assert!(v1 < v2);
            assert!(v2 >= v1);
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_zero_versions() {
            let v1 = SemVer::new(0, 0, 0);
            let v2 = SemVer::new(0, 0, 1);
            assert!(v1 < v2);
        }

        #[test]
        fn test_max_values() {
            let v1 = SemVer::new(u64::MAX - 1, u64::MAX, u64::MAX);
            let v2 = SemVer::new(u64::MAX, 0, 0);
            assert!(v1 < v2);
        }

        #[test]
        fn test_empty_pre_release() {
            let v1 = SemVer::new(1, 0, 0).with_pre_release(vec![]);
            let v2 = SemVer::new(1, 0, 0);
            assert!(v1 < v2); // empty pre-release still makes it a pre-release
        }

        #[test]
        fn test_very_long_pre_release() {
            let long_pre_release = (0..100).map(PreReleaseIdentifier::Integer).collect();
            let short_pre_release = vec![PreReleaseIdentifier::Integer(0)];

            let v1 = SemVer::new(1, 0, 0).with_pre_release(short_pre_release);
            let v2 = SemVer::new(1, 0, 0).with_pre_release(long_pre_release);

            assert!(v1 < v2); // fewer identifiers < more identifiers
        }

        #[test]
        fn test_numeric_string_comparison() {
            let v1 = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("10".to_string())]);
            let v2 = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("2".to_string())]);

            // String comparison: "10" < "2" lexicographically
            assert!(v1 < v2);
        }

        #[test]
        fn test_integer_vs_numeric_string() {
            let integer =
                SemVer::new(1, 0, 0).with_pre_release(vec![PreReleaseIdentifier::Integer(10)]);
            let string = SemVer::new(1, 0, 0)
                .with_pre_release(vec![PreReleaseIdentifier::String("2".to_string())]);

            // Integer < String regardless of numeric value
            assert!(integer < string);
        }
    }

    mod helper_function_tests {
        use super::*;

        #[test]
        fn test_compare_pre_release_identifiers_empty() {
            let left = vec![];
            let right = vec![];
            assert_eq!(
                compare_pre_release_identifiers(&left, &right),
                Ordering::Equal
            );
        }

        #[test]
        fn test_compare_pre_release_identifiers_different_lengths() {
            let left = vec![PreReleaseIdentifier::String("alpha".to_string())];
            let right = vec![
                PreReleaseIdentifier::String("alpha".to_string()),
                PreReleaseIdentifier::Integer(1),
            ];
            assert_eq!(
                compare_pre_release_identifiers(&left, &right),
                Ordering::Less
            );
            assert_eq!(
                compare_pre_release_identifiers(&right, &left),
                Ordering::Greater
            );
        }

        #[test]
        fn test_compare_pre_release_identifiers_same_prefix() {
            let left = vec![
                PreReleaseIdentifier::String("alpha".to_string()),
                PreReleaseIdentifier::Integer(1),
            ];
            let right = vec![
                PreReleaseIdentifier::String("alpha".to_string()),
                PreReleaseIdentifier::Integer(2),
            ];
            assert_eq!(
                compare_pre_release_identifiers(&left, &right),
                Ordering::Less
            );
        }
    }
}
