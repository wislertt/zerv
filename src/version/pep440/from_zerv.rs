use super::{LocalSegment, PEP440};
use crate::version::pep440::core::{DevLabel, PostLabel};
use crate::version::zerv::{Component, Zerv, resolve_timestamp};

impl From<Zerv> for PEP440 {
    fn from(zerv: Zerv) -> Self {
        // Extract values from core components
        let mut core_values = Vec::new();
        for comp in &zerv.format.core {
            let val = match comp {
                Component::VarField(field) => match field.as_str() {
                    "major" => zerv.vars.major.unwrap_or(0),
                    "minor" => zerv.vars.minor.unwrap_or(0),
                    "patch" => zerv.vars.patch.unwrap_or(0),
                    _ => 0,
                },
                Component::VarTimestamp(pattern) => {
                    resolve_timestamp(pattern, zerv.vars.tag_timestamp).unwrap_or(0)
                }
                Component::Integer(n) => *n,
                _ => 0,
            };
            core_values.push(val as u32);
        }

        // Extract release from core, filtering out non-numeric components that overflow to local
        let mut release = Vec::new();
        let mut local_overflow = Vec::new();

        for val in core_values {
            release.push(val);
        }

        // If release is empty, default to [0]
        if release.is_empty() {
            release.push(0);
        }

        // Process extra_core for epoch, pre-release, post, dev, and other components
        let mut epoch = 0;
        let mut pre_label = None;
        let mut pre_number = None;
        let mut post_label = None;
        let mut post_number = None;
        let mut dev_label = None;
        let mut dev_number = None;

        for comp in &zerv.format.extra_core {
            match comp {
                Component::VarField(field) => match field.as_str() {
                    "pre_release" => {
                        if let Some(pr) = &zerv.vars.pre_release {
                            pre_label = Some(pr.label.clone());
                            pre_number = pr.number.map(|n| n as u32);
                        }
                    }
                    "epoch" => {
                        epoch = zerv.vars.epoch.unwrap_or(0) as u32;
                    }
                    "post" => {
                        post_label = Some(PostLabel::Post);
                        post_number = zerv.vars.post.map(|n| n as u32);
                    }
                    "dev" => {
                        dev_label = Some(DevLabel::Dev);
                        dev_number = zerv.vars.dev.map(|n| n as u32);
                    }
                    _ => {
                        // Other fields overflow to local
                        local_overflow.push(LocalSegment::String(field.clone()));
                    }
                },
                Component::String(s) => {
                    local_overflow.push(LocalSegment::String(s.clone()));
                }
                Component::Integer(n) => {
                    if *n <= u32::MAX as u64 {
                        local_overflow.push(LocalSegment::Integer(*n as u32));
                    } else {
                        local_overflow.push(LocalSegment::String(n.to_string()));
                    }
                }
                _ => {}
            }
        }

        // Process build components - they go to local
        for comp in &zerv.format.build {
            match comp {
                Component::String(s) => {
                    local_overflow.push(LocalSegment::String(s.clone()));
                }
                Component::Integer(n) => {
                    if *n <= u32::MAX as u64 {
                        local_overflow.push(LocalSegment::Integer(*n as u32));
                    } else {
                        local_overflow.push(LocalSegment::String(n.to_string()));
                    }
                }
                Component::VarTimestamp(pattern) => {
                    let val = resolve_timestamp(pattern, zerv.vars.tag_timestamp).unwrap_or(0);
                    if val <= u32::MAX as u64 {
                        local_overflow.push(LocalSegment::Integer(val as u32));
                    } else {
                        local_overflow.push(LocalSegment::String(val.to_string()));
                    }
                }
                _ => {}
            }
        }

        let local = if local_overflow.is_empty() {
            None
        } else {
            Some(local_overflow)
        };

        PEP440 {
            epoch,
            release,
            pre_label,
            pre_number,
            post_label,
            post_number,
            dev_label,
            dev_number,
            local,
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
    #[case(with_version(1, 2, 3), "1.2.3")]
    // With epoch
    #[case({
        let mut zerv = with_version(1, 2, 3);
        zerv.format.extra_core.push(Component::VarField("epoch".to_string()));
        zerv.vars.epoch = Some(2);
        zerv
    }, "2!1.2.3")]
    // With pre-release
    #[case({
        let mut zerv = with_version(1, 2, 3);
        zerv.format.extra_core.push(Component::VarField("pre_release".to_string()));
        zerv.vars.pre_release = Some(crate::version::zerv::PreReleaseVar {
            label: PreReleaseLabel::Alpha,
            number: Some(1),
        });
        zerv
    }, "1.2.3a1")]
    // With post
    #[case({
        let mut zerv = with_version(1, 2, 3);
        zerv.format.extra_core.push(Component::VarField("post".to_string()));
        zerv.vars.post = Some(1);
        zerv
    }, "1.2.3.post1")]
    // With dev
    #[case({
        let mut zerv = with_version(1, 2, 3);
        zerv.format.extra_core.push(Component::VarField("dev".to_string()));
        zerv.vars.dev = Some(1);
        zerv
    }, "1.2.3.dev1")]
    // With local from build
    #[case({
        let mut zerv = with_version(1, 2, 3);
        zerv.format.build = vec![
            Component::String("ubuntu".to_string()),
            Component::Integer(20),
            Component::Integer(4),
        ];
        zerv
    }, "1.2.3+ubuntu.20.4")]
    // Complex version with all components
    #[case({
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
    }, "2!1.2.3a1.post1.dev1+local.1")]
    fn test_zerv_to_pep440_conversion(#[case] zerv: Zerv, #[case] expected_pep440_str: &str) {
        let pep440: PEP440 = zerv.into();
        assert_eq!(pep440.to_string(), expected_pep440_str);
    }
}
