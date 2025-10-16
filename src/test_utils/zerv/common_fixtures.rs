use super::zerv::ZervFixture;
use crate::version::zerv::{
    Component,
    PreReleaseLabel,
    Var,
};

/// Common base fixture builders shared between PEP440 and SemVer
pub struct CommonFixtures;

impl CommonFixtures {
    // Base versions
    pub fn v1_2_3() -> ZervFixture {
        ZervFixture::new().with_version(1, 2, 3)
    }

    pub fn v1_0_0() -> ZervFixture {
        ZervFixture::new().with_version(1, 0, 0)
    }

    // Epoch variants
    pub fn v1_0_0_e1() -> ZervFixture {
        Self::v1_0_0().with_epoch(1)
    }

    pub fn v1_0_0_e2() -> ZervFixture {
        Self::v1_0_0().with_epoch(2)
    }

    pub fn v1_0_0_e3() -> ZervFixture {
        Self::v1_0_0().with_epoch(3)
    }

    pub fn v1_2_3_e2() -> ZervFixture {
        Self::v1_2_3().with_epoch(2)
    }

    // Pre-release variants
    pub fn v1_2_3_a1() -> ZervFixture {
        Self::v1_2_3().with_pre_release(PreReleaseLabel::Alpha, Some(1))
    }

    pub fn v1_0_0_a1() -> ZervFixture {
        Self::v1_0_0().with_pre_release(PreReleaseLabel::Alpha, Some(1))
    }

    pub fn v1_0_0_b2() -> ZervFixture {
        Self::v1_0_0().with_pre_release(PreReleaseLabel::Beta, Some(2))
    }

    pub fn v1_0_0_rc3() -> ZervFixture {
        Self::v1_0_0().with_pre_release(PreReleaseLabel::Rc, Some(3))
    }

    pub fn v1_0_0_a_none() -> ZervFixture {
        Self::v1_0_0().with_pre_release(PreReleaseLabel::Alpha, None)
    }

    pub fn v1_0_0_b_none() -> ZervFixture {
        Self::v1_0_0().with_pre_release(PreReleaseLabel::Beta, None)
    }

    pub fn v1_0_0_rc_none() -> ZervFixture {
        Self::v1_0_0().with_pre_release(PreReleaseLabel::Rc, None)
    }

    pub fn v1_0_0_a0() -> ZervFixture {
        Self::v1_0_0().with_pre_release(PreReleaseLabel::Alpha, Some(0))
    }

    pub fn v1_0_0_b0() -> ZervFixture {
        Self::v1_0_0().with_pre_release(PreReleaseLabel::Beta, Some(0))
    }

    // Post variants
    pub fn v1_2_3_post1() -> ZervFixture {
        Self::v1_2_3().with_post(1)
    }

    pub fn v1_0_0_post1() -> ZervFixture {
        Self::v1_0_0().with_post(1)
    }

    pub fn v1_0_0_post5() -> ZervFixture {
        Self::v1_0_0().with_post(5)
    }

    pub fn v1_0_0_post0() -> ZervFixture {
        Self::v1_0_0().with_post(0)
    }

    // Epoch + pre-release combinations
    pub fn v1_0_0_e2_a1() -> ZervFixture {
        Self::v1_0_0()
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
    }

    pub fn v1_0_0_e3_b2() -> ZervFixture {
        Self::v1_0_0()
            .with_epoch(3)
            .with_pre_release(PreReleaseLabel::Beta, Some(2))
    }

    pub fn v1_0_0_e1_rc5() -> ZervFixture {
        Self::v1_0_0()
            .with_epoch(1)
            .with_pre_release(PreReleaseLabel::Rc, Some(5))
    }

    pub fn v1_0_0_e4_a_none() -> ZervFixture {
        Self::v1_0_0()
            .with_epoch(4)
            .with_pre_release(PreReleaseLabel::Alpha, None)
    }

    // Build metadata variants
    pub fn v1_0_0_build() -> ZervFixture {
        Self::v1_0_0()
            .with_build(Component::Str("build".to_string()))
            .with_build(Component::UInt(123))
    }

    pub fn v1_0_0_a1_build() -> ZervFixture {
        Self::v1_0_0_a1()
            .with_build(Component::Str("build".to_string()))
            .with_build(Component::UInt(123))
    }

    pub fn v1_0_0_e1_build() -> ZervFixture {
        Self::v1_0_0_e1()
            .with_build(Component::Str("build".to_string()))
            .with_build(Component::UInt(123))
    }

    pub fn v1_0_0_post1_build() -> ZervFixture {
        Self::v1_0_0_post1()
            .with_build(Component::Str("build".to_string()))
            .with_build(Component::UInt(456))
    }

    pub fn v1_0_0_e2_a1_build() -> ZervFixture {
        Self::v1_0_0_e2_a1()
            .with_build(Component::Str("build".to_string()))
            .with_build(Component::Str("abc".to_string()))
    }

    pub fn v1_0_0_complex_build() -> ZervFixture {
        Self::v1_0_0()
            .with_build(Component::Str("foo".to_string()))
            .with_build(Component::Str("bar".to_string()))
            .with_build(Component::UInt(123))
    }

    // VarField build metadata
    pub fn v1_0_0_branch_dev() -> ZervFixture {
        Self::v1_0_0().with_branch("dev".to_string())
    }

    pub fn v1_0_0_distance_5() -> ZervFixture {
        Self::v1_0_0().with_distance(5)
    }

    pub fn v1_0_0_commit_abc123() -> ZervFixture {
        Self::v1_0_0().with_commit_hash("abc123".to_string())
    }

    pub fn v1_0_0_branch_distance_commit() -> ZervFixture {
        Self::v1_0_0()
            .with_branch("dev".to_string())
            .with_distance(3)
            .with_commit_hash("def456".to_string())
    }

    // Complex v1.2.3 build
    pub fn v1_2_3_ubuntu_build() -> ZervFixture {
        Self::v1_2_3()
            .with_build(Component::Str("ubuntu".to_string()))
            .with_build(Component::Str("20".to_string()))
            .with_build(Component::UInt(4))
    }

    // Custom field variants
    pub fn v1_0_0_custom_build_field(value: &str) -> ZervFixture {
        let mut fixture = Self::v1_0_0()
            .with_build(Component::Var(Var::Custom(
                "custom_build_field".to_string(),
            )))
            .build();
        fixture.vars.custom = serde_json::json!({
            "custom_build_field": value
        });
        ZervFixture::from(fixture)
    }

    pub fn v1_0_0_custom_core_field(value: &str) -> ZervFixture {
        let mut fixture = Self::v1_0_0()
            .with_core(Component::Var(Var::Custom("custom_core_field".to_string())))
            .build();
        fixture.vars.custom = serde_json::json!({
            "custom_core_field": value
        });
        ZervFixture::from(fixture)
    }

    pub fn v1_0_0_custom_extra_field(value: &str) -> ZervFixture {
        let mut fixture = Self::v1_0_0()
            .with_extra_core(Component::Var(Var::Custom(
                "custom_extra_field".to_string(),
            )))
            .build();
        fixture.vars.custom = serde_json::json!({
            "custom_extra_field": value
        });
        ZervFixture::from(fixture)
    }

    // Maximum complexity fixture base - shared structure without dev component
    pub fn v2_3_4_max_complexity_base() -> ZervFixture {
        let mut fixture = ZervFixture::new()
            .with_version(2, 3, 4)
            .with_epoch(5)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(2)
            .with_core(Component::Var(Var::Custom("core_custom".to_string())))
            .with_core(Component::UInt(99))
            .with_extra_core(Component::Var(Var::Custom("extra_custom".to_string())))
            .with_extra_core(Component::Str("literal".to_string()))
            .with_extra_core(Component::UInt(42))
            .with_build(Component::Var(Var::BumpedBranch))
            .with_build(Component::Var(Var::Distance))
            .with_build(Component::Var(Var::BumpedCommitHashShort))
            .with_build(Component::Var(Var::Dirty))
            .with_build(Component::Var(Var::Custom("build_custom".to_string())))
            .with_build(Component::Str("build".to_string()))
            .with_build(Component::UInt(123))
            .with_branch("feature/complex-test".to_string())
            .with_distance(7)
            .with_commit_hash("abcdef1234567890".to_string())
            .build();

        fixture.vars.dirty = Some(true);
        fixture.vars.custom = serde_json::json!({
            "core_custom": "core_value",
            "extra_custom": "extra_value",
            "build_custom": "build_value"
        });

        ZervFixture::from(fixture)
    }

    // Maximum complexity fixture with dev component for SemVer
    pub fn v2_3_4_max_complexity_with_dev() -> ZervFixture {
        Self::v2_3_4_max_complexity_base().with_dev(3)
    }
}
