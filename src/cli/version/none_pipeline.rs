use super::zerv_draft::ZervDraft;
use crate::error::ZervError;
use crate::version::zerv::vars::ZervVars;

/// Process none source (no VCS) and return a ZervDraft object
/// All version data comes from CLI overrides (--tag-version, --distance, etc.)
pub fn process_none_source() -> Result<ZervDraft, ZervError> {
    // Create empty ZervVars with all default (None) values
    // Overrides will be applied later via ZervDraft::to_zerv()
    let vars = ZervVars::default();

    // Return ZervDraft without schema (none source)
    Ok(ZervDraft::new(vars, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_none_source_basic() {
        let result = process_none_source();
        assert!(result.is_ok());

        let draft = result.unwrap();
        // All fields should be None (defaults)
        assert!(draft.vars.major.is_none());
        assert!(draft.vars.minor.is_none());
        assert!(draft.vars.patch.is_none());
        assert!(draft.vars.distance.is_none());
        assert!(draft.vars.dirty.is_none());
        assert!(draft.vars.last_tag_version.is_none());
    }

    #[test]
    fn test_process_none_source_no_schema() {
        let draft = process_none_source().unwrap();
        // None source doesn't provide a schema
        assert!(draft.schema.is_none());
    }
}
