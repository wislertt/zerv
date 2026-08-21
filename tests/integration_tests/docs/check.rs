use crate::util::TestCommand;

#[test]
fn test_check_page_documentation_examples() {
    let output = TestCommand::run("check 1.2.3-rc.2");
    assert_eq!(
        output,
        "Version: 1.2.3-rc.2\n✓ Valid PEP440 format (normalized: 1.2.3rc2)\n✓ Valid SemVer format"
    );
}
