use super::FlowArgs;
use crate::cli::utils::template::Template;
use crate::utils::constants::post_modes;

impl FlowArgs {
    pub fn build_patch_bump_template(&self, content: &str) -> String {
        let if_part = "{% if not pre_release and (dirty or distance) %}";
        let else_part = "{% else %}None{% endif %}";
        if_part.to_string() + content + else_part
    }

    pub fn build_pre_release_bump_template(&self, content: &str) -> String {
        let if_part = "{% if dirty or distance %}";
        let else_part = "{% else %}None{% endif %}";
        if_part.to_string() + content + else_part
    }

    pub fn bump_pre_release_label(&self) -> Option<Template<String>> {
        self.branch_config.pre_release_label.clone().map(|label| {
            let template = self.build_pre_release_bump_template(&label);
            Template::new(template)
        })
    }

    pub fn bump_pre_release_num(&self) -> Option<Option<Template<u32>>> {
        if self.branch_config.pre_release_label.is_none() {
            None
        } else {
            let hash_len = self.hash_branch_len.to_string();

            let pre_release_num_content = if let Some(num) = self.branch_config.pre_release_num {
                num.to_string()
            } else {
                format!(
                    "{{{{ hash_int(value=bumped_branch, length={}) }}}}",
                    hash_len
                )
            };

            let template = self.build_pre_release_bump_template(&pre_release_num_content);

            Some(Some(Template::new(template)))
        }
    }

    pub fn bump_patch(&self) -> Option<Option<Template<u32>>> {
        let template = self.build_patch_bump_template("1");
        Some(Some(Template::new(template)))
    }

    pub fn bump_post(&self) -> Option<Option<Template<u32>>> {
        let content = match self
            .branch_config
            .post_mode
            .as_deref()
            .unwrap_or(post_modes::COMMIT)
        {
            post_modes::COMMIT => "{{ distance }}", // bump post by distance
            post_modes::TAG => "1",                 // bump post by 1
            _ => unreachable!("Invalid post_mode should have been caught by validation"),
        };
        let template = self.build_pre_release_bump_template(content);
        Some(Some(Template::new(template)))
    }

    pub fn bump_dev(&self) -> Option<Option<Template<u32>>> {
        let if_part = "{% if dirty %}";
        let content = "{{ bumped_timestamp }}";
        let else_part = "{% else %}None{% endif %}";
        let template = format!("{}{}{}", if_part, content, else_part);
        Some(Some(Template::new(template)))
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::cli::flow::args::branch_rules::BranchRulesConfig;
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::zerv::core::Zerv;

    /// Helper function to create a mock zerv object for tests
    fn mock_zerv() -> Zerv {
        let mut zerv = ZervFixture::new().build();
        // Set a mock branch name for tests that need branch detection
        zerv.vars.last_branch = Some("main".to_string());
        zerv
    }

    mod bump_pre_release_label {
        use super::*;

        #[test]
        fn test_default_returns_alpha() {
            let mut args = FlowArgs::default();
            args.validate(&mock_zerv()).unwrap(); // This sets the default pre_release_label
            let expected = args.build_pre_release_bump_template("alpha");
            assert_eq!(args.bump_pre_release_label(), Some(Template::new(expected)));
        }

        #[rstest]
        #[case("beta")]
        #[case("rc")]
        fn test_custom_label_returned(#[case] label: &str) {
            let args = FlowArgs {
                branch_config: BranchRulesConfig {
                    pre_release_label: Some(label.to_string()),
                    ..Default::default()
                },
                ..FlowArgs::default()
            };
            let expected = args.build_pre_release_bump_template(label);
            assert_eq!(args.bump_pre_release_label(), Some(Template::new(expected)));
        }
    }

    mod bump_pre_release_num {
        use super::*;

        #[test]
        fn test_default_returns_template() {
            let args = FlowArgs {
                branch_config: BranchRulesConfig {
                    pre_release_label: Some("alpha".to_string()), // Need a pre-release label for bump_pre_release_num to return template
                    ..Default::default()
                },
                ..FlowArgs::default()
            };
            let result = args.bump_pre_release_num();
            assert!(result.is_some());
            assert!(result.unwrap().is_some()); // Should be Some(Template)
        }

        #[rstest]
        #[case(5)]
        #[case(123)]
        #[case(999)]
        fn test_custom_num_returns_value(#[case] num: u32) {
            let args = FlowArgs {
                branch_config: BranchRulesConfig {
                    pre_release_label: Some("alpha".to_string()), // Need a pre-release label for bump_pre_release_num to return template
                    pre_release_num: Some(num),
                    ..Default::default()
                },
                ..FlowArgs::default()
            };
            let result = args.bump_pre_release_num();
            assert!(result.is_some());
            let template = result.unwrap().unwrap();

            // Generate expected template using the helper function
            let expected = args.build_pre_release_bump_template(&num.to_string());
            assert_eq!(template.as_str(), expected);
        }

        #[rstest]
        #[case(3)]
        #[case(7)]
        #[case(5)]
        fn test_template_uses_hash_branch_len(#[case] length: u32) {
            let args = FlowArgs {
                branch_config: BranchRulesConfig {
                    pre_release_label: Some("alpha".to_string()), // Need a pre-release label for bump_pre_release_num to return template
                    ..Default::default()
                },
                hash_branch_len: length,
                ..FlowArgs::default()
            };
            let result = args.bump_pre_release_num();
            assert!(result.is_some());
            let template = result.unwrap().unwrap();

            // Generate expected template from input
            let content = format!("{{{{ hash_int(value=bumped_branch, length={}) }}}}", length);
            let expected = args.build_pre_release_bump_template(&content);
            assert_eq!(template.as_str(), expected);
        }
    }

    mod bump_post {
        use super::*;

        #[rstest]
        #[case(post_modes::COMMIT, "{{ distance }}")]
        #[case(post_modes::TAG, "1")]
        fn test_bump_post_templates(#[case] mode: &str, #[case] expected_content: &str) {
            let args = FlowArgs {
                branch_config: BranchRulesConfig {
                    post_mode: Some(mode.to_string()),
                    pre_release_label: Some("alpha".to_string()), // Need a pre-release label
                    ..Default::default()
                },
                ..FlowArgs::default()
            };

            let result = args.bump_post();
            assert!(result.is_some());
            let template_result = result.unwrap();
            assert!(template_result.is_some());
            let template = template_result.unwrap();

            let expected = args.build_pre_release_bump_template(expected_content);
            assert_eq!(template.as_str(), expected);
        }

        #[test]
        fn test_bump_post_commit_mode_default() {
            let args = FlowArgs::default();
            let result = args.bump_post();
            assert!(result.is_some());

            let template_result = result.unwrap();
            assert!(template_result.is_some());
            let template = template_result.unwrap();
            let expected = args.build_pre_release_bump_template("{{ distance }}");
            assert_eq!(template.as_str(), expected);
        }

        #[test]
        #[should_panic(expected = "Invalid post_mode should have been caught by validation")]
        fn test_bump_post_invalid_mode_panics() {
            let args = FlowArgs {
                branch_config: BranchRulesConfig {
                    post_mode: Some("invalid".to_string()),
                    ..Default::default()
                },
                ..FlowArgs::default()
            };

            // This should panic because invalid post_mode should be caught by validation
            args.bump_post();
        }
    }

    mod bump_dev {
        use super::*;

        #[test]
        fn test_bump_dev_always_returns_template() {
            let args = FlowArgs::default();
            let result = args.bump_dev();
            assert!(result.is_some());
            let template_result = result.unwrap();
            assert!(template_result.is_some());
            let template = template_result.unwrap();

            assert_eq!(
                template.as_str(),
                "{% if dirty %}{{ bumped_timestamp }}{% else %}None{% endif %}"
            );
        }
    }
}
