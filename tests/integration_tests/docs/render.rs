use crate::util::TestCommand;

#[test]
fn test_render_page_format_conversion_examples() {
    let output = TestCommand::run(r#"render "1.2.3-alpha.1" --output-format pep440"#);
    assert_eq!(output, "1.2.3a1");

    let output =
        TestCommand::run(r#"render "1.2.3b2" --input-format pep440 --output-format semver"#);
    assert_eq!(output, "1.2.3-beta.2");

    let output = TestCommand::run(r#"render "1.2.3a1" --output-format semver"#);
    assert_eq!(output, "1.2.3-alpha.1");
}

#[test]
fn test_render_page_template_examples() {
    let output = TestCommand::run(r#"render "1.2.3" --output-template "v{{major}}.{{minor}}""#);
    assert_eq!(output, "v1.2");

    let output = TestCommand::run(
        r#"render "2.0.0-beta.2" --output-template "{{major}}.{{minor}}.{{patch}}-{{pre_release.label}}""#,
    );
    assert_eq!(output, "2.0.0-beta");
}

#[test]
fn test_render_page_prefix_examples() {
    let output = TestCommand::run(r#"render "1.2.3" --output-prefix "v""#);
    assert_eq!(output, "v1.2.3");
}
