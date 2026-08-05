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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::version::VersionArgs;
    use crate::error::ZervError;

    #[test]
    fn process_cached_stdin_source_returns_error_when_no_content() {
        let args = VersionArgs::default();
        let result = process_cached_stdin_source(&args, None);
        let err = result.unwrap_err();
        assert!(matches!(err, ZervError::StdinError(_)), "got {err:?}");
        assert!(err.to_string().contains("No stdin content"));
    }
}
