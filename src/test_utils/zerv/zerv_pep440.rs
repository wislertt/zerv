use super::common_fixtures::CommonFixtures;
use super::zerv::ZervFixture;
use crate::version::zerv::{
    Component,
    PreReleaseLabel,
    Var,
};

/// Fixtures for Zerv → PEP440 conversion (from_zerv.rs)
pub mod from {
    use super::*;

    // Base versions
    pub fn v1_2_3() -> ZervFixture {
        CommonFixtures::v1_2_3()
    }

    pub fn v1_0_0() -> ZervFixture {
        CommonFixtures::v1_0_0()
    }

    pub fn v1_0_0_tier3() -> ZervFixture {
        v1_0_0().with_standard_tier_3()
    }

    // v1.2.3 variants
    pub fn v1_2_3_e2() -> ZervFixture {
        CommonFixtures::v1_2_3_e2()
    }

    pub fn v1_2_3_a1() -> ZervFixture {
        CommonFixtures::v1_2_3_a1()
    }

    pub fn v1_2_3_post1() -> ZervFixture {
        CommonFixtures::v1_2_3_post1()
    }

    pub fn v1_2_3_dev1() -> ZervFixture {
        v1_2_3().with_standard_tier_3().with_dev(1)
    }

    // v1.0.0 variants
    pub fn v1_0_0_e1() -> ZervFixture {
        CommonFixtures::v1_0_0_e1()
    }

    pub fn v1_0_0_e2() -> ZervFixture {
        CommonFixtures::v1_0_0_e2()
    }

    pub fn v1_0_0_e3() -> ZervFixture {
        CommonFixtures::v1_0_0_e3()
    }

    pub fn v1_0_0_post1() -> ZervFixture {
        CommonFixtures::v1_0_0_post1()
    }

    pub fn v1_0_0_post5() -> ZervFixture {
        CommonFixtures::v1_0_0_post5()
    }

    pub fn v1_0_0_dev0() -> ZervFixture {
        v1_0_0_tier3().with_dev(0)
    }

    pub fn v1_0_0_dev10() -> ZervFixture {
        v1_0_0_tier3().with_dev(10)
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

    pub fn v1_0_0_post1_dev2() -> ZervFixture {
        v1_0_0_tier3().with_post(1).with_dev(2)
    }

    pub fn v1_0_0_a1_post2() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
    }

    pub fn v1_0_0_b3_post1() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Beta, Some(3))
            .with_post(1)
    }

    pub fn v1_0_0_rc2_post5() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, Some(2))
            .with_post(5)
    }

    pub fn v1_0_0_a1_dev2() -> ZervFixture {
        v1_0_0_tier3()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_dev(2)
    }

    pub fn v1_0_0_b2_dev1() -> ZervFixture {
        v1_0_0_tier3()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_dev(1)
    }

    pub fn v1_0_0_rc1_dev3() -> ZervFixture {
        v1_0_0_tier3()
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_dev(3)
    }

    // Triple combinations
    pub fn v1_0_0_a1_post2_dev3() -> ZervFixture {
        v1_0_0_tier3()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
            .with_dev(3)
    }

    pub fn v1_0_0_b2_post3_dev1() -> ZervFixture {
        v1_0_0_tier3()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_post(3)
            .with_dev(1)
    }

    pub fn v1_0_0_rc1_post1_dev1() -> ZervFixture {
        v1_0_0_tier3()
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_post(1)
            .with_dev(1)
    }

    // Epoch + post + dev combinations
    pub fn v1_0_0_e2_post1_dev3() -> ZervFixture {
        v1_0_0_tier3().with_epoch(2).with_post(1).with_dev(3)
    }

    pub fn v1_0_0_e1_post1_dev2() -> ZervFixture {
        v1_0_0_tier3().with_epoch(1).with_post(1).with_dev(2)
    }

    // All components together
    pub fn v1_0_0_e3_a1_post2_dev1() -> ZervFixture {
        v1_0_0_tier3()
            .with_epoch(3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
            .with_dev(1)
    }

    pub fn v1_0_0_e1_b2_post1_dev3() -> ZervFixture {
        v1_0_0_tier3()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_post(1)
            .with_dev(3)
    }

    // Build metadata fixtures
    pub fn v1_0_0_e1_build() -> ZervFixture {
        CommonFixtures::v1_0_0_e1_build()
    }

    pub fn v1_0_0_post1_build() -> ZervFixture {
        CommonFixtures::v1_0_0_post1_build()
    }

    pub fn v1_0_0_dev2_build() -> ZervFixture {
        v1_0_0_tier3()
            .with_dev(2)
            .with_build(Component::Str("build".to_string()))
            .with_build(Component::Int(789))
    }

    pub fn v1_0_0_e2_a1_build() -> ZervFixture {
        CommonFixtures::v1_0_0_e2_a1_build()
    }

    pub fn v1_0_0_complex_build() -> ZervFixture {
        CommonFixtures::v1_0_0_complex_build()
    }

    pub fn v1_0_0_e1_a1_post1_dev1_complex() -> ZervFixture {
        v1_0_0_tier3()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(1)
            .with_dev(1)
            .with_build(Component::Str("complex".to_string()))
            .with_build(Component::Str("local".to_string()))
            .with_build(Component::Int(456))
    }

    // VarField build metadata
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

    // Complex v1.2.3 build
    pub fn v1_2_3_ubuntu_build() -> ZervFixture {
        CommonFixtures::v1_2_3_ubuntu_build()
    }

    pub fn v1_2_3_e2_a1_post1_dev1_local() -> ZervFixture {
        v1_2_3()
            .with_standard_tier_3()
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(1)
            .with_dev(1)
            .with_build(Component::Str("local".to_string()))
            .with_build(Component::Int(1))
    }

    // Custom field variant - build
    pub fn v1_0_0_custom_build_field(value: &str) -> ZervFixture {
        CommonFixtures::v1_0_0_custom_build_field(value)
    }

    // Custom field variant - core
    pub fn v1_0_0_custom_core_field(value: &str) -> ZervFixture {
        CommonFixtures::v1_0_0_custom_core_field(value)
    }

    // Custom field variant - extra_core
    pub fn v1_0_0_custom_extra_field(value: &str) -> ZervFixture {
        CommonFixtures::v1_0_0_custom_extra_field(value)
    }

    // Test case for duplicate epoch handling - second epoch should go to local
    pub fn v1_0_0_duplicate_epoch() -> ZervFixture {
        v1_0_0()
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_extra_core(Component::Int(1))
            .with_extra_core(Component::Int(2))
            .with_extra_core(Component::Int(3))
            .with_extra_core(Component::Str("epoch".to_string()))
            .with_extra_core(Component::Str("epoch".to_string()))
    }

    // Maximum complexity fixture - PEP440 version (no dev component)
    pub fn v2_3_4_max_complexity() -> ZervFixture {
        CommonFixtures::v2_3_4_max_complexity()
    }
}

/// Fixtures for PEP440 → Zerv conversion (to_zerv.rs)
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
            .with_extra_core_components(vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
                Component::Var(Var::Post),
                Component::Var(Var::Dev),
            ])
    }

    fn v1_0_0_base() -> ZervFixture {
        base_schema().with_version(1, 0, 0)
    }

    fn v1_2_3_base() -> ZervFixture {
        base_schema().with_version(1, 2, 3)
    }

    // Basic conversions
    pub fn v1_2_3() -> ZervFixture {
        v1_2_3_base()
    }

    pub fn v1_2_3_e2() -> ZervFixture {
        v1_2_3_base().with_epoch(2)
    }

    pub fn v1_2_3_a1() -> ZervFixture {
        v1_2_3_base().with_pre_release(PreReleaseLabel::Alpha, Some(1))
    }

    pub fn v1_2_3_post1() -> ZervFixture {
        v1_2_3_base().with_post(1)
    }

    pub fn v1_2_3_dev1() -> ZervFixture {
        v1_2_3_base().with_dev(1)
    }

    pub fn v1_2_3_ubuntu_build() -> ZervFixture {
        v1_2_3_base().with_build_components(vec![
            Component::Str("ubuntu".to_string()),
            Component::Int(20),
            Component::Int(4),
        ])
    }

    pub fn v1_2_3_e2_a1_post1_dev1_local() -> ZervFixture {
        v1_2_3_base()
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(1)
            .with_dev(1)
            .with_build_components(vec![Component::Str("local".to_string()), Component::Int(1)])
    }

    // v1.0.0 variants
    pub fn v1_0_0() -> ZervFixture {
        v1_0_0_base()
    }

    pub fn v1_0_0_e1() -> ZervFixture {
        v1_0_0_base().with_epoch(1)
    }

    pub fn v1_0_0_e5() -> ZervFixture {
        v1_0_0_base().with_epoch(5)
    }

    pub fn v1_0_0_e999() -> ZervFixture {
        v1_0_0_base().with_epoch(999)
    }

    pub fn v1_0_0_post0() -> ZervFixture {
        v1_0_0_base().with_post(0)
    }

    pub fn v1_0_0_e4_a0() -> ZervFixture {
        v1_0_0_base()
            .with_epoch(4)
            .with_pre_release(PreReleaseLabel::Alpha, Some(0))
    }

    pub fn v1_0_0_post5() -> ZervFixture {
        v1_0_0_base().with_post(5)
    }

    pub fn v1_0_0_dev0() -> ZervFixture {
        v1_0_0_base().with_dev(0)
    }

    pub fn v1_0_0_dev10() -> ZervFixture {
        v1_0_0_base().with_dev(10)
    }

    pub fn v1_0_0_e2_a1() -> ZervFixture {
        v1_0_0_base()
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
    }

    pub fn v1_0_0_e3_b2() -> ZervFixture {
        v1_0_0_base()
            .with_epoch(3)
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
    }

    pub fn v1_0_0_e1_rc5() -> ZervFixture {
        v1_0_0_base()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Rc, Some(5))
    }

    pub fn v1_0_0_post1_dev2() -> ZervFixture {
        v1_0_0_base().with_post(1).with_dev(2)
    }

    pub fn v1_0_0_a1_post2() -> ZervFixture {
        v1_0_0_base()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
    }

    pub fn v1_0_0_b3_post1() -> ZervFixture {
        v1_0_0_base()
            .with_pre_release(PreReleaseLabel::Beta, Some(3))
            .with_post(1)
    }

    pub fn v1_0_0_rc2_post5() -> ZervFixture {
        v1_0_0_base()
            .with_pre_release(PreReleaseLabel::Rc, Some(2))
            .with_post(5)
    }

    pub fn v1_0_0_a1_dev2() -> ZervFixture {
        v1_0_0_base()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_dev(2)
    }

    pub fn v1_0_0_b2_dev1() -> ZervFixture {
        v1_0_0_base()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_dev(1)
    }

    pub fn v1_0_0_rc1_dev3() -> ZervFixture {
        v1_0_0_base()
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_dev(3)
    }

    // Triple combinations
    pub fn v1_0_0_a1_post2_dev3() -> ZervFixture {
        v1_0_0_base()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
            .with_dev(3)
    }

    pub fn v1_0_0_b2_post3_dev1() -> ZervFixture {
        v1_0_0_base()
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_post(3)
            .with_dev(1)
    }

    pub fn v1_0_0_rc1_post1_dev1() -> ZervFixture {
        v1_0_0_base()
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_post(1)
            .with_dev(1)
    }

    // Epoch + post + dev combinations
    pub fn v1_0_0_e2_post1_dev3() -> ZervFixture {
        v1_0_0_base().with_epoch(2).with_post(1).with_dev(3)
    }

    pub fn v1_0_0_e1_post1_dev2() -> ZervFixture {
        v1_0_0_base().with_epoch(1).with_post(1).with_dev(2)
    }

    // All components together
    pub fn v1_0_0_e3_a1_post2_dev1() -> ZervFixture {
        v1_0_0_base()
            .with_epoch(3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
            .with_dev(1)
    }

    pub fn v1_0_0_e1_b2_post1_dev3() -> ZervFixture {
        v1_0_0_base()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
            .with_post(1)
            .with_dev(3)
    }

    // Build metadata fixtures
    pub fn v1_0_0_e1_build() -> ZervFixture {
        v1_0_0_e1().with_build_components(vec![
            Component::Str("build".to_string()),
            Component::Int(123),
        ])
    }

    pub fn v1_0_0_post1_build() -> ZervFixture {
        v1_0_0_base().with_post(1).with_build_components(vec![
            Component::Str("build".to_string()),
            Component::Int(456),
        ])
    }

    pub fn v1_0_0_dev2_build() -> ZervFixture {
        v1_0_0_base().with_dev(2).with_build_components(vec![
            Component::Str("build".to_string()),
            Component::Int(789),
        ])
    }

    pub fn v1_0_0_e2_a1_build() -> ZervFixture {
        v1_0_0_e2_a1().with_build_components(vec![
            Component::Str("build".to_string()),
            Component::Str("abc".to_string()),
        ])
    }

    pub fn v1_0_0_complex_build() -> ZervFixture {
        v1_0_0_base().with_build_components(vec![
            Component::Str("foo".to_string()),
            Component::Str("bar".to_string()),
            Component::Int(123),
        ])
    }

    pub fn v1_0_0_e1_a1_post1_dev1_complex() -> ZervFixture {
        v1_0_0_base()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(1)
            .with_dev(1)
            .with_build_components(vec![
                Component::Str("complex".to_string()),
                Component::Str("local".to_string()),
                Component::Int(456),
            ])
    }
}
