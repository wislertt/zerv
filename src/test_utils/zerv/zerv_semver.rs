use super::ZervFixture;
use super::common_fixtures::CommonFixtures;
use crate::version::zerv::{
    Component,
    PreReleaseLabel,
    Var,
};

/// Fixtures for SemVer → Zerv conversion (to_zerv.rs)
pub mod to {
    use super::*;

    fn base_schema() -> ZervFixture {
        ZervFixture::new()
            .with_empty_schema()
            .with_core_components(vec![
                Component::Var(Var::Major),
                Component::Var(Var::Minor),
                Component::Var(Var::Patch),
            ])
    }

    fn v1_0_0() -> ZervFixture {
        base_schema().with_version(1, 0, 0)
    }

    pub fn v1_2_3() -> ZervFixture {
        base_schema().with_version(1, 2, 3)
    }

    pub fn v1_0_0_a1() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_alpha_1() -> ZervFixture {
        v1_0_0_a1()
    }

    pub fn v1_0_0_epoch_1() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_extra_core_components(vec![Component::Var(Var::Epoch)])
    }

    pub fn v1_0_0_post_1() -> ZervFixture {
        v1_0_0()
            .with_post(1)
            .with_extra_core_components(vec![Component::Var(Var::Post)])
    }

    pub fn v1_0_0_dev_1() -> ZervFixture {
        v1_0_0()
            .with_dev(1)
            .with_extra_core_components(vec![Component::Var(Var::Dev)])
    }

    pub fn v1_0_0_something_1() -> ZervFixture {
        v1_0_0().with_extra_core_components(vec![
            Component::Str("something".to_string()),
            Component::Int(1),
        ])
    }

    pub fn v1_0_0_build() -> ZervFixture {
        v1_0_0().with_build_components(vec![
            Component::Str("build".to_string()),
            Component::Int(123),
        ])
    }

    pub fn v1_0_0_a1_build() -> ZervFixture {
        v1_0_0_a1().with_build_components(vec![
            Component::Str("build".to_string()),
            Component::Int(123),
        ])
    }

    pub fn v1_0_0_a1_complex() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Str("lowercase".to_string()),
                Component::Int(4),
                Component::Str("UPPERCASE".to_string()),
                Component::Int(5),
            ])
            .with_build_components(vec![
                Component::Str("build".to_string()),
                Component::Int(123),
            ])
    }

    // Case variations
    pub fn v1_0_0_beta_2() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_rc_3() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, Some(3))
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_preview_4() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, Some(4))
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_a_1() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_b_2() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_c_3() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, Some(3))
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_alpha() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, None)
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_beta() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, None)
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_rc() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, None)
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_alpha_0() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(0))
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    pub fn v1_0_0_beta_0() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, Some(0))
            .with_extra_core_components(vec![Component::Var(Var::PreRelease)])
    }

    // Epoch variants
    pub fn v1_0_0_epoch_5() -> ZervFixture {
        v1_0_0()
            .with_epoch(5)
            .with_extra_core_components(vec![Component::Var(Var::Epoch)])
    }

    pub fn v1_0_0_epoch_0() -> ZervFixture {
        v1_0_0()
            .with_epoch(0)
            .with_extra_core_components(vec![Component::Var(Var::Epoch)])
    }

    pub fn v1_0_0_epoch_999() -> ZervFixture {
        v1_0_0()
            .with_epoch(999)
            .with_extra_core_components(vec![Component::Var(Var::Epoch)])
    }

    // Post variants
    pub fn v1_0_0_post_5() -> ZervFixture {
        v1_0_0()
            .with_post(5)
            .with_extra_core_components(vec![Component::Var(Var::Post)])
    }

    pub fn v1_0_0_post_0() -> ZervFixture {
        v1_0_0()
            .with_post(0)
            .with_extra_core_components(vec![Component::Var(Var::Post)])
    }

    // Dev variants
    pub fn v1_0_0_dev_0() -> ZervFixture {
        v1_0_0()
            .with_dev(0)
            .with_extra_core_components(vec![Component::Var(Var::Dev)])
    }

    pub fn v1_0_0_dev_10() -> ZervFixture {
        v1_0_0()
            .with_dev(10)
            .with_extra_core_components(vec![Component::Var(Var::Dev)])
    }

    // Complex combinations
    pub fn v1_0_0_epoch_2_alpha_1() -> ZervFixture {
        v1_0_0()
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
            ])
    }

    pub fn v1_0_0_epoch_3_beta_2() -> ZervFixture {
        v1_0_0()
            .with_epoch(3)
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
            ])
    }

    pub fn v1_0_0_epoch_1_rc_5() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Rc, Some(5))
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
            ])
    }

    pub fn v1_0_0_epoch_4_alpha() -> ZervFixture {
        v1_0_0()
            .with_epoch(4)
            .with_pre_release(PreReleaseLabel::Alpha, None)
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
            ])
    }

    pub fn v1_0_0_post_1_dev_2() -> ZervFixture {
        v1_0_0()
            .with_post(1)
            .with_dev(2)
            .with_extra_core_components(vec![Component::Var(Var::Post), Component::Var(Var::Dev)])
    }

    // Build metadata combinations
    pub fn v1_0_0_epoch_1_build() -> ZervFixture {
        v1_0_0_epoch_1().with_build_components(vec![
            Component::Str("build".to_string()),
            Component::Int(123),
        ])
    }

    pub fn v1_0_0_post_1_build() -> ZervFixture {
        v1_0_0_post_1().with_build_components(vec![
            Component::Str("build".to_string()),
            Component::Int(456),
        ])
    }

    pub fn v1_0_0_dev_2_build() -> ZervFixture {
        v1_0_0()
            .with_dev(2)
            .with_extra_core_components(vec![Component::Var(Var::Dev)])
            .with_build_components(vec![
                Component::Str("build".to_string()),
                Component::Int(789),
            ])
    }

    pub fn v1_0_0_epoch_2_alpha_1_build() -> ZervFixture {
        v1_0_0_epoch_2_alpha_1().with_build_components(vec![
            Component::Str("build".to_string()),
            Component::Str("abc".to_string()),
        ])
    }

    // Complex custom cases
    pub fn v1_0_0_foo_bar_beta_2_baz() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_extra_core_components(vec![
                Component::Str("foo".to_string()),
                Component::Str("bar".to_string()),
                Component::Var(Var::PreRelease),
                Component::Str("baz".to_string()),
            ])
    }

    pub fn v1_0_0_alpha_1_beta_2() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Str("beta".to_string()),
                Component::Int(2),
            ])
    }

    pub fn v1_0_0_rc_1_alpha_2_beta_3() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Str("alpha".to_string()),
                Component::Int(2),
                Component::Str("beta".to_string()),
                Component::Int(3),
            ])
    }

    pub fn v1_0_0_pre_alpha_1() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, None)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Str("alpha".to_string()),
                Component::Int(1),
            ])
    }

    pub fn v1_0_0_test_alpha_beta_rc_1() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, None)
            .with_extra_core_components(vec![
                Component::Str("test".to_string()),
                Component::Var(Var::PreRelease),
                Component::Str("beta".to_string()),
                Component::Str("rc".to_string()),
                Component::Int(1),
            ])
    }

    pub fn v1_0_0_foo_1_alpha() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, None)
            .with_extra_core_components(vec![
                Component::Str("foo".to_string()),
                Component::Int(1),
                Component::Var(Var::PreRelease),
            ])
    }

    pub fn v1_0_0_bar_2_beta() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, None)
            .with_extra_core_components(vec![
                Component::Str("bar".to_string()),
                Component::Int(2),
                Component::Var(Var::PreRelease),
            ])
    }

    pub fn v1_0_0_dev_3_post_4() -> ZervFixture {
        v1_0_0()
            .with_post(4)
            .with_dev(3)
            .with_extra_core_components(vec![Component::Var(Var::Dev), Component::Var(Var::Post)])
    }

    pub fn v1_0_0_alpha_1_post_2() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Var(Var::Post),
            ])
    }

    pub fn v1_0_0_beta_3_post_1() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, Some(3))
            .with_post(1)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Var(Var::Post),
            ])
    }

    pub fn v1_0_0_rc_2_post_5() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, Some(2))
            .with_post(5)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Var(Var::Post),
            ])
    }

    pub fn v1_0_0_alpha_1_dev_2() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_dev(2)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Var(Var::Dev),
            ])
    }

    pub fn v1_0_0_beta_2_dev_1() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_dev(1)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Var(Var::Dev),
            ])
    }

    pub fn v1_0_0_rc_1_dev_3() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_dev(3)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Var(Var::Dev),
            ])
    }

    pub fn v1_0_0_alpha_1_post_2_dev_3() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
            .with_dev(3)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Var(Var::Post),
                Component::Var(Var::Dev),
            ])
    }

    pub fn v1_0_0_beta_2_dev_1_post_3() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_post(3)
            .with_dev(1)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Var(Var::Dev),
                Component::Var(Var::Post),
            ])
    }

    pub fn v1_0_0_rc_1_post_1_dev_1() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_post(1)
            .with_dev(1)
            .with_extra_core_components(vec![
                Component::Var(Var::PreRelease),
                Component::Var(Var::Post),
                Component::Var(Var::Dev),
            ])
    }

    pub fn v1_0_0_epoch_2_post_1_dev_3() -> ZervFixture {
        v1_0_0()
            .with_epoch(2)
            .with_post(1)
            .with_dev(3)
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::Post),
                Component::Var(Var::Dev),
            ])
    }

    pub fn v1_0_0_epoch_1_dev_2_post_1() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_post(1)
            .with_dev(2)
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::Dev),
                Component::Var(Var::Post),
            ])
    }

    pub fn v1_0_0_epoch_3_alpha_1_post_2_dev_1() -> ZervFixture {
        v1_0_0()
            .with_epoch(3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
            .with_dev(1)
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
                Component::Var(Var::Post),
                Component::Var(Var::Dev),
            ])
    }

    pub fn v1_0_0_epoch_1_beta_2_dev_3_post_1() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_post(1)
            .with_dev(3)
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
                Component::Var(Var::Dev),
                Component::Var(Var::Post),
            ])
    }

    pub fn v1_0_0_foo_epoch_1_alpha_2() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(2))
            .with_extra_core_components(vec![
                Component::Str("foo".to_string()),
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
            ])
    }

    pub fn v1_0_0_epoch_1_foo_post_2() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_post(2)
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Str("foo".to_string()),
                Component::Var(Var::Post),
            ])
    }

    pub fn v1_0_0_bar_dev_1_epoch_2() -> ZervFixture {
        v1_0_0()
            .with_epoch(2)
            .with_dev(1)
            .with_extra_core_components(vec![
                Component::Str("bar".to_string()),
                Component::Var(Var::Dev),
                Component::Var(Var::Epoch),
            ])
    }

    pub fn v1_0_0_duplicate_vars() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)  // First epoch wins
            .with_post(3)   // First post wins
            .with_dev(5)    // First dev wins
            .with_pre_release(PreReleaseLabel::Alpha, Some(7))  // First alpha wins
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),     // epoch.1 -> Var(Epoch)
                Component::Str("epoch".to_string()),  // epoch.2 -> Str("epoch"), Int(2)
                Component::Int(2),
                Component::Var(Var::Post),      // post.3 -> Var(Post)
                Component::Str("post".to_string()),   // post.4 -> Str("post"), Int(4)
                Component::Int(4),
                Component::Var(Var::Dev),       // dev.5 -> Var(Dev)
                Component::Str("dev".to_string()),    // dev.6 -> Str("dev"), Int(6)
                Component::Int(6),
                Component::Var(Var::PreRelease), // alpha.7 -> Var(PreRelease)
                Component::Str("alpha".to_string()),  // alpha.8 -> Str("alpha"), Int(8)
                Component::Int(8),
            ])
    }

    // Test case for duplicate vars without numbers: "1.0.0-epoch.epoch.rc.rc.post.post.dev.dev"
    pub fn v1_0_0_duplicate_vars_without_num() -> ZervFixture {
        base_schema()
            .with_version(1, 0, 0)
            .with_pre_release(PreReleaseLabel::Rc, None)
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Str("epoch".to_string()),
                Component::Var(Var::PreRelease),
                Component::Str("rc".to_string()),
                Component::Var(Var::Post),
                Component::Str("post".to_string()),
                Component::Var(Var::Dev),
                Component::Str("dev".to_string()),
            ])
    }

    // Complex duplicate case: "1.2.3-10.a.rc.epoch.rc.3"
    pub fn v1_2_3_complex_duplicate() -> ZervFixture {
        base_schema()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, None)
            .with_extra_core_components(vec![
                Component::Int(10),
                Component::Var(Var::PreRelease),
                Component::Str("rc".to_string()),
                Component::Var(Var::Epoch),
                Component::Str("rc".to_string()),
                Component::Int(3),
            ])
    }
}

/// Fixtures for Zerv → SemVer conversion (from_zerv.rs)
pub mod from {
    use super::*;

    // Common fixtures - direct delegation
    pub fn v1_2_3() -> ZervFixture {
        CommonFixtures::v1_2_3()
    }
    pub fn v1_0_0() -> ZervFixture {
        CommonFixtures::v1_0_0()
    }
    pub fn v1_0_0_a1() -> ZervFixture {
        CommonFixtures::v1_0_0_a1()
    }
    pub fn v1_0_0_b2() -> ZervFixture {
        CommonFixtures::v1_0_0_b2()
    }
    pub fn v1_0_0_rc3() -> ZervFixture {
        CommonFixtures::v1_0_0_rc3()
    }
    pub fn v1_0_0_a_none() -> ZervFixture {
        CommonFixtures::v1_0_0_a_none()
    }
    pub fn v1_0_0_b_none() -> ZervFixture {
        CommonFixtures::v1_0_0_b_none()
    }
    pub fn v1_0_0_rc_none() -> ZervFixture {
        CommonFixtures::v1_0_0_rc_none()
    }
    pub fn v1_0_0_a0() -> ZervFixture {
        CommonFixtures::v1_0_0_a0()
    }
    pub fn v1_0_0_b0() -> ZervFixture {
        CommonFixtures::v1_0_0_b0()
    }
    pub fn v1_0_0_build() -> ZervFixture {
        CommonFixtures::v1_0_0_build()
    }
    pub fn v1_0_0_a1_build() -> ZervFixture {
        CommonFixtures::v1_0_0_a1_build()
    }
    pub fn v1_0_0_e1() -> ZervFixture {
        CommonFixtures::v1_0_0_e1()
    }
    pub fn v1_0_0_e1_build() -> ZervFixture {
        CommonFixtures::v1_0_0_e1_build()
    }
    pub fn v1_0_0_post1_build() -> ZervFixture {
        CommonFixtures::v1_0_0_post1_build()
    }
    pub fn v1_0_0_e2_a1_build() -> ZervFixture {
        CommonFixtures::v1_0_0_e2_a1_build()
    }
    pub fn v1_0_0_post1() -> ZervFixture {
        CommonFixtures::v1_0_0_post1()
    }
    pub fn v1_0_0_post5() -> ZervFixture {
        CommonFixtures::v1_0_0_post5()
    }
    pub fn v1_0_0_post0() -> ZervFixture {
        CommonFixtures::v1_0_0_post0()
    }
    pub fn v1_0_0_e2_a1() -> ZervFixture {
        CommonFixtures::v1_0_0_e2_a1()
    }
    pub fn v1_0_0_e3_b2() -> ZervFixture {
        CommonFixtures::v1_0_0_e3_b2()
    }
    pub fn v1_0_0_e1_rc5() -> ZervFixture {
        CommonFixtures::v1_0_0_e1_rc5()
    }
    pub fn v1_0_0_e4_a_none() -> ZervFixture {
        CommonFixtures::v1_0_0_e4_a_none()
    }
    pub fn v1_0_0_branch_dev() -> ZervFixture {
        CommonFixtures::v1_0_0_branch_dev()
    }
    pub fn v1_0_0_distance_5() -> ZervFixture {
        CommonFixtures::v1_0_0_distance_5()
    }
    pub fn v1_0_0_commit_abc123() -> ZervFixture {
        CommonFixtures::v1_0_0_commit_abc123()
    }
    pub fn v1_0_0_branch_distance_commit() -> ZervFixture {
        CommonFixtures::v1_0_0_branch_distance_commit()
    }
    pub fn v1_0_0_custom_core_field(value: &str) -> ZervFixture {
        CommonFixtures::v1_0_0_custom_core_field(value)
    }
    pub fn v1_0_0_custom_extra_field(value: &str) -> ZervFixture {
        CommonFixtures::v1_0_0_custom_extra_field(value)
    }
    pub fn v1_0_0_custom_build_field(value: &str) -> ZervFixture {
        CommonFixtures::v1_0_0_custom_build_field(value)
    }

    // SemVer-specific fixtures only
    pub fn v1_0_0_extra_something() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("something".to_string()))
            .with_extra_core(Component::Int(1))
    }

    pub fn v1_0_0_foo_alpha() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("foo".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("alpha".to_string()))
    }

    pub fn v1_0_0_bar_beta() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("bar".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("beta".to_string()))
    }

    pub fn v1_0_0_e5() -> ZervFixture {
        v1_0_0().with_epoch(5)
    }

    pub fn v1_0_0_e0() -> ZervFixture {
        v1_0_0().with_epoch(0)
    }

    pub fn v1_0_0_e999() -> ZervFixture {
        v1_0_0().with_epoch(999)
    }

    pub fn v1_0_0_dev1() -> ZervFixture {
        v1_0_0().with_standard_tier_3().with_dev(1)
    }

    pub fn v1_0_0_dev0() -> ZervFixture {
        v1_0_0().with_standard_tier_3().with_dev(0)
    }

    pub fn v1_0_0_dev10() -> ZervFixture {
        v1_0_0().with_standard_tier_3().with_dev(10)
    }

    // Post + dev combinations
    pub fn v1_0_0_post1_dev2() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(2))
    }

    pub fn v1_0_0_dev3_post4() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(3))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(4))
    }

    // Pre-release + post combinations
    pub fn v1_0_0_a1_post2() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("alpha".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(2))
    }

    pub fn v1_0_0_b3_post1() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("beta".to_string()))
            .with_extra_core(Component::Int(3))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(1))
    }

    pub fn v1_0_0_rc2_post5() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("rc".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(5))
    }

    // Pre-release + dev combinations
    pub fn v1_0_0_a1_dev2() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("alpha".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(2))
    }

    pub fn v1_0_0_b2_dev1() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("beta".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(1))
    }

    pub fn v1_0_0_rc1_dev3() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("rc".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(3))
    }

    // Triple combinations
    pub fn v1_0_0_a1_post2_dev3() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("alpha".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(3))
    }

    pub fn v1_0_0_b2_dev1_post3() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("beta".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(3))
    }

    pub fn v1_0_0_rc1_post1_dev1() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("rc".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(1))
    }

    // Epoch + post + dev combinations
    pub fn v1_0_0_e2_post1_dev3() -> ZervFixture {
        v1_0_0()
            .with_epoch(2)
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(3))
    }

    pub fn v1_0_0_e1_dev2_post1() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(1))
    }

    // All components together
    pub fn v1_0_0_e3_a1_post2_dev1() -> ZervFixture {
        v1_0_0()
            .with_epoch(3)
            .with_extra_core(Component::Str("alpha".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(1))
    }

    pub fn v1_0_0_e1_b2_dev3_post1() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_extra_core(Component::Str("beta".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(3))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(1))
    }

    pub fn v1_0_0_dev2_build() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(2))
            .with_build(Component::Str("build".to_string()))
            .with_build(Component::Int(789))
    }

    // Mixed with extra core
    pub fn v1_0_0_e1_foo_a2() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_extra_core(Component::Str("foo".to_string()))
            .with_extra_core(Component::Str("alpha".to_string()))
            .with_extra_core(Component::Int(2))
    }

    pub fn v1_0_0_e1_foo_post2() -> ZervFixture {
        v1_0_0()
            .with_epoch(1)
            .with_extra_core(Component::Str("foo".to_string()))
            .with_extra_core(Component::Str("post".to_string()))
            .with_extra_core(Component::Int(2))
    }

    pub fn v1_0_0_e2_bar_dev1() -> ZervFixture {
        v1_0_0()
            .with_epoch(2)
            .with_extra_core(Component::Str("bar".to_string()))
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(1))
    }

    // Core values variants
    pub fn v1_2_0() -> ZervFixture {
        ZervFixture::new().with_core_values(vec![1, 2])
    }

    pub fn v1_2_3_4_5() -> ZervFixture {
        ZervFixture::new().with_core_values(vec![1, 2, 3, 4, 5])
    }

    // Complex combinations
    pub fn v1_0_0_a1_extra_complex() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_extra_core(Component::Str("lowercase".to_string()))
            .with_extra_core(Component::Int(4))
            .with_extra_core(Component::Str("UPPERCASE".to_string()))
            .with_extra_core(Component::Int(5))
            .with_build(Component::Str("build".to_string()))
            .with_build(Component::Int(123))
    }

    pub fn v1_0_0_foo_bar_b2_baz() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("foo".to_string()))
            .with_extra_core(Component::Str("bar".to_string()))
            .with_extra_core(Component::Str("beta".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("baz".to_string()))
    }

    pub fn v1_0_0_a1_b2() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("alpha".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("beta".to_string()))
            .with_extra_core(Component::Int(2))
    }

    pub fn v1_0_0_rc1_a2_b3() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("rc".to_string()))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Str("alpha".to_string()))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Str("beta".to_string()))
            .with_extra_core(Component::Int(3))
    }

    pub fn v1_0_0_rc_none_a1() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("rc".to_string()))
            .with_extra_core(Component::Str("alpha".to_string()))
            .with_extra_core(Component::Int(1))
    }

    pub fn v1_0_0_test_alpha_beta_rc1() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("test".to_string()))
            .with_extra_core(Component::Str("alpha".to_string()))
            .with_extra_core(Component::Str("beta".to_string()))
            .with_extra_core(Component::Str("rc".to_string()))
            .with_extra_core(Component::Int(1))
    }

    // Helper for dev2 variant
    pub fn v1_0_0_dev2() -> ZervFixture {
        v1_0_0()
            .with_extra_core(Component::Str("dev".to_string()))
            .with_extra_core(Component::Int(2))
    }

    // Override max complexity to use version with dev
    pub fn v2_3_4_max_complexity() -> ZervFixture {
        CommonFixtures::v2_3_4_max_complexity_with_dev()
    }
}
