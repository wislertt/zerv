use crate::version::zerv::PreReleaseLabel;

pub fn pre_release_label_to_semver_string(label: &PreReleaseLabel) -> &'static str {
    match label {
        PreReleaseLabel::Alpha => "alpha",
        PreReleaseLabel::Beta => "beta",
        PreReleaseLabel::Rc => "rc",
    }
}
