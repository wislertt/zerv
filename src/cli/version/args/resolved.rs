use std::fmt::Display;
use std::str::FromStr;

use super::{
    BumpsConfig,
    OverridesConfig,
    VersionArgs,
};
use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
};
use crate::cli::utils::template::Template;
use crate::error::ZervError;
use crate::utils::constants::pre_release_labels;
use crate::version::Zerv;

/// Resolve pre-release label template with strict validation
/// Shared utility function used by both ResolvedOverrides and ResolvedBumps
fn resolve_pre_release_label(
    template: &Option<Template<String>>,
    zerv: &Zerv,
) -> Result<Option<String>, ZervError> {
    match template {
        Some(t) => {
            let resolved = t.resolve(Some(zerv))?;

            // Handle None keywords after template resolution
            let trimmed = resolved.trim().to_lowercase();
            if matches!(
                trimmed.as_str(),
                "none" | "null" | "nil" | "nothing" | "empty"
            ) {
                return Ok(None);
            }

            // Strict validation: ensure resolved value is a valid pre-release label
            if !pre_release_labels::VALID_LABELS.contains(&resolved.as_str()) {
                return Err(ZervError::TemplateError(format!(
                    "Template resolved to invalid pre-release label '{}'. Must be one of: {} or None keywords: {}",
                    resolved,
                    pre_release_labels::VALID_LABELS.join(", "),
                    "none, null, nil, nothing, empty"
                )));
            }

            Ok(Some(resolved))
        }
        None => Ok(None),
    }
}

/// Resolved version of VersionArgs with templates rendered
#[derive(Debug, Clone)]
pub struct ResolvedArgs {
    pub overrides: ResolvedOverrides,
    pub bumps: ResolvedBumps,
    pub input: InputConfig,   // Reusable input config
    pub output: OutputConfig, // Reusable output config
}

/// Resolved overrides with all templates rendered to values
#[derive(Debug, Clone, Default)]
pub struct ResolvedOverrides {
    // VCS overrides (unchanged)
    pub tag_version: Option<String>,
    pub distance: Option<u32>,
    pub dirty: bool,
    pub no_dirty: bool,
    pub clean: bool,
    pub bumped_branch: Option<String>,
    pub bumped_commit_hash: Option<String>,
    pub bumped_timestamp: Option<i64>,

    // Version component overrides (resolved from templates)
    pub major: Option<u32>,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
    pub epoch: Option<u32>,
    pub post: Option<u32>,
    pub dev: Option<u32>,
    pub pre_release_label: Option<String>,
    pub pre_release_num: Option<u32>,
    pub custom: Option<String>,

    // Schema component overrides (resolved from templates)
    pub core: Vec<String>, // Resolved INDEX=VALUE strings
    pub extra_core: Vec<String>,
    pub build: Vec<String>,
}

/// Resolved bumps with all templates rendered to values
#[derive(Debug, Clone, Default)]
pub struct ResolvedBumps {
    // Field-based bumps (resolved from templates)
    pub bump_major: Option<Option<u32>>,
    pub bump_minor: Option<Option<u32>>,
    pub bump_patch: Option<Option<u32>>,
    pub bump_post: Option<Option<u32>>,
    pub bump_dev: Option<Option<u32>>,
    pub bump_pre_release_num: Option<Option<u32>>,
    pub bump_epoch: Option<Option<u32>>,
    pub bump_pre_release_label: Option<String>,

    // Schema-based bumps (resolved from templates)
    pub bump_core: Vec<String>,
    pub bump_extra_core: Vec<String>,
    pub bump_build: Vec<String>,

    // Context control (unchanged)
    pub bump_context: bool,
    pub no_bump_context: bool,
}

impl ResolvedArgs {
    /// Resolve all templates in VersionArgs using Zerv snapshot
    pub fn resolve(args: &VersionArgs, zerv: &Zerv) -> Result<Self, ZervError> {
        let overrides = ResolvedOverrides::resolve(&args.overrides, zerv)?;
        let bumps = ResolvedBumps::resolve(&args.bumps, zerv)?;

        Ok(ResolvedArgs {
            overrides,
            bumps,
            input: args.input.clone(),
            output: args.output.clone(),
        })
    }
}

impl ResolvedOverrides {
    fn resolve(overrides: &OverridesConfig, zerv: &Zerv) -> Result<Self, ZervError> {
        Ok(ResolvedOverrides {
            // VCS overrides (copy as-is)
            tag_version: overrides.tag_version.clone(),
            distance: overrides.distance,
            dirty: overrides.dirty,
            no_dirty: overrides.no_dirty,
            clean: overrides.clean,
            bumped_branch: overrides.bumped_branch.clone(),
            bumped_commit_hash: overrides.bumped_commit_hash.clone(),
            bumped_timestamp: overrides.bumped_timestamp,

            // Version component overrides (resolve templates)
            major: Self::resolve_template(&overrides.major, zerv)?,
            minor: Self::resolve_template(&overrides.minor, zerv)?,
            patch: Self::resolve_template(&overrides.patch, zerv)?,
            epoch: Self::resolve_template(&overrides.epoch, zerv)?,
            post: Self::resolve_template(&overrides.post, zerv)?,
            dev: Self::resolve_template(&overrides.dev, zerv)?,
            pre_release_label: resolve_pre_release_label(&overrides.pre_release_label, zerv)?,
            pre_release_num: Self::resolve_template(&overrides.pre_release_num, zerv)?,
            custom: overrides.custom.clone(),

            // Schema component overrides (resolve templates)
            core: Self::resolve_template_strings(&overrides.core, zerv)?,
            extra_core: Self::resolve_template_strings(&overrides.extra_core, zerv)?,
            build: Self::resolve_template_strings(&overrides.build, zerv)?,
        })
    }

    fn resolve_template<T>(
        template: &Option<Template<T>>,
        zerv: &Zerv,
    ) -> Result<Option<T>, ZervError>
    where
        T: FromStr + Clone,
        T::Err: Display,
    {
        match template {
            Some(t) => Ok(Some(t.resolve(Some(zerv))?)),
            None => Ok(None),
        }
    }

    fn resolve_template_strings(
        templates: &[Template<String>],
        zerv: &Zerv,
    ) -> Result<Vec<String>, ZervError> {
        templates
            .iter()
            .map(|template| template.resolve(Some(zerv)))
            .collect()
    }

    /// Get the dirty override state (None = use VCS, Some(bool) = override)
    // TODO: this is duplicated
    pub fn dirty_override(&self) -> Option<bool> {
        match (self.dirty, self.no_dirty) {
            (true, false) => Some(true),    // --dirty
            (false, true) => Some(false),   // --no-dirty
            (false, false) => None,         // neither (use VCS)
            (true, true) => unreachable!(), // Should be caught by validation
        }
    }
}

impl ResolvedBumps {
    fn resolve(bumps: &BumpsConfig, zerv: &Zerv) -> Result<Self, ZervError> {
        Ok(ResolvedBumps {
            // Field-based bumps (resolve templates)
            bump_major: Self::resolve_bump(&bumps.bump_major, zerv)?,
            bump_minor: Self::resolve_bump(&bumps.bump_minor, zerv)?,
            bump_patch: Self::resolve_bump(&bumps.bump_patch, zerv)?,
            bump_post: Self::resolve_bump(&bumps.bump_post, zerv)?,
            bump_dev: Self::resolve_bump(&bumps.bump_dev, zerv)?,
            bump_pre_release_num: Self::resolve_bump(&bumps.bump_pre_release_num, zerv)?,
            bump_epoch: Self::resolve_bump(&bumps.bump_epoch, zerv)?,
            bump_pre_release_label: resolve_pre_release_label(&bumps.bump_pre_release_label, zerv)?,

            // Schema-based bumps (resolve templates)
            bump_core: Self::resolve_template_strings(&bumps.bump_core, zerv)?,
            bump_extra_core: Self::resolve_template_strings(&bumps.bump_extra_core, zerv)?,
            bump_build: Self::resolve_template_strings(&bumps.bump_build, zerv)?,

            // Context control (copy as-is)
            bump_context: bumps.bump_context,
            no_bump_context: bumps.no_bump_context,
        })
    }

    fn resolve_bump(
        bump: &Option<Option<Template<u32>>>,
        zerv: &Zerv,
    ) -> Result<Option<Option<u32>>, ZervError> {
        match bump {
            Some(Some(template)) => Ok(Some(Some(template.resolve(Some(zerv))?))),
            Some(None) => Ok(Some(None)),
            None => Ok(None),
        }
    }

    fn resolve_template_strings(
        templates: &[Template<String>],
        zerv: &Zerv,
    ) -> Result<Vec<String>, ZervError> {
        templates
            .iter()
            .map(|template| template.resolve(Some(zerv)))
            .collect()
    }
}
