use std::fs::Metadata;
use std::path::Path;

use crate::error::{Result, RustUtilsError};

pub fn get_file_metadata(path: &Path) -> Result<Metadata> {
    std::fs::symlink_metadata(path).map_err(|e| RustUtilsError::Io(e))
}

pub fn should_exclude(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    let path_str = path.to_string_lossy();

    for pattern in patterns {
        if matches_pattern(&path_str, pattern) {
            return true;
        }
    }

    false
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    // Simple glob-like pattern matching
    // This is a basic implementation - for production use, consider using the `glob` crate

    // Empty pattern should not match anything
    if pattern.is_empty() {
        return false;
    }

    if pattern.contains('*') {
        // Handle simple wildcard patterns
        let parts: Vec<&str> = pattern.split('*').collect();

        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];

            if prefix.is_empty() {
                return path.ends_with(suffix);
            }
            if suffix.is_empty() {
                return path.starts_with(prefix);
            }
            return path.starts_with(prefix) && path.ends_with(suffix);
        }

        // More complex patterns would need a proper glob implementation
        // For now, fall back to exact match
        return path == pattern;
    }

    // Exact match or substring match for directory patterns
    path == pattern || path.contains(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("file.log", "*.log"));
        assert!(matches_pattern("test.txt", "test.*"));
        assert!(matches_pattern("var/log/test.log", "var/log/*"));
        assert!(!matches_pattern("file.txt", "*.log"));
        assert!(matches_pattern("exact/match", "exact/match"));
    }

    #[test]
    fn test_matches_pattern_edge_cases() {
        // Empty pattern
        assert!(!matches_pattern("file.txt", ""));

        // Pattern with only asterisk
        assert!(matches_pattern("anything", "*"));

        // Multiple asterisks (fallback to exact match)
        assert!(!matches_pattern("a.b.c", "a*b*c"));

        // Substring matching
        assert!(matches_pattern("path/to/file", "path/to"));
        assert!(matches_pattern("long/path/name", "path"));

        // Case sensitivity
        assert!(!matches_pattern("File.LOG", "*.log"));
        assert!(matches_pattern("file.log", "*.log"));
    }

    #[test]
    fn test_should_exclude() {
        let patterns = vec!["*.log".to_string(), "tmp/*".to_string()];

        assert!(should_exclude(Path::new("test.log"), &patterns));
        assert!(should_exclude(Path::new("tmp/file.txt"), &patterns));
        assert!(!should_exclude(Path::new("test.txt"), &patterns));
        assert!(!should_exclude(Path::new("src/main.rs"), &patterns));
    }

    #[test]
    fn test_should_exclude_empty_patterns() {
        let patterns: Vec<String> = vec![];
        assert!(!should_exclude(Path::new("any/file"), &patterns));
    }

    #[test]
    fn test_should_exclude_multiple_patterns() {
        let patterns = vec![
            "*.log".to_string(),
            "tmp/*".to_string(),
            "*.sock".to_string(),
            "var/cache/*".to_string(),
        ];

        assert!(should_exclude(Path::new("app.log"), &patterns));
        assert!(should_exclude(Path::new("tmp/temp.txt"), &patterns));
        assert!(should_exclude(Path::new("server.sock"), &patterns));
        assert!(should_exclude(Path::new("var/cache/data"), &patterns));
        assert!(!should_exclude(Path::new("src/main.rs"), &patterns));
    }

    #[test]
    fn test_get_file_metadata() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");

        // Create a test file
        File::create(&file_path)?;

        // Test getting metadata
        let metadata = get_file_metadata(&file_path)?;
        assert!(metadata.is_file());

        // Test with directory
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path)?;

        let dir_metadata = get_file_metadata(&dir_path)?;
        assert!(dir_metadata.is_dir());

        Ok(())
    }

    #[test]
    fn test_get_file_metadata_nonexistent() {
        let result = get_file_metadata(Path::new("/nonexistent/file"));
        assert!(result.is_err());
    }
}
