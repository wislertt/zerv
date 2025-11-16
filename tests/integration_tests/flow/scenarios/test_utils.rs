// Test utilities for flow scenario integration tests
// Reuses utilities from src/cli/flow/test_utils.rs

use zerv::cli::utils::template::{
    Template,
    TemplateExtGeneric,
};
use zerv::schema::schema_preset_names::*;
use zerv::version::pep440::utils::pre_release_label_to_pep440_string;
use zerv::version::zerv::PreReleaseLabel;

/// Generates a branch hash and asserts it matches the expected value
pub fn expect_branch_hash(branch_name: &str, length: usize, expected_hash: &str) -> String {
    let hash = Template::<u32>::new(format!(
        "{{{{ hash_int(value='{}', length={}) }}}}",
        branch_name, length
    ))
    .render_unwrap(None);
    let hash_str = hash.to_string();
    assert_eq!(
        hash_str, expected_hash,
        "Hash generation for branch '{}' with length {} failed",
        branch_name, length
    );
    hash_str
}

// Test case structure for better readability and type safety
#[derive(Debug, Clone)]
pub struct SchemaTestCase {
    pub name: &'static str,
    pub semver_expectation: String,
    pub pep440_expectation: String,
}

/// Creates comprehensive test cases for ALL standard-related schema constants
pub struct SchemaTestExtraCore<'a> {
    pub pre_release_label: PreReleaseLabel,
    pub pre_release_num: &'a str,
    pub post: u32,
    pub dev: Option<&'a str>,
}

pub struct SchemaTestBuild<'a> {
    pub sanitized_branch_name: &'a str,
    pub distance: u32,
    pub include_build_for_standard: bool,
}

pub fn create_full_schema_test_cases(
    base_version: &str,
    extra_core: SchemaTestExtraCore,
    build: SchemaTestBuild,
) -> Vec<SchemaTestCase> {
    let semver_label = extra_core.pre_release_label.label_str();
    let pep440_label = pre_release_label_to_pep440_string(&extra_core.pre_release_label);

    let semver_dev = match extra_core.dev {
        Some(dev_str) => format!(".dev.{}", dev_str),
        None => String::new(),
    };
    let pep440_dev = match extra_core.dev {
        Some(dev_str) => format!(".dev{}", dev_str),
        None => String::new(),
    };

    vec![
        // Base schemas
        SchemaTestCase {
            name: STANDARD_BASE,
            semver_expectation: base_version.to_string(),
            pep440_expectation: base_version.to_string(),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE,
            semver_expectation: format!(
                "{}-{}.{}",
                base_version, semver_label, extra_core.pre_release_num
            ),
            pep440_expectation: format!(
                "{}{}{}",
                base_version, pep440_label, extra_core.pre_release_num
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST,
            semver_expectation: format!(
                "{}-{}.{}.post.{}",
                base_version, semver_label, extra_core.pre_release_num, extra_core.post
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}",
                base_version, pep440_label, extra_core.pre_release_num, extra_core.post
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_DEV,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}",
                base_version, semver_label, extra_core.pre_release_num, extra_core.post, semver_dev
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}",
                base_version, pep440_label, extra_core.pre_release_num, extra_core.post, pep440_dev
            ),
        },
        // Context schemas
        SchemaTestCase {
            name: STANDARD_BASE_CONTEXT,
            semver_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, build.sanitized_branch_name, build.distance
            ),
            pep440_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, build.sanitized_branch_name, build.distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                extra_core.pre_release_num,
                build.sanitized_branch_name,
                build.distance
            ),
            pep440_expectation: format!(
                "{}{}{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                extra_core.pre_release_num,
                build.sanitized_branch_name,
                build.distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                extra_core.pre_release_num,
                extra_core.post,
                build.sanitized_branch_name,
                build.distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                extra_core.pre_release_num,
                extra_core.post,
                build.sanitized_branch_name,
                build.distance
            ),
        },
        SchemaTestCase {
            name: STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                extra_core.pre_release_num,
                extra_core.post,
                semver_dev,
                build.sanitized_branch_name,
                build.distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                extra_core.pre_release_num,
                extra_core.post,
                pep440_dev,
                build.sanitized_branch_name,
                build.distance
            ),
        },
        // Complete schemas
        SchemaTestCase {
            name: STANDARD_NO_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}",
                base_version, semver_label, extra_core.pre_release_num, extra_core.post, semver_dev
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}",
                base_version, pep440_label, extra_core.pre_release_num, extra_core.post, pep440_dev
            ),
        },
        SchemaTestCase {
            name: STANDARD_CONTEXT,
            semver_expectation: format!(
                "{}-{}.{}.post.{}{}+{}.{}.g{{hex:7}}",
                base_version,
                semver_label,
                extra_core.pre_release_num,
                extra_core.post,
                semver_dev,
                build.sanitized_branch_name,
                build.distance
            ),
            pep440_expectation: format!(
                "{}{}{}.post{}{}+{}.{}.g{{hex:7}}",
                base_version,
                pep440_label,
                extra_core.pre_release_num,
                extra_core.post,
                pep440_dev,
                build.sanitized_branch_name,
                build.distance
            ),
        },
        SchemaTestCase {
            name: STANDARD,
            semver_expectation: if build.include_build_for_standard {
                format!(
                    "{}-{}.{}.post.{}{}+{}.{}.g{{hex:7}}",
                    base_version,
                    semver_label,
                    extra_core.pre_release_num,
                    extra_core.post,
                    semver_dev,
                    build.sanitized_branch_name,
                    build.distance
                )
            } else {
                format!(
                    "{}-{}.{}.post.{}{}",
                    base_version,
                    semver_label,
                    extra_core.pre_release_num,
                    extra_core.post,
                    semver_dev
                )
            },
            pep440_expectation: if build.include_build_for_standard {
                format!(
                    "{}{}{}.post{}{}+{}.{}.g{{hex:7}}",
                    base_version,
                    pep440_label,
                    extra_core.pre_release_num,
                    extra_core.post,
                    pep440_dev,
                    build.sanitized_branch_name,
                    build.distance
                )
            } else {
                format!(
                    "{}{}{}.post{}{}",
                    base_version,
                    pep440_label,
                    extra_core.pre_release_num,
                    extra_core.post,
                    pep440_dev
                )
            },
        },
    ]
}

pub fn create_base_schema_test_cases(
    base_version: &str,
    sanitized_branch_name: &str,
) -> Vec<SchemaTestCase> {
    let distance = 0;
    let mut test_cases = Vec::new();

    let no_context_schemas = vec![
        STANDARD,
        STANDARD_BASE,
        STANDARD_BASE_PRERELEASE,
        STANDARD_BASE_PRERELEASE_POST,
        STANDARD_BASE_PRERELEASE_POST_DEV,
        STANDARD_NO_CONTEXT,
    ];

    let context_schemas = vec![
        STANDARD_BASE_CONTEXT,
        STANDARD_BASE_PRERELEASE_CONTEXT,
        STANDARD_BASE_PRERELEASE_POST_CONTEXT,
        STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT,
        STANDARD_CONTEXT,
    ];

    // Test no context schemas with base version only
    for schema_name in &no_context_schemas {
        test_cases.push(SchemaTestCase {
            name: schema_name,
            semver_expectation: base_version.to_string(),
            pep440_expectation: base_version.to_string(),
        });
    }

    // Test context schemas with context
    for schema_name in &context_schemas {
        test_cases.push(SchemaTestCase {
            name: schema_name,
            semver_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, sanitized_branch_name, distance
            ),
            pep440_expectation: format!(
                "{}+{}.{}.g{{hex:7}}",
                base_version, sanitized_branch_name, distance
            ),
        });
    }

    test_cases
}
