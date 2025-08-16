use crate::version::zerv::PreReleaseLabel;

pub fn pre_release_label_to_pep440_string(label: &PreReleaseLabel) -> &'static str {
    match label {
        PreReleaseLabel::Alpha => "a",
        PreReleaseLabel::Beta => "b",
        PreReleaseLabel::Rc => "rc",
    }
}
