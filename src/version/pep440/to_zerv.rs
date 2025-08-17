use super::{LocalSegment, PEP440};
use crate::version::zerv::{Component, PreReleaseVar, Zerv, ZervFormat, ZervVars};

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
            format: ZervFormat {
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
    use crate::version::zerv::{Component, PreReleaseLabel};
    use rstest::rstest;

    #[rstest]
    // Basic version
    #[case("1.2.3", with_version(1, 2, 3))]
    // With epoch
    #[case("2!1.2.3", {
        let mut zerv = with_version(1, 2, 3);
        zerv.format.extra_core.push(Component::VarField("epoch".to_string()));
        zerv.vars.epoch = Some(2);
        zerv
    })]
    // With pre-release
    #[case("1.2.3a1", {
        let mut zerv = with_version(1, 2, 3);
        zerv.format.extra_core.push(Component::VarField("pre_release".to_string()));
        zerv.vars.pre_release = Some(crate::version::zerv::PreReleaseVar {
            label: PreReleaseLabel::Alpha,
            number: Some(1),
        });
        zerv
    })]
    // With post
    #[case("1.2.3.post1", {
        let mut zerv = with_version(1, 2, 3);
        zerv.format.extra_core.push(Component::VarField("post".to_string()));
        zerv.vars.post = Some(1);
        zerv
    })]
    // With dev
    #[case("1.2.3.dev1", {
        let mut zerv = with_version(1, 2, 3);
        zerv.format.extra_core.push(Component::VarField("dev".to_string()));
        zerv.vars.dev = Some(1);
        zerv
    })]
    // With local
    #[case("1.2.3+ubuntu.20.4", {
        let mut zerv = with_version(1, 2, 3);
        zerv.format.build = vec![
            Component::String("ubuntu".to_string()),
            Component::Integer(20),
            Component::Integer(4),
        ];
        zerv
    })]
    // Complex version with all components
    #[case("2!1.2.3a1.post1.dev1+local.1", {
        let mut zerv = with_version(1, 2, 3);
        zerv.format.extra_core = vec![
            Component::VarField("epoch".to_string()),
            Component::VarField("pre_release".to_string()),
            Component::VarField("post".to_string()),
            Component::VarField("dev".to_string()),
        ];
        zerv.format.build = vec![
            Component::String("local".to_string()),
            Component::Integer(1),
        ];
        zerv.vars.epoch = Some(2);
        zerv.vars.pre_release = Some(crate::version::zerv::PreReleaseVar {
            label: PreReleaseLabel::Alpha,
            number: Some(1),
        });
        zerv.vars.post = Some(1);
        zerv.vars.dev = Some(1);
        zerv
    })]
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
