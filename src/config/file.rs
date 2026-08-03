use std::path::{
    Path,
    PathBuf,
};

use crate::error::ZervError;
use crate::utils::constants::config_files;

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZervFileConfig {
    pub source: Option<String>,
    pub input_format: Option<String>,
    pub directory: Option<String>,
    pub output_format: Option<String>,
    pub output_template: Option<String>,
    pub output_prefix: Option<String>,
    pub schema: Option<String>,
    pub schema_ron: Option<String>,

    pub version: Option<VersionSection>,

    pub flow: Option<FlowSection>,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionSection {
    pub schema: Option<String>,
    pub schema_ron: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowSection {
    pub schema: Option<String>,
    pub schema_ron: Option<String>,
    pub post_mode: Option<String>,
    pub pre_release_label: Option<String>,
    pub hash_branch_len: Option<u32>,
    /// RON payload, parsed via `BranchRules::from_str` at consume time.
    pub branch_rules: Option<String>,
}

impl ZervFileConfig {
    pub fn from_toml_str(contents: &str) -> Result<Self, ZervError> {
        toml::from_str(contents)
            .map_err(|e| ZervError::ConfigParseError(format!("Failed to parse zerv.toml: {e}")))
    }
}

fn find_in_dir(dir: &Path) -> Option<PathBuf> {
    let primary = dir.join(config_files::PRIMARY);
    let fallback = dir.join(config_files::FALLBACK);

    match (primary.exists(), fallback.exists()) {
        (true, true) => {
            tracing::warn!(
                "Both {} and {} found in {}; using {}. Remove {} to silence this warning.",
                config_files::PRIMARY,
                config_files::FALLBACK,
                dir.display(),
                config_files::PRIMARY,
                config_files::FALLBACK
            );
            Some(primary)
        }
        (true, false) => Some(primary),
        (false, true) => Some(fallback),
        (false, false) => None,
    }
}

/// Config is repo-scoped: read only at the repository root resolved from
/// `start_dir` (nearest VCS boundary). No subdir shadowing — a version must be
/// fn(commit, policy), not fn(commit, policy, cwd).
pub fn discover(start_dir: &Path) -> Result<Option<PathBuf>, ZervError> {
    let dir = start_dir.canonicalize()?;
    match crate::vcs::find_vcs_root(&dir) {
        Ok(root) => Ok(find_in_dir(&root)),
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::vcs_markers;

    #[test]
    fn parse_empty_returns_default() {
        let config =
            ZervFileConfig::from_toml_str("").expect("empty TOML should parse to defaults");
        assert_eq!(config, ZervFileConfig::default());
    }

    #[test]
    fn parse_shared_top_fields() {
        let toml = r#"
            source = "git"
            output_format = "semver"
            output_prefix = "v"
            schema = "standard-context"
        "#;
        let config = ZervFileConfig::from_toml_str(toml).expect("valid shared-top TOML");
        assert_eq!(config.source.as_deref(), Some("git"));
        assert_eq!(config.output_format.as_deref(), Some("semver"));
        assert_eq!(config.output_prefix.as_deref(), Some("v"));
        assert_eq!(config.schema.as_deref(), Some("standard-context"));
        assert!(config.flow.is_none());
        assert!(config.version.is_none());
    }

    #[test]
    fn parse_flow_section() {
        let toml = r#"
            [flow]
            post_mode = "tag"
            pre_release_label = "beta"
            hash_branch_len = 7
            branch_rules = "[(pattern: \"main\", pre_release_label: beta, post_mode: commit)]"
        "#;
        let config = ZervFileConfig::from_toml_str(toml).expect("valid flow TOML");
        let flow = config.flow.expect("flow section present");
        assert_eq!(flow.post_mode.as_deref(), Some("tag"));
        assert_eq!(flow.pre_release_label.as_deref(), Some("beta"));
        assert_eq!(flow.hash_branch_len, Some(7));
        assert!(flow.branch_rules.is_some());
    }

    #[test]
    fn parse_version_section_overrides_shared_schema() {
        let toml = r#"
            schema = "standard-context"

            [version]
            schema = "standard-no-context"
        "#;
        let config = ZervFileConfig::from_toml_str(toml).expect("valid version override TOML");
        assert_eq!(config.schema.as_deref(), Some("standard-context"));
        assert_eq!(
            config.version.unwrap().schema.as_deref(),
            Some("standard-no-context")
        );
    }

    #[test]
    fn parse_invalid_toml_errors() {
        let result = ZervFileConfig::from_toml_str("not = valid = toml");
        let err = result.expect_err("malformed TOML should error");
        assert!(matches!(err, ZervError::ConfigParseError(_)));
        assert!(err.to_string().contains("zerv.toml"));
    }

    #[test]
    fn parse_rejects_unknown_field() {
        let result = ZervFileConfig::from_toml_str("output_format_typo = \"semver\"");
        assert!(matches!(
            result.expect_err("unknown field should be rejected"),
            ZervError::ConfigParseError(_)
        ));
    }

    use std::fs;

    use tempfile::TempDir;

    /// Canonicalize so assertions survive the macOS `/var` → `/private/var` symlink.
    fn canon_root(tmp: &TempDir) -> PathBuf {
        tmp.path().canonicalize().expect("canonicalize temp root")
    }

    fn write_config(dir: &Path, name: &str) {
        fs::write(dir.join(name), "source = \"git\"\n").expect("write config fixture");
    }

    fn init_git_dir(dir: &Path) {
        fs::create_dir_all(dir.join(vcs_markers::GIT)).unwrap();
    }

    #[test]
    fn discover_finds_primary_at_repo_root() {
        let tmp = TempDir::new().unwrap();
        let root = canon_root(&tmp);
        init_git_dir(&root);
        write_config(&root, config_files::PRIMARY);

        let found = discover(&root).expect("discover should succeed");
        assert_eq!(found, Some(root.join(config_files::PRIMARY)));
    }

    #[test]
    fn discover_finds_fallback_when_no_primary() {
        let tmp = TempDir::new().unwrap();
        let root = canon_root(&tmp);
        init_git_dir(&root);
        write_config(&root, config_files::FALLBACK);

        let found = discover(&root).expect("discover should succeed");
        assert_eq!(found, Some(root.join(config_files::FALLBACK)));
    }

    #[test]
    fn discover_walks_up_to_repo_root() {
        let tmp = TempDir::new().unwrap();
        let root = canon_root(&tmp);
        init_git_dir(&root);
        write_config(&root, config_files::PRIMARY);
        let deep = root.join("a").join("b");
        fs::create_dir_all(&deep).unwrap();

        let found = discover(&deep).expect("discover should succeed");
        assert_eq!(found, Some(root.join(config_files::PRIMARY)));
    }

    #[test]
    fn discover_repo_root_wins_over_subdir_config() {
        let tmp = TempDir::new().unwrap();
        let root = canon_root(&tmp);
        init_git_dir(&root);
        write_config(&root, config_files::PRIMARY);
        let pkg = root.join("pkg");
        fs::create_dir_all(&pkg).unwrap();
        write_config(&pkg, config_files::FALLBACK);

        let found = discover(&pkg).expect("discover should succeed");
        assert_eq!(
            found,
            Some(root.join(config_files::PRIMARY)),
            "subdir config must not shadow the repo-root config"
        );
    }

    #[test]
    fn discover_tie_returns_primary() {
        let tmp = TempDir::new().unwrap();
        let root = canon_root(&tmp);
        init_git_dir(&root);
        write_config(&root, config_files::PRIMARY);
        write_config(&root, config_files::FALLBACK);

        let found = discover(&root).expect("discover should succeed");
        assert_eq!(found, Some(root.join(config_files::PRIMARY)));
    }

    #[test]
    fn discover_returns_none_when_no_config_at_repo_root() {
        let tmp = TempDir::new().unwrap();
        let root = canon_root(&tmp);
        init_git_dir(&root);
        let sub = root.join("empty");
        fs::create_dir_all(&sub).unwrap();

        let found = discover(&sub).expect("discover should succeed");
        assert_eq!(found, None, "repo with no config should discover none");
    }

    #[test]
    fn discover_errors_on_missing_start_dir() {
        let result = discover(Path::new("/nonexistent/zerv/start/dir"));
        let err = result.expect_err("missing start dir should error");
        assert!(matches!(err, ZervError::Io(_)));
    }

    #[test]
    fn discover_ignores_config_above_git_boundary() {
        let tmp = TempDir::new().unwrap();
        let root = canon_root(&tmp);
        write_config(&root, config_files::PRIMARY);
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("sub")).unwrap();
        init_git_dir(&repo);

        let found = discover(&repo.join("sub")).expect("discover should succeed");
        assert_eq!(
            found, None,
            "config above the .git boundary must not be discovered"
        );
    }

    #[test]
    fn discover_finds_config_at_repo_root_from_subdir() {
        let tmp = TempDir::new().unwrap();
        let root = canon_root(&tmp);
        let repo = root.join("repo");
        fs::create_dir_all(&repo).unwrap();
        init_git_dir(&repo);
        write_config(&repo, config_files::PRIMARY);
        let deep = repo.join("a").join("b");
        fs::create_dir_all(&deep).unwrap();

        let found = discover(&deep).expect("discover should succeed");
        assert_eq!(found, Some(repo.join(config_files::PRIMARY)));
    }

    #[test]
    fn discover_gitfile_boundary_treated_same_as_dir() {
        let tmp = TempDir::new().unwrap();
        let root = canon_root(&tmp);
        write_config(&root, config_files::PRIMARY);
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("sub")).unwrap();
        fs::write(
            repo.join(vcs_markers::GIT),
            "gitdir: ../../.git/modules/repo\n",
        )
        .unwrap();

        let found = discover(&repo.join("sub")).expect("discover should succeed");
        assert_eq!(
            found, None,
            "a .git gitfile must bound discovery like a .git directory"
        );
    }
}
