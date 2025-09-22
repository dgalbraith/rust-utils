use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustUtilsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("Invalid UID/GID range: {0}")]
    InvalidRange(String),

    #[error("Remapping failed: {0}")]
    RemapFailed(String),

    #[error("System error: {0}")]
    System(#[from] nix::errno::Errno),

    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

pub type Result<T> = std::result::Result<T, RustUtilsError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let error = RustUtilsError::DirectoryNotFound("/test/path".to_string());
        assert_eq!(error.to_string(), "Directory not found: /test/path");

        let error = RustUtilsError::InvalidRange("test range".to_string());
        assert_eq!(error.to_string(), "Invalid UID/GID range: test range");

        let error = RustUtilsError::RemapFailed("test operation".to_string());
        assert_eq!(error.to_string(), "Remapping failed: test operation");

        let error = RustUtilsError::Permission("test permission".to_string());
        assert_eq!(error.to_string(), "Permission denied: test permission");

        let error = RustUtilsError::InvalidArguments("test args".to_string());
        assert_eq!(error.to_string(), "Invalid arguments: test args");

        let error = RustUtilsError::OperationFailed("test op".to_string());
        assert_eq!(error.to_string(), "Operation failed: test op");
    }

    #[test]
    fn test_error_from_io() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let rust_error: RustUtilsError = io_error.into();

        match rust_error {
            RustUtilsError::Io(_) => (),
            _ => panic!("Expected IO error conversion"),
        }
    }

    #[test]
    fn test_result_type() {
        let success: Result<i32> = Ok(42);
        assert!(success.is_ok());
        if let Ok(value) = success {
            assert_eq!(value, 42);
        }

        let failure: Result<i32> = Err(RustUtilsError::InvalidRange("test".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_debug() {
        let error = RustUtilsError::DirectoryNotFound("/test".to_string());
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("DirectoryNotFound"));
        assert!(debug_str.contains("/test"));
    }
}
