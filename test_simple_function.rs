#[cfg(test)]
mod test {
    use zerv::cli::utils::template::tera::{TeraTemplate, functions::register_functions};

    #[test]
    fn test_simple_tera_function() {
        let mut tera = tera::Tera::default();
        let result = register_functions(&mut tera);
        assert!(result.is_ok(), "Function registration failed: {:?}", result);

        // Try adding a simple template with custom function
        let add_result = tera.add_raw_template("test", "{{ sanitize('test') }}");
        println!("Add template result: {:?}", add_result);

        if let Err(e) = add_result {
            panic!("Failed to add template: {}", e);
        }
    }
}
