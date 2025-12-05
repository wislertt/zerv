// use zerv::cli::flow::test_utils::expect_branch_hash;

use chrono::Utc;
use rstest::rstest;

use crate::integration_tests::flow::docs::test_utils::TestScenario;
// use crate::util::TestCommand;

#[test]
fn test_io_documentation_examples() {
    let branch_name = "branch-name".to_string();
    // let branch_name_hash = expect_branch_hash(&branch_name, 5, "10192");
    let dirty_feature_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch(&branch_name)
        .checkout(&branch_name)
        .commit()
        .make_dirty();

    // Test case 1
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin",
        "1.0.1-alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 2
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command_contains(
        "flow --source stdin --output-format zerv",
        &[
            "schema:",
            "core:",
            "var(Major)",
            "var(Minor)",
            "var(Patch)",
            "extra_core:",
            "var(Epoch)",
            "var(PreRelease)",
            "vars:",
            "major: Some(1)",
            "minor: Some(0)",
            "patch: Some(1)",
            "pre_release:",
            "bumped_branch: Some(\"branch-name\")",
            "distance: Some(1)",
        ],
    );

    // Test case 3
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_commands(
        &[
            "flow --source stdin --output-format zerv",
            "version --source stdin --major 4 --output-format semver",
        ],
        "4.0.1-alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 4
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-format pep440",
        "1.0.1a10192.post1.dev{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 5
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-format semver",
        "1.0.1-alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 6
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-prefix v --output-format semver",
        "v1.0.1-alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 7
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"app:{{ major }}.{{ minor }}.{{ patch }}\"",
        "app:1.0.1",
    );

    // Test case 8
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"{{ semver_obj.docker }}\"",
        "1.0.1-alpha.10192.post.1.dev.{timestamp:now}-branch.name.1.g{hex:7}",
    );

    // Test case 9
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"{{ semver_obj.base_part }}++{{ semver_obj.pre_release_part }}++{{ semver_obj.build_part }}\"",
        "1.0.1++alpha.10192.post.1.dev.{timestamp:now}++branch.name.1.g{hex:7}",
    );

    // Test case 10
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"Build: {{ major }}.{{ minor }}.{{ patch }}-{{ pre_release.label | default(value='release') }}{% if pre_release.number %}{{ pre_release.number }}{% endif %} ({{ bumped_branch }}@{{ bumped_commit_hash_short }})\"",
        "Build: 1.0.1-alpha10192 (branch-name@g{hex:7})",
    );

    // Test case 11
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"Version: {{ semver_obj.docker }}, Branch: {{ bumped_branch | upper }}, Clean: {% if dirty %}No{% else %}Yes{% endif %}\"",
        "Version: 1.0.1-alpha.10192.post.1.dev.{timestamp:now}-branch.name.1.g{hex:7}, Branch: BRANCH-NAME, Clean: No",
    );

    // Test case 12
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"{% if distance %}{{ distance }} commits since {% if last_timestamp %}{{ format_timestamp(value=last_timestamp, format='%Y-%m-%d') }}{% else %}beginning{% endif %}{% else %}Exact tag{% endif %}\"",
        &format!("1 commits since {}", Utc::now().format("%Y-%m-%d")),
    );

    // Test case 13
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"App-{{ major }}{{ minor }}{{ patch }}{% if pre_release %}-{{ pre_release.label }}{% endif %}{% if dirty %}-SNAPSHOT{% endif %}-{{ hash(value=bumped_branch, length=4) }}\"",
        "App-101-alpha-SNAPSHOT-{hex:4}",
    );

    // Test case 14
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"PEP440: {{ pep440 }}\"",
        "PEP440: 1.0.1a10192.post1.dev{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 15
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"Release: v{{ major }}.{{ minor }}.{{ patch }}, Pre: {{ pre_release.label_code | default(value='release') }}, Hash: {{ bumped_commit_hash_short }}\"",
        "Release: v1.0.1, Pre: a, Hash: g{hex:7}",
    );

    _ = dirty_feature_branch_scenario;
}

#[rstest]
// Core Version Fields
#[case("{{ major }}", "1")]
#[case("{{ minor }}", "0")]
#[case("{{ patch }}", "1")]
#[case("{{ epoch }}", "5")]
#[case("{{ post }}", "1")]
#[case("{{ dev }}", "{timestamp:now}")]
// Pre-release Context
#[case("{{ pre_release }}", "[object]")]
#[case("{{ pre_release.label }}", "alpha")]
#[case("{{ pre_release.number }}", "10192")]
#[case("{{ pre_release.label_code }}", "a")]
#[case("{{ pre_release.label_pep440 }}", "a")]
// VCS/Metadata Fields
#[case("{{ distance }}", "1")]
#[case("{{ dirty }}", "true")]
#[case("{{ bumped_branch }}", "branch-name")]
#[case("{{ bumped_commit_hash }}", "g{hex:40}")]
#[case("{{ bumped_commit_hash_short }}", "g{hex:7}")]
#[case("{{ last_commit_hash }}", "g{hex:40}")]
#[case("{{ last_commit_hash_short }}", "g{hex:7}")]
#[case("{{ last_timestamp }}", "{timestamp:now}")]
// Parsed Version Objects
#[case("{{ semver_obj.base_part }}", "1.0.1")]
#[case(
    "{{ semver_obj.pre_release_part }}",
    "epoch.5.alpha.10192.post.1.dev.{timestamp:now}"
)]
#[case("{{ semver_obj.build_part }}", "branch.name.1.g{hex:7}")]
#[case(
    "{{ semver_obj.docker }}",
    "1.0.1-epoch.5.alpha.10192.post.1.dev.{timestamp:now}-branch.name.1.g{hex:7}"
)]
#[case("{{ pep440_obj.base_part }}", "5!1.0.1")]
#[case("{{ pep440_obj.pre_release_part }}", "a10192.post1.dev{timestamp:now}")]
#[case("{{ pep440_obj.build_part }}", "branch.name.1.g{hex:7}")]
// Formatted Versions
#[case(
    "{{ semver }}",
    "1.0.1-epoch.5.alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}"
)]
#[case(
    "{{ pep440 }}",
    "5!1.0.1a10192.post1.dev{timestamp:now}+branch.name.1.g{hex:7}"
)]
#[case("{{ current_timestamp }}", "{timestamp:now}")]
// Custom Template Functions - String Manipulation
#[case("{{ sanitize(value=bumped_branch, preset='dotted') }}", "branch.name")]
#[case(
    "{{ sanitize(value=bumped_branch, separator='*', lowercase=true, max_length=10) }}",
    "branch*nam"
)]
#[case("{{ prefix(value=bumped_branch, length=2) }}", "br")]
#[case("{{ prefix_if(value=bumped_branch, prefix='+') }}", "+branch-name")]
#[case("{{ prefix_if(value='', prefix='+') }}", "")]
// Custom Template Functions - Hashing & Formatting
#[case("{{ hash(value=bumped_branch, length=7) }}", "8d721e2")]
#[case("{{ hash_int(value=bumped_branch, length=7) }}", "1019224")]
#[case(
    "{{ format_timestamp(value=current_timestamp, format='%Y-%m-%d') }}",
    Utc::now().format("%Y-%m-%d").to_string()
)]
#[case(
    "{{ format_timestamp(value=current_timestamp, format='compact_date') }}",
    Utc::now().format("%Y%m%d").to_string()
)]
fn test_template_documentation_examples(
    #[case] output_template: &str,
    #[case] expected_output: String,
) {
    let branch_name = "branch-name".to_string();
    // let branch_name_hash = expect_branch_hash(&branch_name, 5, "10192");
    let dirty_feature_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0-epoch.5")
        .create_branch(&branch_name)
        .checkout(&branch_name)
        .commit()
        .make_dirty();

    let _ = dirty_feature_branch_scenario.assert_command(
        &format!(
            "flow --source stdin --output-template \"{}\"",
            output_template
        ),
        &expected_output,
    );
}
