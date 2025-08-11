use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Temporary directory utility for testing
pub struct TestDir {
    path: PathBuf,
}

impl TestDir {
    /// Create a new temporary directory
    pub fn new() -> io::Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("zerv-test-{}-{}", std::process::id(), id));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create a file with content
    pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) -> io::Result<()> {
        let file_path = self.path.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)
    }

    /// Create a directory
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        fs::create_dir_all(self.path.join(path))
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
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
        // Keep dir alive until end of test
        drop(dir);
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
        // Keep dir alive until end of test
        drop(dir);
    }

    #[test]
    fn test_dir_path() {
        let dir = TestDir::new().unwrap();
        let path = dir.path();
        assert!(path.exists());
        assert!(path.is_absolute());
        // Keep dir alive until end of test
        drop(dir);
    }
}
