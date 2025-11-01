// NOTE: Tests temporarily disabled during Handlebars → Tera migration
// Test side-by-side comparisons between Handlebars and Tera templates
// This demonstrates that Tera can produce identical output to Handlebars
// while providing more expressive syntax.
/*
#[cfg(all(test, feature = "test-utils"))]
mod tests {
    use crate::cli::utils::template::handlebars::Template as HandlebarsTemplate;
    use crate::cli::utils::template::tera::TeraTemplate;
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::zerv::PreReleaseLabel;

    #[test]
    fn test_side_by_side_basic_version() {
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{major}}.{{minor}}.{{patch}}");
        let tera_template =
            TeraTemplate::new("{{ major }}.{{ minor }}.{{ patch }}".to_string()).unwrap();

        let zerv_fixture = ZervFixture::new().with_version(2, 5, 1);
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "2.5.1");
        assert_eq!(tera_result, "2.5.1");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_v_prefix() {
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("v{{major}}.{{minor}}.{{patch}}".to_string());
        let tera_template =
            TeraTemplate::new("v{{ major }}.{{ minor }}.{{ patch }}".to_string()).unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0);
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "v1.0.0");
        assert_eq!(tera_result, "v1.0.0");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_math_operations() {
        // Handlebars: {{add major 1}}.{{minor}}.{{patch}}
        // Tera: {{ major + 1 }}.{{ minor }}.{{ patch }}
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("v{{add major 1}}.{{minor}}.{{patch}}".to_string());
        let tera_template =
            TeraTemplate::new("v{{ major + 1 }}.{{ minor }}.{{ patch }}".to_string()).unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "v2.2.3");
        assert_eq!(tera_result, "v2.2.3");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_pre_release() {
        let hb_template: HandlebarsTemplate<String> = HandlebarsTemplate::from(
            "{{major}}.{{minor}}.{{patch}}{{#if pre_release.number}}.{{pre_release.number}}{{/if}}"
                .to_string(),
        );
        let tera_template = TeraTemplate::new(
            "{{ major }}.{{ minor }}.{{ patch }}{% if pre_release.number %}.{{ pre_release.number }}{% endif %}"
                .to_string(),
        ).unwrap();

        let zerv_fixture = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_pre_release(PreReleaseLabel::Alpha, Some(2));
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "1.0.0.2");
        assert_eq!(tera_result, "1.0.0.2");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_pre_release_without_number() {
        let hb_template: HandlebarsTemplate<String> = HandlebarsTemplate::from(
            "{{major}}.{{minor}}.{{patch}}{{#if pre_release.label}}-{{pre_release.label}}{{/if}}"
                .to_string(),
        );
        let tera_template = TeraTemplate::new(
            "{{ major }}.{{ minor }}.{{ patch }}{% if pre_release %}-{{ pre_release.label }}{% endif %}"
                .to_string(),
        ).unwrap();

        let zerv_fixture = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_pre_release(PreReleaseLabel::Beta, None);
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "1.0.0-beta");
        assert_eq!(tera_result, "1.0.0-beta");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_vcs_info() {
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{major}}.{{minor}}.{{patch}}+{{distance}}".to_string());
        let tera_template =
            TeraTemplate::new("{{ major }}.{{ minor }}.{{ patch }}+{{ distance }}".to_string())
                .unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            Some(7),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "1.2.3+7");
        assert_eq!(tera_result, "1.2.3+7");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_dirty_suffix() {
        // Handlebars: {{#if dirty}}-dirty{{/if}}
        // Tera: {% if dirty %}-dirty{% endif %}
        let hb_template: HandlebarsTemplate<String> = HandlebarsTemplate::from(
            "{{major}}.{{minor}}.{{patch}}{{#if dirty}}-dirty{{/if}}".to_string(),
        );
        let tera_template = TeraTemplate::new(
            "{{ major }}.{{ minor }}.{{ patch }}{% if dirty %}-dirty{% endif %}".to_string(),
        )
        .unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            Some(0),
            Some(true),
            None,
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "1.0.0-dirty");
        assert_eq!(tera_result, "1.0.0-dirty");
        assert_eq!(hb_result, tera_result);

        // Test clean case
        let zerv_fixture_clean = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            Some(0),
            Some(false),
            None,
            None,
            None,
            None,
            None,
        );
        let zerv_clean = zerv_fixture_clean.zerv();

        let hb_result_clean = hb_template.resolve(Some(zerv_clean)).unwrap().unwrap();
        let tera_result_clean = tera_template.render(zerv_clean).unwrap();

        assert_eq!(hb_result_clean, "1.0.0");
        assert_eq!(tera_result_clean, "1.0.0");
        assert_eq!(hb_result_clean, tera_result_clean);
    }

    #[test]
    fn test_side_by_side_hash_short() {
        let hb_template: HandlebarsTemplate<String> = HandlebarsTemplate::from(
            "{{major}}.{{minor}}.{{patch}}+{{bumped_commit_hash_short}}".to_string(),
        );
        let tera_template = TeraTemplate::new(
            "{{ major }}.{{ minor }}.{{ patch }}+{{ bumped_commit_hash_short }}".to_string(),
        )
        .unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            Some(0),
            None,
            None,
            Some("abcdef123456".to_string()),
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "1.0.0+abcdef1");
        assert_eq!(tera_result, "1.0.0+abcdef1");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_custom_json() {
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{major}}.{{minor}}.{{patch}}-{{custom.build}}".to_string());
        let tera_template =
            TeraTemplate::new("{{ major }}.{{ minor }}.{{ patch }}-{{ custom.build }}".to_string())
                .unwrap();

        let vars = crate::version::zerv::ZervVars {
            major: Some(2),
            minor: Some(1),
            patch: Some(0),
            custom: serde_json::json!({
                "build": "42",
                "env": "prod"
            }),
            ..Default::default()
        };
        let schema = crate::version::zerv::ZervSchema::semver_default().unwrap();
        let zerv = crate::version::zerv::Zerv::new(schema, vars).unwrap();

        let hb_result = hb_template.resolve(Some(&zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(&zerv).unwrap();

        assert_eq!(hb_result, "2.1.0-42");
        assert_eq!(tera_result, "2.1.0-42");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_tera_advantages_math_expressions() {
        // This demonstrates Tera's advantage: complex math expressions
        // that would require nested Handlebars helpers

        // Tera can do complex expressions in one line
        let tera_template = TeraTemplate::new(
            "{{ (major * 100 + minor * 10 + patch) }}-{{ distance | default(value=0) + 1 }}"
                .to_string(),
        )
        .unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            Some(4),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let result = tera_template.render(zerv).unwrap();
        assert_eq!(result, "123-5");

        // Same result with Tera's default filter for distance when it's None
        let zerv_fixture_no_distance = ZervFixture::new().with_version(1, 2, 3);
        let zerv_no_distance = zerv_fixture_no_distance.zerv();

        let result_no_distance = tera_template.render(zerv_no_distance).unwrap();
        assert_eq!(result_no_distance, "123-1");
    }

    #[test]
    fn test_tera_advances_conditional_logic() {
        // This demonstrates Tera's richer conditional logic
        let tera_template = TeraTemplate::new(
            "{% if major == 0 %}v0.{{ minor }}.{{ patch }}{% elif dirty %}{{ major }}.{{ minor }}.{{ patch }}-dirty{% elif distance and distance > 0 %}{{ major }}.{{ minor }}.{{ patch }}+{{ distance }}{% else %}{{ major }}.{{ minor }}.{{ patch }}{% endif %}"
                .to_string(),
        ).unwrap();

        // Test each condition
        let cases = vec![
            // (major, minor, patch, distance, dirty, expected)
            (0, 1, 0, Some(0), Some(false), "v0.1.0"),
            (1, 2, 3, Some(0), Some(true), "1.2.3-dirty"),
            (1, 2, 3, Some(15), Some(false), "1.2.3+15"),
            (1, 2, 3, Some(5), Some(false), "1.2.3+5"),
        ];

        for (major, minor, patch, distance, dirty, expected) in cases {
            let mut zerv_fixture = ZervFixture::new().with_version(major, minor, patch);
            zerv_fixture =
                zerv_fixture.with_vcs_data(distance, dirty, None, None, None, None, None);
            let zerv = zerv_fixture.zerv();

            let result = tera_template.render(zerv).unwrap();
            assert_eq!(
                result, expected,
                "Failed for major={}, distance={:?}, dirty={:?}",
                major, distance, dirty
            );
        }
    }

    #[test]
    fn test_tera_string_concatenation_advantage() {
        // Tera's string concatenation with filters is more expressive
        let tera_template = TeraTemplate::new(
            "{{ major }}.{{ minor }}.{{ patch }}{% if pre_release %}-{{ pre_release.label }}.{{ pre_release.number | default(value=0) }}{% endif %}+{{ bumped_commit_hash_short | default(value='unknown') }}"
                .to_string(),
        ).unwrap();

        // Test with pre-release
        let zerv_fixture = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_vcs_data(
                Some(0),
                None,
                None,
                Some("abc123def".to_string()),
                None,
                None,
                None,
            );
        let zerv = zerv_fixture.zerv();

        let result = tera_template.render(zerv).unwrap();
        assert_eq!(result, "1.0.0-alpha.1+abc123d");

        // Test without pre-release
        let zerv_fixture_clean = ZervFixture::new().with_version(2, 0, 0);
        let zerv_clean = zerv_fixture_clean.zerv();

        let result_clean = tera_template.render(zerv_clean).unwrap();
        assert_eq!(result_clean, "2.0.0+unknown");
    }

    #[test]
    fn test_simple_tera_function_debug() {
        let mut tera = tera::Tera::default();
        let result = crate::cli::utils::template::tera::functions::register_functions(&mut tera);
        assert!(result.is_ok(), "Function registration failed: {:?}", result);

        // Try adding a simple template with custom function
        let add_result = tera.add_raw_template("test", "{{ sanitize(value=value) }}");
        println!("Add template result: {:?}", add_result);

        if let Err(e) = add_result {
            panic!("Failed to add template: {}", e);
        }
    }

    #[test]
    fn test_side_by_side_sanitize_helper() {
        // Handlebars: {{sanitize bumped_branch preset='dotted'}}
        // Tera: {{ sanitize(bumped_branch, preset="dotted") }}
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{sanitize bumped_branch preset='dotted'}}".to_string());
        let tera_template =
            match TeraTemplate::new("{{ sanitize(value=bumped_branch) }}".to_string()) {
                Ok(template) => template,
                Err(e) => {
                    println!("Tera template error: {:?}", e);
                    panic!("Template creation failed");
                }
            };

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            Some("feature/test-branch".to_string()),
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "feature.test.branch");
        assert_eq!(tera_result, "feature.test.branch");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_sanitize_helper_pep440() {
        // Handlebars: {{sanitize bumped_branch preset='pep440'}}
        // Tera: {{ sanitize(bumped_branch, preset="pep440") }}
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{sanitize bumped_branch preset='pep440'}}".to_string());
        let tera_template =
            TeraTemplate::new("{{ sanitize(value=bumped_branch, preset=\"pep440\") }}".to_string())
                .unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            Some("feature/test-branch".to_string()),
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "feature.test.branch");
        assert_eq!(tera_result, "feature.test.branch");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_sanitize_helper_uint() {
        // Handlebars: {{sanitize bumped_branch preset='uint'}}
        // Tera: {{ sanitize(bumped_branch, preset="uint") }}
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{sanitize bumped_branch preset='uint'}}".to_string());
        let tera_template =
            TeraTemplate::new("{{ sanitize(value=bumped_branch, preset=\"uint\") }}".to_string())
                .unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            Some("12345".to_string()),
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "12345");
        assert_eq!(tera_result, "12345");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_hash_helper() {
        // Handlebars: {{hash bumped_branch 7}}
        // Tera: {{ hash(bumped_branch, length=7) }}
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{hash bumped_branch 7}}".to_string());
        let tera_template =
            TeraTemplate::new("{{ hash(value=bumped_branch, length=7) }}".to_string()).unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            Some("main".to_string()),
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        // Both should produce 7-character hex strings
        assert_eq!(hb_result.len(), 7);
        assert_eq!(tera_result.len(), 7);
        assert!(hb_result.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(tera_result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_side_by_side_hash_int_helper() {
        // Handlebars: {{hash_int bumped_branch 10 true}}
        // Tera: {{ hash_int(bumped_branch, length=10, allow_leading_zero=true) }}
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{hash_int bumped_branch 10 true}}".to_string());
        let tera_template = TeraTemplate::new(
            "{{ hash_int(value=bumped_branch, length=10, allow_leading_zero=true) }}".to_string(),
        )
        .unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            Some("main".to_string()),
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        // Both should produce 10-character hex strings with leading zeros allowed
        assert_eq!(hb_result.len(), 10);
        assert_eq!(tera_result.len(), 10);
        assert!(hb_result.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(tera_result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_side_by_side_prefix_helper() {
        // Handlebars: {{prefix bumped_branch 5}}
        // Tera: {{ prefix(bumped_branch, length=5) }}
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{prefix bumped_branch 5}}".to_string());
        let tera_template =
            TeraTemplate::new("{{ prefix(value=bumped_branch, length=5) }}".to_string()).unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            Some("feature-branch".to_string()),
            None,
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        assert_eq!(hb_result, "featu");
        assert_eq!(tera_result, "featu");
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_timestamp_helper() {
        // Handlebars: {{format_timestamp bumped_timestamp}}
        // Tera: {{ format_timestamp(bumped_timestamp, format="%Y-%m-%d") }}
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{format_timestamp last_timestamp}}".to_string());
        let tera_template = TeraTemplate::new(
            "{{ format_timestamp(value=last_timestamp, format=\"%Y-%m-%d\") }}".to_string(),
        )
        .unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            Some(0),
            None,
            None,
            None,
            None,
            Some(1698675600),
            None,
        );
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        // Both should produce date in YYYY-MM-DD format
        assert!(hb_result.contains("2023-10-30"));
        assert!(tera_result.contains("2023-10-30"));
        assert_eq!(hb_result, tera_result);
    }

    #[test]
    fn test_side_by_side_math_helper_add() {
        // Handlebars: {{add major 1}}
        // Tera: {{ major + 1 }}
        let hb_template: HandlebarsTemplate<String> =
            HandlebarsTemplate::from("{{add major 1}}".to_string());
        let tera_template = TeraTemplate::new("{{ major + 1 }}".to_string()).unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv = zerv_fixture.zerv();

        let hb_result = hb_template.resolve(Some(zerv)).unwrap().unwrap();
        let tera_result = tera_template.render(zerv).unwrap();

        // Both should return the mathematical result of major + 1
        assert_eq!(hb_result, "2");
        assert_eq!(tera_result, "2");
    }

    #[test]
    fn test_tera_custom_functions_complex_usage() {
        // Test complex usage of Tera custom functions
        // This demonstrates how Tera can combine custom functions with built-in operations
        let tera_template = TeraTemplate::new(
            "v{{ major }}.{{ minor }}.{{ patch }}-{{ sanitize(value=bumped_branch, preset=\"dotted\") }}+{{ hash(value=bumped_commit_hash_short, length=8) }}"
                .to_string(),
        ).unwrap();

        let zerv_fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            Some(0),
            None,
            Some("feature/test-branch".to_string()),
            Some("abcdef123456789".to_string()),
            None,
            None,
            None,
        );
        let zerv = zerv_fixture.zerv();

        let result = tera_template.render(zerv).unwrap();

        assert_eq!(result, "v1.2.3-feature.test.branch+11e0e658");
    }
}
*/
