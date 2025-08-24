use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ZervConfig {
    pub ci: bool,
}

impl ZervConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = Config::builder()
            .add_source(Environment::with_prefix("ZERV").separator("_"))
            .add_source(Environment::default().try_parsing(true));

        // Future: Add zerv.toml support in Phase 4
        // if Path::new("zerv.toml").exists() {
        //     builder = builder.add_source(File::with_name("zerv.toml"));
        // }

        builder.build()?.try_deserialize()
    }

    pub fn should_use_native_git(&self) -> bool {
        self.ci
    }
}
