use super::{LocalSegment, PEP440};
use crate::constants::ron_fields;
use crate::version::zerv::{Component, PreReleaseVar, Zerv, ZervSchema, ZervVars};

impl From<PEP440> for Zerv {
    fn from(pep440: PEP440) -> Self {
        let mut extra_core = Vec::new();
        let mut build = Vec::new();

        // Add epoch to extra_core if non-zero
        if pep440.epoch > 0 {
            extra_core.push(Component::VarField(ron_fields::EPOCH.to_string()));
        }

        // Add pre-release to extra_core if present
        let pre_release = if let (Some(label), number) = (pep440.pre_label, pep440.pre_number) {
            extra_core.push(Component::VarField(ron_fields::PRE_RELEASE.to_string()));
            Some(PreReleaseVar {
                label,
                number: number.map(|n| n as u64),
            })
        } else {
            None
        };

        // Add post to extra_core if present
        let post = if pep440.post_label.is_some() {
            extra_core.push(Component::VarField(ron_fields::POST.to_string()));
            pep440.post_number.map(|n| n as u64)
        } else {
            None
        };

        // Add dev to extra_core if present
        let dev = if pep440.dev_label.is_some() {
            extra_core.push(Component::VarField(ron_fields::DEV.to_string()));
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
                    Component::VarField(ron_fields::MAJOR.to_string()),
                    Component::VarField(ron_fields::MINOR.to_string()),
                    Component::VarField(ron_fields::PATCH.to_string()),
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
    use crate::test_utils::zerv::*;

    use rstest::rstest;

    #[rstest]
    // Basic conversions
    #[case("1.2.3", ZervFixture::pep_zerv_1_2_3())]
    #[case("2!1.2.3", ZervFixture::pep_zerv_1_2_3_epoch_2())]
    #[case("1.2.3a1", ZervFixture::pep_zerv_1_2_3_alpha_1())]
    #[case("1.2.3.post1", ZervFixture::pep_zerv_1_2_3_post_1())]
    #[case("1.2.3.dev1", ZervFixture::pep_zerv_1_2_3_dev_1())]
    #[case("1.2.3+ubuntu.20.4", ZervFixture::pep_zerv_1_2_3_ubuntu_build())]
    #[case(
        "2!1.2.3a1.post1.dev1+local.1",
        ZervFixture::pep_zerv_complex_2_1_2_3_alpha_1_post_1_dev_1_local_1()
    )]
    // Epoch handling
    #[case("1!1.0.0", ZervFixture::pep_zerv_1_0_0_epoch_1())]
    #[case("5!1.0.0", ZervFixture::pep_zerv_1_0_0_epoch_5())]
    #[case("999!1.0.0", ZervFixture::pep_zerv_1_0_0_epoch_999())]
    // Post handling
    #[case("1.0.0.post5", ZervFixture::pep_zerv_1_0_0_post_5())]
    #[case("1.0.0.post0", ZervFixture::pep_zerv_1_0_0_post_0())]
    // Dev handling
    #[case("1.0.0.dev0", ZervFixture::pep_zerv_1_0_0_dev_0())]
    #[case("1.0.0.dev10", ZervFixture::pep_zerv_1_0_0_dev_10())]
    // Epoch + pre-release combinations
    #[case("2!1.0.0a1", ZervFixture::pep_zerv_1_0_0_epoch_2_alpha_1())]
    #[case("3!1.0.0b2", ZervFixture::pep_zerv_1_0_0_epoch_3_beta_2())]
    #[case("1!1.0.0rc5", ZervFixture::pep_zerv_1_0_0_epoch_1_rc_5())]
    #[case("4!1.0.0a0", ZervFixture::pep_zerv_1_0_0_epoch_4_alpha())]
    // Post + dev combinations
    #[case("1.0.0.post1.dev2", ZervFixture::pep_zerv_1_0_0_post_1_dev_2())]
    // Pre-release + post combinations
    #[case("1.0.0a1.post2", ZervFixture::pep_zerv_1_0_0_alpha_1_post_2())]
    #[case("1.0.0b3.post1", ZervFixture::pep_zerv_1_0_0_beta_3_post_1())]
    #[case("1.0.0rc2.post5", ZervFixture::pep_zerv_1_0_0_rc_2_post_5())]
    // Pre-release + dev combinations
    #[case("1.0.0a1.dev2", ZervFixture::pep_zerv_1_0_0_alpha_1_dev_2())]
    #[case("1.0.0b2.dev1", ZervFixture::pep_zerv_1_0_0_beta_2_dev_1())]
    #[case("1.0.0rc1.dev3", ZervFixture::pep_zerv_1_0_0_rc_1_dev_3())]
    // Triple combinations
    #[case(
        "1.0.0a1.post2.dev3",
        ZervFixture::pep_zerv_1_0_0_alpha_1_post_2_dev_3()
    )]
    #[case(
        "1.0.0b2.post3.dev1",
        ZervFixture::pep_zerv_1_0_0_beta_2_post_3_dev_1()
    )]
    #[case("1.0.0rc1.post1.dev1", ZervFixture::pep_zerv_1_0_0_rc_1_post_1_dev_1())]
    // Epoch + post + dev combinations
    #[case(
        "2!1.0.0.post1.dev3",
        ZervFixture::pep_zerv_1_0_0_epoch_2_post_1_dev_3()
    )]
    #[case(
        "1!1.0.0.post1.dev2",
        ZervFixture::pep_zerv_1_0_0_epoch_1_post_1_dev_2()
    )]
    // All components together
    #[case(
        "3!1.0.0a1.post2.dev1",
        ZervFixture::pep_zerv_1_0_0_epoch_3_alpha_1_post_2_dev_1()
    )]
    #[case(
        "1!1.0.0b2.post1.dev3",
        ZervFixture::pep_zerv_1_0_0_epoch_1_beta_2_post_1_dev_3()
    )]
    // With build metadata
    #[case("1!1.0.0+build.123", ZervFixture::pep_zerv_1_0_0_epoch_1_build())]
    #[case("1.0.0.post1+build.456", ZervFixture::pep_zerv_1_0_0_post_1_build())]
    #[case("1.0.0.dev2+build.789", ZervFixture::pep_zerv_1_0_0_dev_2_build())]
    #[case(
        "2!1.0.0a1+build.abc",
        ZervFixture::pep_zerv_1_0_0_epoch_2_alpha_1_build()
    )]
    // Complex local version identifiers
    #[case("1.0.0+foo.bar.123", ZervFixture::pep_zerv_1_0_0_complex_local())]
    #[case(
        "1!1.0.0a1.post1.dev1+complex.local.456",
        ZervFixture::pep_zerv_1_0_0_all_components_complex_local()
    )]
    fn test_pep440_to_zerv_conversion(#[case] pep440_str: &str, #[case] expected: Zerv) {
        let pep440: PEP440 = pep440_str.parse().unwrap();
        let zerv: Zerv = pep440.into();
        assert_eq!(zerv, expected);
    }

    #[rstest]
    #[case("1.0.0")]
    #[case("2!1.2.3")]
    #[case("1.0.0a1")]
    #[case("1.0.0.post1")]
    #[case("1.0.0.dev1")]
    #[case("1.0.0+local.1")]
    #[case("2!1.2.3a1.post1.dev1+local.1")]
    fn test_round_trip_conversion(#[case] version_str: &str) {
        let original: PEP440 = version_str.parse().unwrap();
        let zerv: Zerv = original.clone().into();
        let converted: PEP440 = zerv.into();

        assert_eq!(original.to_string(), converted.to_string());
    }
}
