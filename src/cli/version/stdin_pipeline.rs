use crate::cli::utils::format_handler::InputFormatHandler;
use crate::error::ZervError;
use crate::version::Zerv;

use super::args::VersionArgs;

/// Process stdin source and return a Zerv object
pub fn process_stdin_source(args: &VersionArgs) -> Result<Zerv, ZervError> {
    // Parse stdin as Zerv RON
    let mut zerv_from_stdin = InputFormatHandler::parse_stdin_to_zerv()?;

    // TODO: try to move this logic to main pipeline (unify with other sources)
    // Apply overrides to be consistent with git pipeline
    // Always apply overrides for consistency with git pipeline
    zerv_from_stdin.vars.apply_overrides(args)?;

    // Apply schema with default fallback if needed
    let (schema_name, schema_ron) = args.resolve_schema();
    if schema_name.is_some() || schema_ron.is_some() {
        zerv_from_stdin =
            crate::schema::create_zerv_version(zerv_from_stdin.vars, schema_name, schema_ron)?;
    }
    // TODO: end of move plan

    Ok(zerv_from_stdin)
}
