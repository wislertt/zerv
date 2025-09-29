use crate::cli::utils::format_handler::InputFormatHandler;
use crate::error::ZervError;

use super::args::VersionArgs;
use super::zerv_draft::ZervDraft;

/// Process stdin source and return a ZervDraft object
pub fn process_stdin_source(_args: &VersionArgs) -> Result<ZervDraft, ZervError> {
    // Parse stdin as Zerv RON (includes schema)
    let zerv_from_stdin = InputFormatHandler::parse_stdin_to_zerv()?;

    // Return ZervDraft with existing schema (stdin source)
    Ok(ZervDraft::new(
        zerv_from_stdin.vars,
        Some(zerv_from_stdin.schema.into()),
    ))
}
