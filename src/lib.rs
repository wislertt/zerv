pub mod cli;
pub mod config;
pub mod error;
pub mod logging;
pub mod pipeline;
pub mod schema;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
pub mod utils;
pub mod vcs;
pub mod version;

#[cfg(test)]
mod test_setup {
    use tracing_subscriber::{
        EnvFilter,
        fmt,
    };

    #[ctor::ctor]
    fn init_test_logging() {
        let _ = dotenvy::dotenv().ok();

        let _ = fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(EnvFilter::from_default_env())
            .with_target(true)
            .compact()
            .try_init();
    }
}
