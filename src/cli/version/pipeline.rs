use crate::cli::utils::output_formatter::OutputFormatter;
use crate::constants::sources;
use crate::error::ZervError;
use crate::schema::create_zerv_version;
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
        sources::GIT => super::git_pipeline::process_git_source(&work_dir, &args)?,
        sources::STDIN => super::stdin_pipeline::process_stdin_source(&args)?,
        source => return Err(ZervError::UnknownSource(source.to_string())),
    };

    // 3. Apply overrides consistently for all sources
    if args.has_overrides() {
        zerv_object.vars.apply_overrides(&args)?;
    }

    // 4. Apply schema if specified (this can override the schema from stdin)
    if args.schema.is_some() || args.schema_ron.is_some() {
        zerv_object = create_zerv_version(
            zerv_object.vars,
            args.schema.as_deref(),
            args.schema_ron.as_deref(),
        )?;
    }

    // 5. Apply output formatting with enhanced options
    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.output_format,
        args.output_prefix.as_deref(),
        args.output_template.as_deref(),
    )?;

    Ok(output)
}
