use crate::version::zerv::{Component, PreReleaseLabel, PreReleaseVar, Zerv, ZervSchema, ZervVars};

use super::{ZervSchemaFixture, ZervVarsFixture};

/// Fixture for creating complete Zerv test data with RON string support
pub struct ZervFixture {
    zerv: Zerv,
}

impl ZervFixture {
    /// Create a basic Zerv with version 1.0.0
    pub fn basic() -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::basic().into(),
                ZervVarsFixture::basic().into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create basic Zerv: {e}")),
        }
    }

    /// Create a Zerv with specific version
    pub fn with_version(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::basic().into(),
                ZervVarsFixture::with_version(major, minor, patch).into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv with version: {e}")),
        }
    }

    /// Create a Zerv with pre-release
    pub fn with_pre_release(label: PreReleaseLabel, number: Option<u64>) -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::with_pre_release().into(),
                ZervVarsFixture::basic()
                    .with_pre_release(label, number)
                    .into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv with pre-release: {e}")),
        }
    }

    /// Create a Zerv with epoch
    pub fn with_epoch(epoch: u64) -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::with_epoch().into(),
                ZervVarsFixture::basic().with_epoch(epoch).into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv with epoch: {e}")),
        }
    }

    /// Create a Zerv with post version
    pub fn with_post(post: u64) -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::with_post().into(),
                ZervVarsFixture::basic().with_post(post).into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv with post: {e}")),
        }
    }

    /// Create a Zerv with dev version
    pub fn with_dev(dev: u64) -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::with_dev().into(),
                ZervVarsFixture::basic().with_dev(dev).into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv with dev: {e}")),
        }
    }

    /// Create a Zerv with build metadata
    pub fn with_build(components: Vec<Component>) -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::with_build(components).into(),
                ZervVarsFixture::basic().into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv with build: {e}")),
        }
    }

    /// Create a Zerv with custom extra core components
    pub fn with_extra_core(components: Vec<Component>) -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::with_extra_core(components).into(),
                ZervVarsFixture::basic().into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv with extra core: {e}")),
        }
    }

    /// Create a complex Zerv with all components
    pub fn with_all_components(
        epoch: u64,
        label: PreReleaseLabel,
        number: Option<u64>,
        post: u64,
        dev: u64,
    ) -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::with_all_components().into(),
                ZervVarsFixture::basic()
                    .with_epoch(epoch)
                    .with_pre_release(label, number)
                    .with_post(post)
                    .with_dev(dev)
                    .into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv with all components: {e}")),
        }
    }

    /// Create a Zerv with VCS data
    pub fn with_vcs_data(
        distance: u64,
        dirty: bool,
        bumped_branch: String,
        bumped_commit_hash: String,
        last_commit_hash: String,
        last_timestamp: u64,
        last_branch: String,
    ) -> Self {
        Self {
            zerv: Zerv::new(
                ZervSchemaFixture::basic().into(),
                ZervVarsFixture::basic()
                    .with_vcs_data(
                        distance,
                        dirty,
                        bumped_branch,
                        bumped_commit_hash,
                        last_commit_hash,
                        last_timestamp,
                        last_branch,
                    )
                    .into(),
            )
            .unwrap_or_else(|e| panic!("Failed to create Zerv with VCS data: {e}")),
        }
    }

    /// Create from separate schema and vars fixtures
    pub fn from_parts(schema: ZervSchemaFixture, vars: ZervVarsFixture) -> Self {
        Self {
            zerv: Zerv::new(schema.into(), vars.into())
                .unwrap_or_else(|e| panic!("Failed to create Zerv from parts: {e}")),
        }
    }

    /// Modify the schema
    pub fn with_schema(mut self, schema: ZervSchemaFixture) -> Self {
        self.zerv.schema = schema.into();
        self
    }

    /// Modify the vars
    pub fn with_vars(mut self, vars: ZervVarsFixture) -> Self {
        self.zerv.vars = vars.into();
        self
    }

    /// Add a component to extra_core
    pub fn add_extra_core(mut self, component: Component) -> Self {
        self.zerv.schema.extra_core.push(component);
        self
    }

    /// Add a component to build
    pub fn add_build(mut self, component: Component) -> Self {
        self.zerv.schema.build.push(component);
        self
    }

    /// Get the Zerv
    pub fn zerv(&self) -> &Zerv {
        &self.zerv
    }

    /// Get the Zerv as RON string
    pub fn to_ron_string(&self) -> String {
        ron::ser::to_string_pretty(&self.zerv, ron::ser::PrettyConfig::default())
            .unwrap_or_else(|e| panic!("Failed to serialize Zerv to RON: {e}"))
    }

    /// Create from RON string
    pub fn from_ron_string(ron_string: &str) -> Result<Self, ron::error::SpannedError> {
        let zerv: Zerv = ron::de::from_str(ron_string)?;
        Ok(Self { zerv })
    }

    /// Get the schema fixture
    pub fn schema_fixture(&self) -> ZervSchemaFixture {
        self.zerv.schema.clone().into()
    }

    /// Get the vars fixture
    pub fn vars_fixture(&self) -> ZervVarsFixture {
        self.zerv.vars.clone().into()
    }
}

impl From<ZervFixture> for Zerv {
    fn from(fixture: ZervFixture) -> Self {
        fixture.zerv
    }
}

impl From<Zerv> for ZervFixture {
    fn from(zerv: Zerv) -> Self {
        Self { zerv }
    }
}

// Helper functions for creating complex schemas
impl ZervFixture {
    /// Create a Zerv with extra core components
    pub fn zerv_1_0_0_with_extra_core(components: Vec<Component>) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = components;
        zerv
    }

    /// Create a Zerv with pre-release and extra components
    pub fn zerv_1_0_0_with_pre_release_and_extra(
        label: PreReleaseLabel,
        number: Option<u64>,
        extra: Vec<Component>,
    ) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![Component::VarField("pre_release".to_string())];
        zerv.schema.extra_core.extend(extra);
        zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        zerv
    }

    /// Create a Zerv with pre-release and build metadata
    pub fn zerv_1_0_0_with_pre_release_and_build(
        label: PreReleaseLabel,
        number: Option<u64>,
        build: Vec<Component>,
    ) -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_pre_release(label, number);
        zerv.schema.build = build;
        zerv
    }
}

// Legacy compatibility functions - these match the old test_utils.rs API
impl ZervFixture {
    /// Legacy: base_zerv() - Create a basic Zerv with version 1.0.0
    pub fn base_zerv() -> Zerv {
        Self::basic().zerv().clone()
    }

    /// Legacy: zerv_1_0_0_with_pre_release() - Create Zerv with pre-release
    pub fn zerv_1_0_0_with_pre_release(label: PreReleaseLabel, number: Option<u64>) -> Zerv {
        Self::with_pre_release(label, number).zerv().clone()
    }

    /// Legacy: zerv_1_0_0_with_build() - Create Zerv with build components
    pub fn zerv_1_0_0_with_build(components: Vec<Component>) -> Zerv {
        let mut zerv = Self::basic().zerv().clone();
        zerv.schema.build = components;
        zerv
    }

    /// Legacy: zerv_1_0_0_with_epoch() - Create Zerv with epoch
    pub fn zerv_1_0_0_with_epoch(epoch: u64) -> Zerv {
        Self::with_epoch(epoch).zerv().clone()
    }

    /// Legacy: zerv_1_0_0_with_post() - Create Zerv with post version
    pub fn zerv_1_0_0_with_post(post: u64) -> Zerv {
        Self::with_post(post).zerv().clone()
    }

    /// Legacy: zerv_1_0_0_with_dev() - Create Zerv with dev version
    pub fn zerv_1_0_0_with_dev(dev: u64) -> Zerv {
        Self::with_dev(dev).zerv().clone()
    }

    /// Legacy: zerv_version() - Create Zerv with specific version
    pub fn zerv_version(major: u64, minor: u64, patch: u64) -> Zerv {
        Self::with_version(major, minor, patch).zerv().clone()
    }

    // PEP440-specific Zerv builders
    pub fn pep_zerv_1_2_3() -> Zerv {
        Self::zerv_version(1, 2, 3)
    }

    pub fn pep_zerv_1_2_3_epoch_2() -> Zerv {
        let mut zerv = Self::zerv_version(1, 2, 3);
        zerv.schema
            .extra_core
            .push(Component::VarField("epoch".to_string()));
        zerv.vars.epoch = Some(2);
        zerv
    }

    pub fn pep_zerv_1_2_3_alpha_1() -> Zerv {
        let mut zerv = Self::zerv_version(1, 2, 3);
        zerv.schema
            .extra_core
            .push(Component::VarField("pre_release".to_string()));
        zerv.vars.pre_release = Some(PreReleaseVar {
            label: PreReleaseLabel::Alpha,
            number: Some(1),
        });
        zerv
    }

    pub fn pep_zerv_1_2_3_post_1() -> Zerv {
        let mut zerv = Self::zerv_version(1, 2, 3);
        zerv.schema
            .extra_core
            .push(Component::VarField("post".to_string()));
        zerv.vars.post = Some(1);
        zerv
    }

    pub fn pep_zerv_1_2_3_dev_1() -> Zerv {
        let mut zerv = Self::zerv_version(1, 2, 3);
        zerv.schema
            .extra_core
            .push(Component::VarField("dev".to_string()));
        zerv.vars.dev = Some(1);
        zerv
    }

    pub fn pep_zerv_1_2_3_ubuntu_build() -> Zerv {
        let mut zerv = Self::zerv_version(1, 2, 3);
        zerv.schema.build = vec![
            Component::String("ubuntu".to_string()),
            Component::Integer(20),
            Component::Integer(4),
        ];
        zerv
    }

    pub fn pep_zerv_complex_2_1_2_3_alpha_1_post_1_dev_1_local_1() -> Zerv {
        let mut zerv = Self::zerv_version(1, 2, 3);
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
    pub fn sem_zerv_1_2_3() -> Zerv {
        Self::zerv_version(1, 2, 3)
    }

    pub fn sem_zerv_1_0_0_alpha_1() -> Zerv {
        Self::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1))
    }

    pub fn sem_zerv_1_0_0_something_1() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::String("something".to_string()),
            Component::Integer(1),
        ];
        zerv
    }

    pub fn sem_zerv_1_0_0_build_123() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![
            Component::String("build".to_string()),
            Component::Integer(123),
        ];
        zerv
    }

    pub fn sem_zerv_1_0_0_alpha_1_build_123() -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1));
        zerv.schema.build = vec![
            Component::String("build".to_string()),
            Component::Integer(123),
        ];
        zerv
    }

    pub fn sem_zerv_1_0_0_alpha_1_lowercase_4_uppercase_5_build_123() -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_pre_release_and_build(
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

    pub fn sem_zerv_1_0_0_foo_bar_beta_2_baz() -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, Some(2));
        zerv.schema.extra_core = vec![
            Component::String("foo".to_string()),
            Component::String("bar".to_string()),
            Component::VarField("pre_release".to_string()),
            Component::String("baz".to_string()),
        ];
        zerv
    }

    pub fn sem_zerv_1_0_0_alpha_1_beta_2() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_and_extra(
            PreReleaseLabel::Alpha,
            Some(1),
            vec![Component::String("beta".to_string()), Component::Integer(2)],
        )
    }

    pub fn sem_zerv_1_0_0_rc_1_alpha_2_beta_3() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_and_extra(
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

    pub fn sem_zerv_1_0_0_rc_alpha_1() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_and_extra(
            PreReleaseLabel::Rc,
            None,
            vec![
                Component::String("alpha".to_string()),
                Component::Integer(1),
            ],
        )
    }

    pub fn sem_zerv_1_0_0_test_alpha_beta_rc_1() -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, None);
        zerv.schema.extra_core = vec![
            Component::String("test".to_string()),
            Component::VarField("pre_release".to_string()),
            Component::String("beta".to_string()),
            Component::String("rc".to_string()),
            Component::Integer(1),
        ];
        zerv
    }

    pub fn sem_zerv_1_0_0_foo_1_alpha() -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, None);
        zerv.schema.extra_core = vec![
            Component::String("foo".to_string()),
            Component::Integer(1),
            Component::VarField("pre_release".to_string()),
        ];
        zerv
    }

    pub fn sem_zerv_1_0_0_bar_2_beta() -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_pre_release(PreReleaseLabel::Beta, None);
        zerv.schema.extra_core = vec![
            Component::String("bar".to_string()),
            Component::Integer(2),
            Component::VarField("pre_release".to_string()),
        ];
        zerv
    }

    pub fn sem_zerv_core_overflow_1_2() -> Zerv {
        Zerv::new(
            ZervSchema::new(
                vec![Component::Integer(1), Component::Integer(2)],
                vec![],
                vec![],
            )
            .unwrap(),
            ZervVars::default(),
        )
        .unwrap()
    }

    pub fn sem_zerv_core_overflow_1_2_3_4_5() -> Zerv {
        Zerv::new(
            ZervSchema::new(
                vec![
                    Component::Integer(1),
                    Component::Integer(2),
                    Component::Integer(3),
                    Component::Integer(4),
                    Component::Integer(5),
                ],
                vec![],
                vec![],
            )
            .unwrap(),
            ZervVars::default(),
        )
        .unwrap()
    }

    // Complex combination functions
    pub fn zerv_1_0_0_with_epoch_and_pre_release(
        epoch: u64,
        label: PreReleaseLabel,
        number: Option<u64>,
    ) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("epoch".to_string()),
            Component::VarField("pre_release".to_string()),
        ];
        zerv.vars.epoch = Some(epoch);
        zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        zerv
    }

    pub fn zerv_1_0_0_with_post_and_dev(post: u64, dev: u64) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("post".to_string()),
            Component::VarField("dev".to_string()),
        ];
        zerv.vars.post = Some(post);
        zerv.vars.dev = Some(dev);
        zerv
    }

    pub fn zerv_1_0_0_with_dev_and_post(dev: u64, post: u64) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("dev".to_string()),
            Component::VarField("post".to_string()),
        ];
        zerv.vars.dev = Some(dev);
        zerv.vars.post = Some(post);
        zerv
    }

    pub fn zerv_1_0_0_with_pre_release_and_post(
        label: PreReleaseLabel,
        number: Option<u64>,
        post: u64,
    ) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("pre_release".to_string()),
            Component::VarField("post".to_string()),
        ];
        zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        zerv.vars.post = Some(post);
        zerv
    }

    pub fn zerv_1_0_0_with_pre_release_and_dev(
        label: PreReleaseLabel,
        number: Option<u64>,
        dev: u64,
    ) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("pre_release".to_string()),
            Component::VarField("dev".to_string()),
        ];
        zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        zerv.vars.dev = Some(dev);
        zerv
    }

    pub fn zerv_1_0_0_with_pre_release_post_and_dev(
        label: PreReleaseLabel,
        number: Option<u64>,
        post: u64,
        dev: u64,
    ) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("pre_release".to_string()),
            Component::VarField("post".to_string()),
            Component::VarField("dev".to_string()),
        ];
        zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        zerv.vars.post = Some(post);
        zerv.vars.dev = Some(dev);
        zerv
    }

    pub fn zerv_1_0_0_with_pre_release_dev_and_post(
        label: PreReleaseLabel,
        number: Option<u64>,
        dev: u64,
        post: u64,
    ) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("pre_release".to_string()),
            Component::VarField("dev".to_string()),
            Component::VarField("post".to_string()),
        ];
        zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        zerv.vars.dev = Some(dev);
        zerv.vars.post = Some(post);
        zerv
    }

    pub fn zerv_1_0_0_with_epoch_post_and_dev(epoch: u64, post: u64, dev: u64) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("epoch".to_string()),
            Component::VarField("post".to_string()),
            Component::VarField("dev".to_string()),
        ];
        zerv.vars.epoch = Some(epoch);
        zerv.vars.post = Some(post);
        zerv.vars.dev = Some(dev);
        zerv
    }

    pub fn zerv_1_0_0_with_epoch_dev_and_post(epoch: u64, dev: u64, post: u64) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("epoch".to_string()),
            Component::VarField("dev".to_string()),
            Component::VarField("post".to_string()),
        ];
        zerv.vars.epoch = Some(epoch);
        zerv.vars.dev = Some(dev);
        zerv.vars.post = Some(post);
        zerv
    }

    pub fn zerv_1_0_0_with_all_components(
        epoch: u64,
        label: PreReleaseLabel,
        number: Option<u64>,
        post: u64,
        dev: u64,
    ) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("epoch".to_string()),
            Component::VarField("pre_release".to_string()),
            Component::VarField("post".to_string()),
            Component::VarField("dev".to_string()),
        ];
        zerv.vars.epoch = Some(epoch);
        zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        zerv.vars.post = Some(post);
        zerv.vars.dev = Some(dev);
        zerv
    }

    pub fn zerv_1_0_0_with_all_components_reordered(
        epoch: u64,
        label: PreReleaseLabel,
        number: Option<u64>,
        dev: u64,
        post: u64,
    ) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("epoch".to_string()),
            Component::VarField("pre_release".to_string()),
            Component::VarField("dev".to_string()),
            Component::VarField("post".to_string()),
        ];
        zerv.vars.epoch = Some(epoch);
        zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        zerv.vars.post = Some(post);
        zerv.vars.dev = Some(dev);
        zerv
    }

    // Build-related functions
    pub fn zerv_1_0_0_with_epoch_and_build(epoch: u64) -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_epoch(epoch);
        zerv.schema.build = vec![
            Component::String("build".to_string()),
            Component::Integer(123),
        ];
        zerv
    }

    pub fn zerv_1_0_0_with_post_and_build(post: u64) -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_post(post);
        zerv.schema.build = vec![
            Component::String("build".to_string()),
            Component::Integer(456),
        ];
        zerv
    }

    pub fn zerv_1_0_0_with_dev_and_build(dev: u64) -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_dev(dev);
        zerv.schema.build = vec![
            Component::String("build".to_string()),
            Component::Integer(789),
        ];
        zerv
    }

    pub fn zerv_1_0_0_with_epoch_pre_release_and_build(
        epoch: u64,
        label: PreReleaseLabel,
        number: Option<u64>,
    ) -> Zerv {
        let mut zerv = Self::zerv_1_0_0_with_epoch_and_pre_release(epoch, label, number);
        zerv.schema.build = vec![
            Component::String("build".to_string()),
            Component::String("abc".to_string()),
        ];
        zerv
    }

    pub fn zerv_1_0_0_with_foo_epoch_and_alpha(epoch: u64, alpha_num: u64) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("epoch".to_string()),
            Component::String("foo".to_string()),
            Component::VarField("pre_release".to_string()),
        ];
        zerv.vars.epoch = Some(epoch);
        zerv.vars.pre_release = Some(PreReleaseVar {
            label: PreReleaseLabel::Alpha,
            number: Some(alpha_num),
        });
        zerv
    }

    pub fn zerv_1_0_0_with_epoch_foo_and_post(epoch: u64, post: u64) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("epoch".to_string()),
            Component::String("foo".to_string()),
            Component::VarField("post".to_string()),
        ];
        zerv.vars.epoch = Some(epoch);
        zerv.vars.post = Some(post);
        zerv
    }

    pub fn zerv_1_0_0_with_bar_dev_and_epoch(dev: u64, epoch: u64) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::VarField("epoch".to_string()),
            Component::String("bar".to_string()),
            Component::VarField("dev".to_string()),
        ];
        zerv.vars.dev = Some(dev);
        zerv.vars.epoch = Some(epoch);
        zerv
    }

    // Special order functions
    pub fn zerv_1_0_0_with_foo_epoch_and_alpha_original_order(epoch: u64, alpha_num: u64) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::String("foo".to_string()),
            Component::VarField("epoch".to_string()),
            Component::VarField("pre_release".to_string()),
        ];
        zerv.vars.epoch = Some(epoch);
        zerv.vars.pre_release = Some(PreReleaseVar {
            label: PreReleaseLabel::Alpha,
            number: Some(alpha_num),
        });
        zerv
    }

    pub fn zerv_1_0_0_with_bar_dev_and_epoch_original_order(dev: u64, epoch: u64) -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.extra_core = vec![
            Component::String("bar".to_string()),
            Component::VarField("dev".to_string()),
            Component::VarField("epoch".to_string()),
        ];
        zerv.vars.dev = Some(dev);
        zerv.vars.epoch = Some(epoch);
        zerv
    }

    // PEP440-specific functions
    pub fn pep_zerv_1_0_0_epoch_1() -> Zerv {
        Self::zerv_1_0_0_with_epoch(1)
    }

    pub fn pep_zerv_1_0_0_epoch_5() -> Zerv {
        Self::zerv_1_0_0_with_epoch(5)
    }

    pub fn pep_zerv_1_0_0_epoch_999() -> Zerv {
        Self::zerv_1_0_0_with_epoch(999)
    }

    pub fn pep_zerv_1_0_0_post_5() -> Zerv {
        Self::zerv_1_0_0_with_post(5)
    }

    pub fn pep_zerv_1_0_0_post_0() -> Zerv {
        Self::zerv_1_0_0_with_post(0)
    }

    pub fn pep_zerv_1_0_0_dev_0() -> Zerv {
        Self::zerv_1_0_0_with_dev(0)
    }

    pub fn pep_zerv_1_0_0_dev_10() -> Zerv {
        Self::zerv_1_0_0_with_dev(10)
    }

    pub fn pep_zerv_1_0_0_epoch_2_alpha_1() -> Zerv {
        Self::zerv_1_0_0_with_epoch_and_pre_release(2, PreReleaseLabel::Alpha, Some(1))
    }

    pub fn pep_zerv_1_0_0_epoch_3_beta_2() -> Zerv {
        Self::zerv_1_0_0_with_epoch_and_pre_release(3, PreReleaseLabel::Beta, Some(2))
    }

    pub fn pep_zerv_1_0_0_epoch_1_rc_5() -> Zerv {
        Self::zerv_1_0_0_with_epoch_and_pre_release(1, PreReleaseLabel::Rc, Some(5))
    }

    pub fn pep_zerv_1_0_0_epoch_4_alpha() -> Zerv {
        Self::zerv_1_0_0_with_epoch_and_pre_release(4, PreReleaseLabel::Alpha, Some(0))
    }

    pub fn pep_zerv_1_0_0_post_1_dev_2() -> Zerv {
        Self::zerv_1_0_0_with_post_and_dev(1, 2)
    }

    pub fn pep_zerv_1_0_0_alpha_1_post_2() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Alpha, Some(1), 2)
    }

    pub fn pep_zerv_1_0_0_beta_3_post_1() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Beta, Some(3), 1)
    }

    pub fn pep_zerv_1_0_0_rc_2_post_5() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_and_post(PreReleaseLabel::Rc, Some(2), 5)
    }

    pub fn pep_zerv_1_0_0_alpha_1_dev_2() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Alpha, Some(1), 2)
    }

    pub fn pep_zerv_1_0_0_beta_2_dev_1() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Beta, Some(2), 1)
    }

    pub fn pep_zerv_1_0_0_rc_1_dev_3() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_and_dev(PreReleaseLabel::Rc, Some(1), 3)
    }

    pub fn pep_zerv_1_0_0_alpha_1_post_2_dev_3() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_post_and_dev(PreReleaseLabel::Alpha, Some(1), 2, 3)
    }

    pub fn pep_zerv_1_0_0_beta_2_post_3_dev_1() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_post_and_dev(PreReleaseLabel::Beta, Some(2), 3, 1)
    }

    pub fn pep_zerv_1_0_0_rc_1_post_1_dev_1() -> Zerv {
        Self::zerv_1_0_0_with_pre_release_post_and_dev(PreReleaseLabel::Rc, Some(1), 1, 1)
    }

    pub fn pep_zerv_1_0_0_epoch_2_post_1_dev_3() -> Zerv {
        Self::zerv_1_0_0_with_epoch_post_and_dev(2, 1, 3)
    }

    pub fn pep_zerv_1_0_0_epoch_1_post_1_dev_2() -> Zerv {
        Self::zerv_1_0_0_with_epoch_post_and_dev(1, 1, 2)
    }

    pub fn pep_zerv_1_0_0_epoch_3_alpha_1_post_2_dev_1() -> Zerv {
        Self::zerv_1_0_0_with_all_components(3, PreReleaseLabel::Alpha, Some(1), 2, 1)
    }

    pub fn pep_zerv_1_0_0_epoch_1_beta_2_post_1_dev_3() -> Zerv {
        Self::zerv_1_0_0_with_all_components(1, PreReleaseLabel::Beta, Some(2), 1, 3)
    }

    pub fn pep_zerv_1_0_0_epoch_1_build() -> Zerv {
        Self::zerv_1_0_0_with_epoch_and_build(1)
    }

    pub fn pep_zerv_1_0_0_post_1_build() -> Zerv {
        Self::zerv_1_0_0_with_post_and_build(1)
    }

    pub fn pep_zerv_1_0_0_dev_2_build() -> Zerv {
        Self::zerv_1_0_0_with_dev_and_build(2)
    }

    pub fn pep_zerv_1_0_0_epoch_2_alpha_1_build() -> Zerv {
        Self::zerv_1_0_0_with_epoch_pre_release_and_build(2, PreReleaseLabel::Alpha, Some(1))
    }

    pub fn pep_zerv_1_0_0_complex_local() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![
            Component::String("foo".to_string()),
            Component::String("bar".to_string()),
            Component::Integer(123),
        ];
        zerv
    }

    pub fn pep_zerv_1_0_0_all_components_complex_local() -> Zerv {
        let mut zerv =
            Self::zerv_1_0_0_with_all_components(1, PreReleaseLabel::Alpha, Some(1), 1, 1);
        zerv.schema.build = vec![
            Component::String("complex".to_string()),
            Component::String("local".to_string()),
            Component::Integer(456),
        ];
        zerv
    }

    // VCS-related functions
    pub fn pep_zerv_1_0_0_with_bumped_branch() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.vars.bumped_branch = Some("main".to_string());
        zerv
    }

    pub fn pep_zerv_1_0_0_with_distance() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![Component::VarField("distance".to_string())];
        zerv.vars.distance = Some(5);
        zerv
    }

    pub fn pep_zerv_1_0_0_with_commit_hash() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![Component::VarField("bumped_commit_hash".to_string())];
        zerv.vars.bumped_commit_hash = Some("abc123".to_string());
        zerv
    }

    pub fn pep_zerv_1_0_0_with_branch_distance_hash() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![
            Component::VarField("bumped_branch".to_string()),
            Component::VarField("distance".to_string()),
            Component::VarField("bumped_commit_hash".to_string()),
        ];
        zerv.vars.bumped_branch = Some("dev".to_string());
        zerv.vars.distance = Some(3);
        zerv.vars.bumped_commit_hash = Some("def456".to_string());
        zerv
    }

    pub fn pep_zerv_1_0_0_with_none_varfields() -> Zerv {
        Self::zerv_version(1, 0, 0)
    }

    // SemVer VCS-related functions
    pub fn sem_zerv_1_0_0_with_branch() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![Component::VarField("bumped_branch".to_string())];
        zerv.vars.bumped_branch = Some("dev".to_string());
        zerv
    }

    pub fn sem_zerv_1_0_0_with_distance() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![Component::VarField("distance".to_string())];
        zerv.vars.distance = Some(5);
        zerv
    }

    pub fn sem_zerv_1_0_0_with_commit_hash() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![Component::VarField("bumped_commit_hash".to_string())];
        zerv.vars.bumped_commit_hash = Some("abc123".to_string());
        zerv
    }

    pub fn sem_zerv_1_0_0_with_branch_distance_hash() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![
            Component::VarField("bumped_branch".to_string()),
            Component::VarField("distance".to_string()),
            Component::VarField("bumped_commit_hash".to_string()),
        ];
        zerv.vars.bumped_branch = Some("dev".to_string());
        zerv.vars.distance = Some(3);
        zerv.vars.bumped_commit_hash = Some("def456".to_string());
        zerv
    }

    pub fn sem_zerv_1_0_0_with_none_varfields() -> Zerv {
        let mut zerv = Self::zerv_version(1, 0, 0);
        zerv.schema.build = vec![
            Component::VarField("bumped_branch".to_string()),
            Component::VarField("distance".to_string()),
            Component::VarField("bumped_commit_hash".to_string()),
        ];
        // All vars are None by default
        zerv
    }
}

// Convenience methods for common test patterns
impl ZervFixture {
    /// Create a PEP440-style Zerv
    pub fn pep440_style() -> Self {
        Self::basic()
    }

    /// Create a SemVer-style Zerv
    pub fn semver_style() -> Self {
        Self::basic()
    }

    /// Create a Zerv with pre-release and build
    pub fn with_pre_release_and_build(
        label: PreReleaseLabel,
        number: Option<u64>,
        build: Vec<Component>,
    ) -> Self {
        let mut fixture = Self::with_pre_release(label, number);
        fixture.zerv.schema.build = build;
        fixture
    }

    /// Create a Zerv with epoch and pre-release
    pub fn with_epoch_and_pre_release(
        epoch: u64,
        label: PreReleaseLabel,
        number: Option<u64>,
    ) -> Self {
        let mut fixture = Self::with_epoch(epoch);
        fixture
            .zerv
            .schema
            .extra_core
            .push(Component::VarField("pre_release".to_string()));
        fixture.zerv.vars.pre_release = Some(PreReleaseVar { label, number });
        fixture
    }

    /// Create a Zerv with post and dev
    pub fn with_post_and_dev(post: u64, dev: u64) -> Self {
        let mut fixture = Self::with_post(post);
        fixture
            .zerv
            .schema
            .extra_core
            .push(Component::VarField("dev".to_string()));
        fixture.zerv.vars.dev = Some(dev);
        fixture
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_zerv_fixture() {
        let fixture = ZervFixture::basic();
        let zerv = fixture.zerv();

        // Print the complete Zerv object to see what it looks like
        println!("Basic Zerv object:");
        println!("{zerv:#?}");

        // Print the RON string representation
        let ron_string = fixture.to_ron_string();
        println!("\nBasic Zerv RON string:");
        println!("{ron_string}");

        // Verify the structure
        assert_eq!(zerv.schema.core.len(), 3);
        assert!(zerv.schema.extra_core.is_empty());
        assert!(zerv.schema.build.is_empty());

        // Verify vars
        assert_eq!(zerv.vars.major, Some(1));
        assert_eq!(zerv.vars.minor, Some(0));
        assert_eq!(zerv.vars.patch, Some(0));

        // Verify core components
        assert!(matches!(zerv.schema.core[0], Component::VarField(ref name) if name == "major"));
        assert!(matches!(zerv.schema.core[1], Component::VarField(ref name) if name == "minor"));
        assert!(matches!(zerv.schema.core[2], Component::VarField(ref name) if name == "patch"));
    }
}
