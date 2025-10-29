use std::env::current_dir;

use super::args::VersionArgs;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::error::ZervError;
use crate::utils::constants::sources;

pub fn run_version_pipeline(mut args: VersionArgs) -> Result<String, ZervError> {
    // 0. Early validation - fail fast on conflicting options
    args.validate()?;

    // 1. Determine working directory
    let work_dir = match args.input.directory.as_deref() {
        Some(dir) => std::path::PathBuf::from(dir),
        None => current_dir()?,
    };

    // 2. Get ZervDraft from source (no schema applied yet)
    let zerv_draft = match args.input.source.as_str() {
        sources::GIT => super::git_pipeline::process_git_source(&work_dir, &args)?,
        sources::STDIN => super::stdin_pipeline::process_stdin_source(&args)?,
        source => return Err(ZervError::UnknownSource(source.to_string())),
    };

    // 3. Convert to Zerv (applies overrides internally)
    let zerv_object = zerv_draft.to_zerv(&args)?;

    // 4. Apply output formatting with template resolution
    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.output.output_format,
        args.output.output_prefix.as_deref(),
        &args.output.output_template,
    )?;

    Ok(output)
}
