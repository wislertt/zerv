use rstest::rstest;

use super::super::*;
use crate::test_utils::version_args::VersionArgsFixture;
use crate::test_utils::zerv::ZervFixture;

#[rstest]
#[case(2, 1, 0, 1, 2)]
#[case(5, 3, 1, 2, 4)]
fn test_resolved_args_basic_resolution(
    #[case] major: u32,
    #[case] minor: u32,
    #[case] patch: u32,
    #[case] bump_major: u32,
    #[case] bump_minor: u32,
) {
    let args = VersionArgsFixture::new()
        .with_major(major)
        .with_minor(minor)
        .with_patch(patch)
        .with_bump_major(bump_major)
        .with_bump_minor(bump_minor)
        .build();

    let zerv = ZervFixture::new().with_version(1, 0, 0).build();
    let resolved = ResolvedArgs::resolve(&args, &zerv).unwrap();

    assert_eq!(resolved.overrides.major, Some(major));
    assert_eq!(resolved.overrides.minor, Some(minor));
    assert_eq!(resolved.overrides.patch, Some(patch));
    assert_eq!(resolved.bumps.bump_major, Some(Some(bump_major)));
    assert_eq!(resolved.bumps.bump_minor, Some(Some(bump_minor)));
}

#[rstest]
#[case(2, 1, 3)]
#[case(5, 4, 2)]
fn test_resolved_args_template_resolution(
    #[case] major: u64,
    #[case] minor: u64,
    #[case] patch: u64,
) {
    let mut args = VersionArgsFixture::new().build();
    args.overrides.major = Some("{{major}}".into());
    args.overrides.minor = Some("{{minor}}".into());
    args.overrides.patch = Some("{{patch}}".into());
    args.bumps.bump_major = Some(Some("{{major}}".into()));
    args.bumps.bump_minor = Some(Some("{{minor}}".into()));

    let zerv = ZervFixture::new().with_version(major, minor, patch).build();
    let resolved = ResolvedArgs::resolve(&args, &zerv).unwrap();

    assert_eq!(resolved.overrides.major, Some(major as u32));
    assert_eq!(resolved.overrides.minor, Some(minor as u32));
    assert_eq!(resolved.overrides.patch, Some(patch as u32));
    assert_eq!(resolved.bumps.bump_major, Some(Some(major as u32)));
    assert_eq!(resolved.bumps.bump_minor, Some(Some(minor as u32)));
}

#[test]
fn test_resolved_overrides_vcs_fields() {
    let args = VersionArgsFixture::new()
        .with_tag_version("v1.0.0")
        .with_distance(5)
        .with_dirty(true)
        .with_current_branch("main")
        .with_commit_hash("abc123")
        .build();

    let zerv = ZervFixture::new().build();
    let resolved = ResolvedArgs::resolve(&args, &zerv).unwrap();

    assert_eq!(resolved.overrides.tag_version, Some("v1.0.0".to_string()));
    assert_eq!(resolved.overrides.distance, Some(5));
    assert!(resolved.overrides.dirty);
    assert_eq!(resolved.overrides.bumped_branch, Some("main".to_string()));
    assert_eq!(
        resolved.overrides.bumped_commit_hash,
        Some("abc123".to_string())
    );
}

#[rstest]
#[case(true, false, Some(true))]
#[case(false, true, Some(false))]
#[case(false, false, None)]
fn test_resolved_overrides_dirty_override(
    #[case] dirty: bool,
    #[case] no_dirty: bool,
    #[case] expected: Option<bool>,
) {
    let overrides = ResolvedOverrides {
        dirty,
        no_dirty,
        ..Default::default()
    };
    assert_eq!(overrides.dirty_override(), expected);
}

#[test]
fn test_resolved_overrides_schema_fields() {
    let mut args = VersionArgsFixture::new().build();
    args.overrides.core = vec!["0=2".into(), "1={{minor}}".into()];
    args.overrides.extra_core = vec!["0=5".into()];
    args.overrides.build = vec!["0=build".into(), "1={{patch}}".into()];

    let zerv = ZervFixture::new().with_version(1, 2, 3).build();
    let resolved = ResolvedArgs::resolve(&args, &zerv).unwrap();

    assert_eq!(resolved.overrides.core, vec!["0=2", "1=2"]);
    assert_eq!(resolved.overrides.extra_core, vec!["0=5"]);
    assert_eq!(resolved.overrides.build, vec!["0=build", "1=3"]);
}

#[test]
fn test_resolved_bumps_field_based() {
    let mut args = VersionArgsFixture::new()
        .with_bump_major(1)
        .with_bump_pre_release_label("alpha")
        .build();
    args.bumps.bump_minor = Some(None);
    args.bumps.bump_patch = Some(Some("{{major}}".into()));

    let zerv = ZervFixture::new().with_version(2, 1, 0).build();
    let resolved = ResolvedArgs::resolve(&args, &zerv).unwrap();

    assert_eq!(resolved.bumps.bump_major, Some(Some(1)));
    assert_eq!(resolved.bumps.bump_minor, Some(None));
    assert_eq!(resolved.bumps.bump_patch, Some(Some(2)));
    assert_eq!(
        resolved.bumps.bump_pre_release_label,
        Some("alpha".to_string())
    );
}

#[test]
fn test_resolved_bumps_schema_based() {
    let mut args = VersionArgsFixture::new().build();
    args.bumps.bump_core = vec!["0=1".into(), "1={{minor}}".into()];
    args.bumps.bump_extra_core = vec!["0={{patch}}".into()];
    args.bumps.bump_build = vec!["0=test".into()];

    let zerv = ZervFixture::new().with_version(1, 2, 3).build();
    let resolved = ResolvedArgs::resolve(&args, &zerv).unwrap();

    assert_eq!(resolved.bumps.bump_core, vec!["0=1", "1=2"]);
    assert_eq!(resolved.bumps.bump_extra_core, vec!["0=3"]);
    assert_eq!(resolved.bumps.bump_build, vec!["0=test"]);
}

#[rstest]
#[case(true, false)]
#[case(false, true)]
fn test_resolved_bumps_context_control(#[case] bump_context: bool, #[case] no_bump_context: bool) {
    let args = VersionArgsFixture::new()
        .with_bump_context(bump_context)
        .with_no_bump_context(no_bump_context)
        .build();

    let zerv = ZervFixture::new().build();
    let resolved = ResolvedArgs::resolve(&args, &zerv).unwrap();

    assert_eq!(resolved.bumps.bump_context, bump_context);
    assert_eq!(resolved.bumps.no_bump_context, no_bump_context);
}

#[test]
fn test_resolved_args_empty_defaults() {
    let args = VersionArgsFixture::new().build();
    let zerv = ZervFixture::new().build();
    let resolved = ResolvedArgs::resolve(&args, &zerv).unwrap();

    assert!(resolved.overrides.major.is_none());
    assert!(resolved.overrides.minor.is_none());
    assert!(resolved.overrides.patch.is_none());
    assert!(resolved.overrides.core.is_empty());
    assert!(resolved.overrides.extra_core.is_empty());
    assert!(resolved.overrides.build.is_empty());
    assert!(resolved.bumps.bump_major.is_none());
    assert!(resolved.bumps.bump_minor.is_none());
    assert!(resolved.bumps.bump_patch.is_none());
    assert!(resolved.bumps.bump_core.is_empty());
    assert!(resolved.bumps.bump_extra_core.is_empty());
    assert!(resolved.bumps.bump_build.is_empty());
    assert!(!resolved.bumps.bump_context);
    assert!(!resolved.bumps.no_bump_context);
}

#[rstest]
#[case(5, 1, 2, 3, 4)]
#[case(10, 2, 1, 5, 7)]
fn test_resolved_args_mixed_templates_and_values(
    #[case] override_major: u32,
    #[case] bump_major: u32,
    #[case] zerv_major: u64,
    #[case] zerv_minor: u64,
    #[case] zerv_patch: u64,
) {
    let mut args = VersionArgsFixture::new()
        .with_major(override_major)
        .with_bump_major(bump_major)
        .build();
    args.overrides.minor = Some("{{major}}".into());
    args.overrides.patch = Some("{{minor}}".into());
    args.bumps.bump_minor = Some(Some("{{patch}}".into()));

    let zerv = ZervFixture::new()
        .with_version(zerv_major, zerv_minor, zerv_patch)
        .build();
    let resolved = ResolvedArgs::resolve(&args, &zerv).unwrap();

    assert_eq!(resolved.overrides.major, Some(override_major));
    assert_eq!(resolved.overrides.minor, Some(zerv_major as u32));
    assert_eq!(resolved.overrides.patch, Some(zerv_minor as u32));
    assert_eq!(resolved.bumps.bump_major, Some(Some(bump_major)));
    assert_eq!(resolved.bumps.bump_minor, Some(Some(zerv_patch as u32)));
}
