use crate::error::ZervError;

pub struct BoolResolution;

impl BoolResolution {
    #[inline]
    pub fn resolve_opposing_flags(affirmative: bool, negative: bool) -> Option<bool> {
        match (affirmative, negative) {
            (true, false) => Some(true),  // --flag
            (false, true) => Some(false), // --no-flag
            (false, false) => None,       // neither (use default)
            (true, true) => {
                unreachable!("Both opposing flags were true - validate flags first")
            }
        }
    }

    pub fn validate_opposing_flags(
        affirmative: bool,
        negative: bool,
        flag_name: &str,
    ) -> Result<(), ZervError> {
        if affirmative && negative {
            return Err(ZervError::ConflictingOptions(format!(
                "Cannot use --{} with --no-{}",
                flag_name, flag_name
            )));
        }
        Ok(())
    }

    pub fn validate_and_resolve(
        affirmative: bool,
        negative: bool,
        flag_name: &str,
    ) -> Result<Option<bool>, ZervError> {
        Self::validate_opposing_flags(affirmative, negative, flag_name)?;
        Ok(Self::resolve_opposing_flags(affirmative, negative))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_opposing_flags() {
        assert_eq!(
            BoolResolution::resolve_opposing_flags(true, false),
            Some(true)
        );
        assert_eq!(
            BoolResolution::resolve_opposing_flags(false, true),
            Some(false)
        );
        assert_eq!(BoolResolution::resolve_opposing_flags(false, false), None);
    }

    #[test]
    #[should_panic(expected = "Both opposing flags were true")]
    fn test_resolve_opposing_flags_panics_on_both_true() {
        BoolResolution::resolve_opposing_flags(true, true);
    }

    #[test]
    fn test_validate_opposing_flags() {
        assert!(BoolResolution::validate_opposing_flags(true, false, "dirty").is_ok());
        assert!(BoolResolution::validate_opposing_flags(false, true, "dirty").is_ok());
        assert!(BoolResolution::validate_opposing_flags(false, false, "dirty").is_ok());

        let result = BoolResolution::validate_opposing_flags(true, true, "dirty");
        assert!(result.is_err());

        if let Err(ZervError::ConflictingOptions(msg)) = result {
            assert!(msg.contains("Cannot use --dirty with --no-dirty"));
        } else {
            panic!("Expected ConflictingOptions error");
        }
    }

    #[test]
    fn test_validate_and_resolve() {
        assert_eq!(
            BoolResolution::validate_and_resolve(true, false, "dirty").unwrap(),
            Some(true)
        );
        assert_eq!(
            BoolResolution::validate_and_resolve(false, true, "dirty").unwrap(),
            Some(false)
        );
        assert_eq!(
            BoolResolution::validate_and_resolve(false, false, "dirty").unwrap(),
            None
        );

        let result = BoolResolution::validate_and_resolve(true, true, "dirty");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_and_resolve_different_flag_names() {
        let result = BoolResolution::validate_and_resolve(true, true, "bump-context");
        assert!(result.is_err());

        if let Err(ZervError::ConflictingOptions(msg)) = result {
            assert!(msg.contains("Cannot use --bump-context with --no-bump-context"));
        } else {
            panic!("Expected ConflictingOptions error");
        }
    }

    #[test]
    fn test_error_message_formatting() {
        let test_cases = ["dirty", "bump-context", "clean", "verbose"];

        for flag_name in test_cases {
            let result = BoolResolution::validate_opposing_flags(true, true, flag_name);
            assert!(result.is_err());

            if let Err(ZervError::ConflictingOptions(msg)) = result {
                let expected = format!("Cannot use --{} with --no-{}", flag_name, flag_name);
                assert_eq!(msg, expected);
            } else {
                panic!("Expected ConflictingOptions error for flag: {}", flag_name);
            }
        }
    }
}
