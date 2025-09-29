use crate::cli::utils::output_formatter::OutputFormatter;
use crate::constants::sources;
use crate::error::ZervError;
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
    // Note: Overrides are applied within each source pipeline to ensure
    // schema tier is determined based on final state after overrides
    let zerv_object = match args.source.as_str() {
        sources::GIT => super::git_pipeline::process_git_source(&work_dir, &args)?,
        sources::STDIN => super::stdin_pipeline::process_stdin_source(&args)?,
        source => return Err(ZervError::UnknownSource(source.to_string())),
    };

    // Note: Schema application is now handled within each source pipeline
    // to ensure proper tier determination based on final state after overrides

    // 5. Apply output formatting with enhanced options
    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.output_format,
        args.output_prefix.as_deref(),
        args.output_template.as_deref(),
    )?;

    Ok(output)
}
