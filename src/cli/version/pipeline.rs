use crate::cli::utils::format_handler::InputFormatHandler;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::constants::{formats, sources};
use crate::error::ZervError;
use crate::pipeline::vcs_data_to_zerv_vars;
use crate::schema::create_zerv_version;
use crate::version::Zerv;
use std::env::current_dir;

use super::args::VersionArgs;

pub fn run_version_pipeline(mut args: VersionArgs) -> Result<String, ZervError> {
    // 0. Early validation - fail fast on conflicting options
    args.validate()?;

    // 1. Determine working directory
    let work_dir = match args.directory.as_deref() {
        Some(dir) => std::path::PathBuf::from(dir),
        None => current_dir()?,
    };

    // 2. Resolve input source and get Zerv object
    let mut zerv_object = match args.source.as_str() {
        sources::GIT => {
            // Get git VCS data
            // If directory was specified via -C, only look in that directory (depth 0)
            // If no directory specified, allow unlimited depth search
            let max_depth = if args.directory.is_some() {
                Some(0)
            } else {
                None
            };
            let mut vcs_data =
                crate::vcs::detect_vcs_with_limit(&work_dir, max_depth)?.get_vcs_data()?;

            // Parse git tag with input format if available and validate it
            if let Some(ref tag_version) = vcs_data.tag_version {
                let _parsed_version =
                    InputFormatHandler::parse_version_string(tag_version, &args.input_format)?;
                // Validation passed - the tag is in a valid format
            }

            // Apply VCS overrides (including --tag-version with input format validation and context control)
            vcs_data = vcs_data.apply_overrides(&args)?;

            // Convert VCS data to ZervVars
            let vars = vcs_data_to_zerv_vars(vcs_data)?;

            // Create Zerv object from vars and schema
            create_zerv_version(vars, args.schema.as_deref(), args.schema_ron.as_deref())?
        }
        sources::STDIN => {
            // For stdin source, default to "zerv" format if "auto" is specified
            let input_format = if args.input_format == formats::AUTO {
                formats::ZERV
            } else {
                &args.input_format
            };

            // Parse stdin as Zerv RON
            let mut zerv_from_stdin = InputFormatHandler::parse_stdin(input_format)?;

            // Apply overrides to the parsed Zerv object if any are specified
            if args.has_overrides() {
                // Convert Zerv back to VcsData-like structure for override processing
                let mut temp_vcs_data = zerv_to_vcs_data(&zerv_from_stdin)?;

                // Apply overrides
                temp_vcs_data = temp_vcs_data.apply_overrides(&args)?;

                // Convert back to ZervVars and create new Zerv object
                let updated_vars = vcs_data_to_zerv_vars(temp_vcs_data)?;

                // Preserve the original schema but update vars
                zerv_from_stdin.vars = updated_vars;
            }

            zerv_from_stdin
        }
        source => return Err(ZervError::UnknownSource(source.to_string())),
    };

    // 3. Apply schema if specified (this can override the schema from stdin)
    if args.schema.is_some() || args.schema_ron.is_some() {
        zerv_object = create_zerv_version(
            zerv_object.vars,
            args.schema.as_deref(),
            args.schema_ron.as_deref(),
        )?;
    }

    // 4. Apply output formatting with enhanced options
    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.output_format,
        args.output_prefix.as_deref(),
        args.output_template.as_deref(),
    )?;

    Ok(output)
}

/// Convert Zerv object back to VcsData for override processing
/// This is a helper function to enable override application on stdin input
fn zerv_to_vcs_data(zerv: &Zerv) -> Result<crate::vcs::VcsData, ZervError> {
    use crate::vcs::VcsData;

    // Extract values from ZervVars, providing defaults where needed
    let vars = &zerv.vars;

    // Reconstruct tag version from major.minor.patch if available
    let tag_version = match (vars.major, vars.minor, vars.patch) {
        (Some(major), Some(minor), Some(patch)) => Some(format!("{major}.{minor}.{patch}")),
        _ => None,
    };

    Ok(VcsData {
        tag_version,
        distance: vars.distance.unwrap_or(0) as u32,
        commit_hash: vars
            .bumped_commit_hash
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        commit_hash_short: vars
            .bumped_commit_hash
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
            .chars()
            .take(7)
            .collect(),
        current_branch: vars.bumped_branch.clone(),
        commit_timestamp: vars.dev.unwrap_or(0) as i64,
        tag_timestamp: vars.last_timestamp.map(|t| t as i64),
        is_dirty: vars.dirty.unwrap_or(false),
        is_shallow: false, // Default to false for stdin input
    })
}
