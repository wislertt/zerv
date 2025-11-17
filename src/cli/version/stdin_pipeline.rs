use super::args::VersionArgs;
use super::zerv_draft::ZervDraft;
use crate::cli::utils::format_handler::InputFormatHandler;
use crate::error::ZervError;

/// Process stdin content and return a ZervDraft object
/// Expects cached stdin content (None should not happen with centralized extraction)
pub fn process_cached_stdin_source(
    _args: &VersionArgs,
    stdin_content: Option<&str>,
) -> Result<ZervDraft, ZervError> {
    let content = stdin_content.ok_or_else(|| {
        ZervError::StdinError(
            "No stdin content provided to process_cached_stdin_source".to_string(),
        )
    })?;

    // Parse stdin content as Zerv RON (includes schema)
    let zerv_from_stdin = InputFormatHandler::parse_and_validate_zerv_ron(content)?;

    // Return ZervDraft with existing schema (stdin source)
    Ok(ZervDraft::new(
        zerv_from_stdin.vars,
        Some(zerv_from_stdin.schema),
    ))
}
