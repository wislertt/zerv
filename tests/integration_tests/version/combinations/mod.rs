//! Cross-module combination tests for integration test revitalization
//!
//! This module tests interactions between different configuration modules:
//! - MainConfig + OverrideConfig
//! - MainConfig + BumpsConfig
//! - OverrideConfig + BumpsConfig
//! - All three modules together

pub mod complex_workflow_scenarios;
pub mod main_bump_interactions;
pub mod main_override_interactions;
pub mod override_bump_interactions;
