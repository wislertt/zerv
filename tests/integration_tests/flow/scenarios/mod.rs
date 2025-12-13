// Flow workflow scenario integration tests

pub mod abbreviated_versions;
pub mod complex_release_branch;
pub mod gitflow;
pub mod test_utils;
pub mod trunk_based;

// Re-export test utilities for easy access
pub use test_utils::{
    FlowIntegrationTestScenario,
    FlowTestResult,
    test_flow_pipeline_with_fixture,
    test_flow_pipeline_with_fixture_and_schema,
    test_flow_pipeline_with_fixture_and_schema_opt,
    test_flow_pipeline_with_schema_test_cases,
};
