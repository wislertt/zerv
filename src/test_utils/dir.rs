use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Temporary directory utility for testing
pub struct TestDir {
    inner: TempDir,
}

impl TestDir {
    /// Create a new temporary directory
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            inner: TempDir::new()?,
        })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        self.inner.path()
    }

    /// Create a file with content
    pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) -> io::Result<()> {
        let file_path = self.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)
    }

    /// Create a directory
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        fs::create_dir_all(self.path().join(path))
    }

    /// Initialize a git repository
    pub fn init_git(&self) -> io::Result<()> {
        let output = Command::new("git")
            .arg("init")
            .current_dir(self.path())
            .output()?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "git init failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Create git repository with initial files for testing
    pub fn create_dummy_git_files(&self) -> io::Result<()> {
        self.init_git()?;
        self.create_file("README.md", "# Test Repository")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_dir_new() {
        let dir = TestDir::new().unwrap();
        assert!(dir.path().exists());
        assert!(dir.path().is_dir());
    }

    #[rstest]
    #[case("test.txt", "content")]
    #[case("sub/dir/test.txt", "content")]
    #[case("deep/nested/path/file.txt", "deep content")]
    fn test_dir_create_file_variations(#[case] path: &str, #[case] content: &str) {
        let dir = TestDir::new().unwrap();
        dir.create_file(path, content).unwrap();
        let file_path = dir.path().join(path);
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), content);
    }

    #[test]
    fn test_dir_create_dir() {
        let dir = TestDir::new().unwrap();
        dir.create_dir("subdir").unwrap();
        let sub_path = dir.path().join("subdir");
        assert!(sub_path.exists());
        assert!(sub_path.is_dir());
    }

    #[test]
    fn test_dir_path() {
        let dir = TestDir::new().unwrap();
        let path = dir.path();
        assert!(path.exists());
        assert!(path.is_absolute());
    }

    #[test]
    fn test_git_init() {
        let dir = TestDir::new().unwrap();
        dir.init_git().unwrap();
        assert!(dir.path().join(".git").exists());
        assert!(dir.path().join(".git/HEAD").exists());
    }

    #[test]
    fn test_git_files() {
        let dir = TestDir::new().unwrap();
        dir.create_dummy_git_files().unwrap();
        assert!(dir.path().join(".git").exists());
        assert!(dir.path().join("README.md").exists());
    }
}
