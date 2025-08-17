#[cfg(test)]
use super::{Component, PreReleaseLabel, PreReleaseVar, Zerv, ZervSchema, ZervVars};

#[cfg(test)]
pub fn base_zerv() -> Zerv {
    Zerv {
        schema: ZervSchema {
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
pub fn zerv_1_0_0_with_pre_release(label: PreReleaseLabel, number: Option<u64>) -> Zerv {
    let mut zerv = base_zerv();
    zerv.schema
        .extra_core
        .push(Component::VarField("pre_release".to_string()));
    zerv.vars.pre_release = Some(PreReleaseVar { label, number });
    zerv
}

#[cfg(test)]
pub fn zerv_1_0_0_with_extra_core(components: Vec<Component>) -> Zerv {
    let mut zerv = base_zerv();
    zerv.schema.extra_core = components;
    zerv
}

#[cfg(test)]
pub fn zerv_1_0_0_with_build(components: Vec<Component>) -> Zerv {
    let mut zerv = base_zerv();
    zerv.schema.build = components;
    zerv
}

#[cfg(test)]
pub fn zerv_1_0_0_with_pre_release_and_extra(
    label: PreReleaseLabel,
    number: Option<u64>,
    extra: Vec<Component>,
) -> Zerv {
    let mut zerv = base_zerv();
    zerv.schema.extra_core = vec![Component::VarField("pre_release".to_string())];
    zerv.schema.extra_core.extend(extra);
    zerv.vars.pre_release = Some(PreReleaseVar { label, number });
    zerv
}

#[cfg(test)]
pub fn zerv_1_0_0_with_pre_release_and_build(
    label: PreReleaseLabel,
    number: Option<u64>,
    build: Vec<Component>,
) -> Zerv {
    let mut zerv = zerv_1_0_0_with_pre_release(label, number);
    zerv.schema.build = build;
    zerv
}

#[cfg(test)]
pub fn zerv_1_0_0_with_epoch(epoch: u64) -> Zerv {
    let mut zerv = base_zerv();
    zerv.schema
        .extra_core
        .push(Component::VarField("epoch".to_string()));
    zerv.vars.epoch = Some(epoch);
    zerv
}

#[cfg(test)]
pub fn zerv_1_0_0_with_post(post: u64) -> Zerv {
    let mut zerv = base_zerv();
    zerv.schema
        .extra_core
        .push(Component::VarField("post".to_string()));
    zerv.vars.post = Some(post);
    zerv
}

#[cfg(test)]
pub fn zerv_1_0_0_with_dev(dev: u64) -> Zerv {
    let mut zerv = base_zerv();
    zerv.schema
        .extra_core
        .push(Component::VarField("dev".to_string()));
    zerv.vars.dev = Some(dev);
    zerv
}

#[cfg(test)]
pub fn zerv_version(major: u64, minor: u64, patch: u64) -> Zerv {
    let mut zerv = base_zerv();
    zerv.vars.major = Some(major);
    zerv.vars.minor = Some(minor);
    zerv.vars.patch = Some(patch);
    zerv
}

// Aliases for backward compatibility
#[cfg(test)]
pub fn with_version(major: u64, minor: u64, patch: u64) -> Zerv {
    zerv_version(major, minor, patch)
}

#[cfg(test)]
pub fn with_pre_release(label: PreReleaseLabel, number: Option<u64>) -> Zerv {
    zerv_1_0_0_with_pre_release(label, number)
}

#[cfg(test)]
pub fn with_extra_core(components: Vec<Component>) -> Zerv {
    zerv_1_0_0_with_extra_core(components)
}

#[cfg(test)]
pub fn with_build(components: Vec<Component>) -> Zerv {
    zerv_1_0_0_with_build(components)
}

#[cfg(test)]
pub fn with_pre_release_and_extra(
    label: PreReleaseLabel,
    number: Option<u64>,
    extra: Vec<Component>,
) -> Zerv {
    zerv_1_0_0_with_pre_release_and_extra(label, number, extra)
}

#[cfg(test)]
pub fn with_pre_release_and_build(
    label: PreReleaseLabel,
    number: Option<u64>,
    build: Vec<Component>,
) -> Zerv {
    zerv_1_0_0_with_pre_release_and_build(label, number, build)
}

#[cfg(test)]
pub fn with_epoch(epoch: u64) -> Zerv {
    zerv_1_0_0_with_epoch(epoch)
}

#[cfg(test)]
pub fn with_post(post: u64) -> Zerv {
    zerv_1_0_0_with_post(post)
}

#[cfg(test)]
pub fn with_dev(dev: u64) -> Zerv {
    zerv_1_0_0_with_dev(dev)
}

// PEP440-specific Zerv builders
#[cfg(test)]
pub fn pep_zerv_1_2_3() -> Zerv {
    zerv_version(1, 2, 3)
}

#[cfg(test)]
pub fn pep_zerv_1_2_3_epoch_2() -> Zerv {
    let mut zerv = zerv_version(1, 2, 3);
    zerv.schema
        .extra_core
        .push(Component::VarField("epoch".to_string()));
    zerv.vars.epoch = Some(2);
    zerv
}

#[cfg(test)]
pub fn pep_zerv_1_2_3_alpha_1() -> Zerv {
    let mut zerv = zerv_version(1, 2, 3);
    zerv.schema
        .extra_core
        .push(Component::VarField("pre_release".to_string()));
    zerv.vars.pre_release = Some(PreReleaseVar {
        label: PreReleaseLabel::Alpha,
        number: Some(1),
    });
    zerv
}

#[cfg(test)]
pub fn pep_zerv_1_2_3_post_1() -> Zerv {
    let mut zerv = zerv_version(1, 2, 3);
    zerv.schema
        .extra_core
        .push(Component::VarField("post".to_string()));
    zerv.vars.post = Some(1);
    zerv
}

#[cfg(test)]
pub fn pep_zerv_1_2_3_dev_1() -> Zerv {
    let mut zerv = zerv_version(1, 2, 3);
    zerv.schema
        .extra_core
        .push(Component::VarField("dev".to_string()));
    zerv.vars.dev = Some(1);
    zerv
}

#[cfg(test)]
pub fn pep_zerv_1_2_3_ubuntu_build() -> Zerv {
    let mut zerv = zerv_version(1, 2, 3);
    zerv.schema.build = vec![
        Component::String("ubuntu".to_string()),
        Component::Integer(20),
        Component::Integer(4),
    ];
    zerv
}

#[cfg(test)]
pub fn pep_zerv_complex_2_1_2_3_alpha_1_post_1_dev_1_local_1() -> Zerv {
    let mut zerv = zerv_version(1, 2, 3);
    zerv.schema.extra_core = vec![
        Component::VarField("epoch".to_string()),
        Component::VarField("pre_release".to_string()),
        Component::VarField("post".to_string()),
        Component::VarField("dev".to_string()),
    ];
    zerv.schema.build = vec![
        Component::String("local".to_string()),
        Component::Integer(1),
    ];
    zerv.vars.epoch = Some(2);
    zerv.vars.pre_release = Some(PreReleaseVar {
        label: PreReleaseLabel::Alpha,
        number: Some(1),
    });
    zerv.vars.post = Some(1);
    zerv.vars.dev = Some(1);
    zerv
}

// SemVer-specific Zerv builders
#[cfg(test)]
pub fn sem_zerv_1_2_3() -> Zerv {
    zerv_version(1, 2, 3)
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_alpha_1() -> Zerv {
    zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1))
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_something_1() -> Zerv {
    zerv_1_0_0_with_extra_core(vec![
        Component::String("something".to_string()),
        Component::Integer(1),
    ])
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_build_123() -> Zerv {
    zerv_1_0_0_with_build(vec![
        Component::String("build".to_string()),
        Component::Integer(123),
    ])
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_alpha_1_build_123() -> Zerv {
    zerv_1_0_0_with_pre_release_and_build(
        PreReleaseLabel::Alpha,
        Some(1),
        vec![
            Component::String("build".to_string()),
            Component::Integer(123),
        ],
    )
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_alpha_1_lowercase_4_uppercase_5_build_123() -> Zerv {
    let mut zerv = zerv_1_0_0_with_pre_release_and_build(
        PreReleaseLabel::Alpha,
        Some(1),
        vec![
            Component::String("build".to_string()),
            Component::Integer(123),
        ],
    );
    zerv.schema.extra_core = vec![
        Component::VarField("pre_release".to_string()),
        Component::String("lowercase".to_string()),
        Component::Integer(4),
        Component::String("UPPERCASE".to_string()),
        Component::Integer(5),
    ];
    zerv
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_foo_bar_beta_2_baz() -> Zerv {
    let mut zerv = zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(2));
    zerv.schema.extra_core = vec![
        Component::String("foo".to_string()),
        Component::String("bar".to_string()),
        Component::VarField("pre_release".to_string()),
        Component::String("baz".to_string()),
    ];
    zerv
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_alpha_1_beta_2() -> Zerv {
    zerv_1_0_0_with_pre_release_and_extra(
        PreReleaseLabel::Alpha,
        Some(1),
        vec![Component::String("beta".to_string()), Component::Integer(2)],
    )
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_rc_1_alpha_2_beta_3() -> Zerv {
    zerv_1_0_0_with_pre_release_and_extra(
        PreReleaseLabel::Rc,
        Some(1),
        vec![
            Component::String("alpha".to_string()),
            Component::Integer(2),
            Component::String("beta".to_string()),
            Component::Integer(3),
        ],
    )
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_rc_alpha_1() -> Zerv {
    zerv_1_0_0_with_pre_release_and_extra(
        PreReleaseLabel::Rc,
        None,
        vec![
            Component::String("alpha".to_string()),
            Component::Integer(1),
        ],
    )
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_test_alpha_beta_rc_1() -> Zerv {
    let mut zerv = zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, None);
    zerv.schema.extra_core = vec![
        Component::String("test".to_string()),
        Component::VarField("pre_release".to_string()),
        Component::String("beta".to_string()),
        Component::String("rc".to_string()),
        Component::Integer(1),
    ];
    zerv
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_foo_1_alpha() -> Zerv {
    let mut zerv = zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, None);
    zerv.schema.extra_core = vec![
        Component::String("foo".to_string()),
        Component::Integer(1),
        Component::VarField("pre_release".to_string()),
    ];
    zerv
}

#[cfg(test)]
pub fn sem_zerv_1_0_0_bar_2_beta() -> Zerv {
    let mut zerv = zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, None);
    zerv.schema.extra_core = vec![
        Component::String("bar".to_string()),
        Component::Integer(2),
        Component::VarField("pre_release".to_string()),
    ];
    zerv
}

#[cfg(test)]
pub fn sem_zerv_core_overflow_1_2() -> Zerv {
    Zerv {
        schema: ZervSchema {
            core: vec![Component::Integer(1), Component::Integer(2)],
            extra_core: vec![],
            build: vec![],
        },
        vars: ZervVars::default(),
    }
}

#[cfg(test)]
pub fn sem_zerv_core_overflow_1_2_3_4_5() -> Zerv {
    Zerv {
        schema: ZervSchema {
            core: vec![
                Component::Integer(1),
                Component::Integer(2),
                Component::Integer(3),
                Component::Integer(4),
                Component::Integer(5),
            ],
            extra_core: vec![],
            build: vec![],
        },
        vars: ZervVars::default(),
    }
}
