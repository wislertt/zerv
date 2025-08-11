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
        let path = std::env::temp_dir().join(format!("zerv-test-{}", std::process::id()));
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

    #[test]
    fn test_dir_new() {
        let dir = TestDir::new().unwrap();
        assert!(dir.path().exists());
        assert!(dir.path().is_dir());
    }

    #[test]
    fn test_dir_create_file() {
        let dir = TestDir::new().unwrap();
        dir.create_file("test.txt", "content").unwrap();
        let file_path = dir.path().join("test.txt");
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(file_path).unwrap(), "content");
    }

    #[test]
    fn test_dir_create_file_with_subdirs() {
        let dir = TestDir::new().unwrap();
        dir.create_file("sub/dir/test.txt", "content").unwrap();
        let file_path = dir.path().join("sub/dir/test.txt");
        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(file_path).unwrap(), "content");
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
}
