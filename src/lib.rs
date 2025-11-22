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

    #[ctor::ctor]
    fn init_test_logging() {
        let _ = dotenvy::dotenv().ok();
        crate::logging::init_logging(false);
    }
}
