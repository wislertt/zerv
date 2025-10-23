//! Cross-module combination tests for integration test revitalization
//!
//! This module tests interactions between different configuration modules:
//! - MainConfig + OverrideConfig
//! - MainConfig + BumpsConfig
//! - OverrideConfig + BumpsConfig
//! - All three modules together

pub mod main_bump_interactions;
pub mod main_override_interactions;
