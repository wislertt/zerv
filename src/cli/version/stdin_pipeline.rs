use crate::cli::utils::format_handler::InputFormatHandler;
use crate::error::ZervError;
use crate::version::Zerv;

use super::args::VersionArgs;

/// Process stdin source and return a Zerv object
pub fn process_stdin_source(_args: &VersionArgs) -> Result<Zerv, ZervError> {
    // Parse stdin as Zerv RON
    let zerv_from_stdin = InputFormatHandler::parse_stdin_to_zerv()?;

    // Note: Overrides are now handled in the main pipeline for consistency
    Ok(zerv_from_stdin)
}
