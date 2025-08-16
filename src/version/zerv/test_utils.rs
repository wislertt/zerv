#[cfg(test)]
use super::{Component, PreReleaseLabel, PreReleaseVar, Zerv, ZervFormat, ZervVars};

#[cfg(test)]
pub fn base_zerv() -> Zerv {
    Zerv {
        format: ZervFormat {
            core: vec![
                Component::VarField("major".to_string()),
                Component::VarField("minor".to_string()),
                Component::VarField("patch".to_string()),
            ],
            extra_core: vec![],
            build: vec![],
        },
        vars: ZervVars {
            major: Some(1),
            minor: Some(0),
            patch: Some(0),
            ..Default::default()
        },
    }
}

#[cfg(test)]
pub fn with_pre_release(label: PreReleaseLabel, number: Option<u64>) -> Zerv {
    let mut zerv = base_zerv();
    zerv.format
        .extra_core
        .push(Component::VarField("pre_release".to_string()));
    zerv.vars.pre_release = Some(PreReleaseVar { label, number });
    zerv
}

#[cfg(test)]
pub fn with_extra_core(components: Vec<Component>) -> Zerv {
    let mut zerv = base_zerv();
    zerv.format.extra_core = components;
    zerv
}

#[cfg(test)]
pub fn with_build(components: Vec<Component>) -> Zerv {
    let mut zerv = base_zerv();
    zerv.format.build = components;
    zerv
}

#[cfg(test)]
pub fn with_pre_release_and_extra(
    label: PreReleaseLabel,
    number: Option<u64>,
    extra: Vec<Component>,
) -> Zerv {
    let mut zerv = base_zerv();
    zerv.format.extra_core = vec![Component::VarField("pre_release".to_string())];
    zerv.format.extra_core.extend(extra);
    zerv.vars.pre_release = Some(PreReleaseVar { label, number });
    zerv
}

#[cfg(test)]
pub fn with_pre_release_and_build(
    label: PreReleaseLabel,
    number: Option<u64>,
    build: Vec<Component>,
) -> Zerv {
    let mut zerv = with_pre_release(label, number);
    zerv.format.build = build;
    zerv
}
