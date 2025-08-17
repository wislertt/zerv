use super::{LocalSegment, PEP440};
use crate::version::zerv::{Component, PreReleaseVar, Zerv, ZervSchema, ZervVars};

impl From<PEP440> for Zerv {
    fn from(pep440: PEP440) -> Self {
        let mut extra_core = Vec::new();
        let mut build = Vec::new();

        // Add epoch to extra_core if non-zero
        if pep440.epoch > 0 {
            extra_core.push(Component::VarField("epoch".to_string()));
        }

        // Add pre-release to extra_core if present
        let pre_release = if let (Some(label), number) = (pep440.pre_label, pep440.pre_number) {
            extra_core.push(Component::VarField("pre_release".to_string()));
            Some(PreReleaseVar {
                label,
                number: number.map(|n| n as u64),
            })
        } else {
            None
        };

        // Add post to extra_core if present
        let post = if pep440.post_label.is_some() {
            extra_core.push(Component::VarField("post".to_string()));
            pep440.post_number.map(|n| n as u64)
        } else {
            None
        };

        // Add dev to extra_core if present
        let dev = if pep440.dev_label.is_some() {
            extra_core.push(Component::VarField("dev".to_string()));
            pep440.dev_number.map(|n| n as u64)
        } else {
            None
        };

        // Process local segments - they go to build
        if let Some(local_segments) = pep440.local {
            for segment in local_segments {
                match segment {
                    LocalSegment::String(s) => {
                        build.push(Component::String(s));
                    }
                    LocalSegment::Integer(n) => {
                        build.push(Component::Integer(n as u64));
                    }
                }
            }
        }

        // Extract major, minor, patch from release
        let major = pep440.release.first().copied().map(|n| n as u64);
        let minor = pep440.release.get(1).copied().map(|n| n as u64);
        let patch = pep440.release.get(2).copied().map(|n| n as u64);

        Zerv {
            schema: ZervSchema {
                core: vec![
                    Component::VarField("major".to_string()),
                    Component::VarField("minor".to_string()),
                    Component::VarField("patch".to_string()),
                ],
                extra_core,
                build,
            },
            vars: ZervVars {
                major,
                minor,
                patch,
                epoch: if pep440.epoch > 0 {
                    Some(pep440.epoch as u64)
                } else {
                    None
                },
                pre_release,
                post,
                dev,
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::test_utils::*;

    use rstest::rstest;

    #[rstest]
    #[case("1.2.3", pep_zerv_1_2_3())]
    #[case("2!1.2.3", pep_zerv_1_2_3_epoch_2())]
    #[case("1.2.3a1", pep_zerv_1_2_3_alpha_1())]
    #[case("1.2.3.post1", pep_zerv_1_2_3_post_1())]
    #[case("1.2.3.dev1", pep_zerv_1_2_3_dev_1())]
    #[case("1.2.3+ubuntu.20.4", pep_zerv_1_2_3_ubuntu_build())]
    #[case(
        "2!1.2.3a1.post1.dev1+local.1",
        pep_zerv_complex_2_1_2_3_alpha_1_post_1_dev_1_local_1()
    )]
    fn test_pep440_to_zerv_conversion(#[case] pep440_str: &str, #[case] expected: Zerv) {
        let pep440: PEP440 = pep440_str.parse().unwrap();
        let zerv: Zerv = pep440.into();
        assert_eq!(zerv, expected);
    }

    #[test]
    fn test_round_trip_conversion() {
        let original: PEP440 = "2!1.2.3a1.post1.dev1+local.1".parse().unwrap();
        let zerv: Zerv = original.clone().into();
        let converted: PEP440 = zerv.into();

        assert_eq!(original.to_string(), converted.to_string());
    }
}
