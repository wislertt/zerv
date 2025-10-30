use handlebars::Handlebars;
use regex::Regex;
use serde_json::json;

pub fn assert_version_expectation(expectation: &str, actual: &str) {
    let handlebars = Handlebars::new();

    let template_data = json!({
        "commit_hash_7": "[a-f0-9]{7}"
    });

    let re: Regex = Regex::new(r"\{\{[^}]+\}\}").expect("Invalid regex for tokenization");
    let tokens: Vec<&str> = re.split(expectation).collect();
    let placeholders: Vec<_> = re.find_iter(expectation).map(|m| m.as_str()).collect();

    let mut actual_pos = 0;

    for (i, token) in tokens.iter().enumerate() {
        // Match literal token (exact match)
        if !token.is_empty() {
            assert!(
                actual.len() >= actual_pos + token.len(),
                "Expected literal '{}' at position {} in '{}'",
                token,
                actual_pos,
                actual
            );

            assert_eq!(
                &actual[actual_pos..actual_pos + token.len()],
                *token,
                "Expected '{}' at position {} but got '{}' in '{}'",
                token,
                actual_pos,
                &actual[actual_pos..actual_pos + token.len()],
                actual
            );
            actual_pos += token.len();
        }

        // Match placeholder (regex match)
        if i < placeholders.len() {
            let placeholder = placeholders[i];
            let regex_pattern = handlebars
                .render_template(placeholder, &template_data)
                .expect("Failed to render template");

            let regex = Regex::new(&regex_pattern).expect("Invalid regex pattern");

            // Determine segment length
            let segment_len = if let Some(next_token) = tokens.get(i + 1) {
                if !next_token.is_empty() {
                    // Look ahead to find next literal token
                    if let Some(pos) = actual[actual_pos..].find(next_token) {
                        pos
                    } else {
                        actual.len() - actual_pos
                    }
                } else {
                    // Consecutive placeholder, extract length from placeholder name
                    let placeholder_key = placeholder.trim_matches(&['{', '}'][..]);
                    get_fixed_length_from_placeholder_name(placeholder_key)
                }
            } else {
                // Last placeholder
                actual.len() - actual_pos
            };

            assert!(
                actual.len() >= actual_pos + segment_len,
                "Not enough characters for placeholder '{}' at position {}",
                placeholder,
                actual_pos
            );

            let actual_segment = &actual[actual_pos..actual_pos + segment_len];
            assert!(
                regex.is_match(actual_segment),
                "Expected placeholder '{}' (regex: '{}') to match '{}' at position {} in '{}'",
                placeholder,
                regex_pattern,
                actual_segment,
                actual_pos,
                actual
            );

            actual_pos += segment_len;
        }
    }

    assert_eq!(
        actual_pos,
        actual.len(),
        "Unexpected trailing characters in actual string: '{}'",
        actual
    );
}

/// Extract fixed length from placeholder name using convention
/// Examples:
/// - "commit_hash_7" -> 7
/// - "version_3" -> 3
/// - "build_id_10" -> 10
fn get_fixed_length_from_placeholder_name(placeholder_name: &str) -> usize {
    if let Some(last_part) = placeholder_name.split('_').next_back() {
        if let Ok(length) = last_part.parse::<usize>() {
            length
        } else {
            panic!(
                "Placeholder '{}' must end with '_<number>' for consecutive placeholders",
                placeholder_name
            );
        }
    } else {
        panic!(
            "Invalid placeholder name format: '{}', must contain '_' and end with number",
            placeholder_name
        );
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("0.7.74+dev.4.{{commit_hash_7}}", "0.7.74+dev.4.d4738bb")]
    #[case("1.0.0+dev.1.{{commit_hash_7}}", "1.0.0+dev.1.a1b2c3d")]
    #[case("prefix-{{commit_hash_7}}-suffix", "prefix-d4738bb-suffix")]
    #[case("{{commit_hash_7}}-middle-{{commit_hash_7}}", "d4738bb-middle-a1b2c3d")]
    #[case(
        "{{commit_hash_7}}{{commit_hash_7}}{{commit_hash_7}}",
        "d4738bba1b2c3dabc1234"
    )]
    #[case("exact-match-no-placeholders", "exact-match-no-placeholders")]
    fn test_assert_version_expectation_function(#[case] expectation: &str, #[case] actual: &str) {
        assert_version_expectation(expectation, actual);
    }

    #[rstest]
    #[case(
        "0.7.74+dev.4.{{commit_hash_7}}",
        "1.7.74+dev.4.d4738bb",
        "Expected '0.7.74+dev.4.' at position 0"
    )]
    #[case(
        "0.7.74+dev.4.{{commit_hash_7}}",
        "0.7.74+dev.4.xyz1234",
        "Expected placeholder '{{commit_hash_7}}' (regex: '[a-f0-9]{7}') to match 'xyz1234'"
    )]
    #[case(
        "{{commit_hash_7}}{{commit_hash_7}}",
        "d4738bb",
        "Not enough characters for placeholder '{{commit_hash_7}}'"
    )]
    #[case(
        "prefix-{{commit_hash_7}}",
        "prefix-d4738bb-extra",
        "Unexpected trailing characters in actual string"
    )]
    fn test_assert_version_expectation_fail_cases(
        #[case] expectation: &str,
        #[case] actual: &str,
        #[case] expected_error: &str,
    ) {
        let result = std::panic::catch_unwind(|| {
            assert_version_expectation(expectation, actual);
        });
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        let error_str = error_msg.downcast_ref::<String>().unwrap();
        assert!(error_str.contains(expected_error));
    }
}
