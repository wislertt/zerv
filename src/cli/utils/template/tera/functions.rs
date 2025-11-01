use tera::Tera;

use crate::error::ZervError;

/// Register custom Tera functions
pub fn register_functions(_tera: &mut Tera) -> Result<(), ZervError> {
    // TODO: Implement custom functions in Phase 4
    // For now, we're using Tera's built-in functionality
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_functions() {
        let mut tera = Tera::default();
        let result = register_functions(&mut tera);
        assert!(result.is_ok());
    }
}
