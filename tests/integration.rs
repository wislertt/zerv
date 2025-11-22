#[ctor::ctor]
fn init_integration_test_logging() {
    let _ = dotenvy::dotenv().ok();
    zerv::logging::init_logging(false);
}

mod integration_tests;

pub use integration_tests::*;
