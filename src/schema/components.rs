use crate::utils::constants::timestamp_patterns;
use crate::version::zerv::{
    Component,
    Var,
};

pub fn standard_core() -> Vec<Component> {
    vec![
        Component::Var(Var::Major),
        Component::Var(Var::Minor),
        Component::Var(Var::Patch),
    ]
}

pub fn calver_core() -> Vec<Component> {
    vec![
        Component::Var(Var::Timestamp(timestamp_patterns::YYYY.to_string())),
        Component::Var(Var::Timestamp(timestamp_patterns::MM.to_string())),
        Component::Var(Var::Timestamp(timestamp_patterns::DD.to_string())),
        Component::Var(Var::Patch),
    ]
}

pub fn prerelease_core() -> Vec<Component> {
    vec![Component::Var(Var::Epoch), Component::Var(Var::PreRelease)]
}

pub fn prerelease_post_core() -> Vec<Component> {
    vec![
        Component::Var(Var::Epoch),
        Component::Var(Var::PreRelease),
        Component::Var(Var::Post),
    ]
}

pub fn prerelease_post_dev_core() -> Vec<Component> {
    vec![
        Component::Var(Var::Epoch),
        Component::Var(Var::PreRelease),
        Component::Var(Var::Post),
        Component::Var(Var::Dev),
    ]
}

pub fn build_context() -> Vec<Component> {
    vec![
        Component::Var(Var::BumpedBranch),
        Component::Var(Var::Distance),
        Component::Var(Var::BumpedCommitHashShort),
    ]
}

pub fn build_if_enabled(with_context: bool) -> Vec<Component> {
    if with_context {
        build_context()
    } else {
        vec![]
    }
}

pub fn epoch_extra_core() -> Vec<Component> {
    vec![Component::Var(Var::Epoch)]
}
