//!
use std::fmt;

#[derive(Debug, Clone, Copy)]
///
pub struct StorageError {
    ///
    pub message: &'static str,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StorageError {{ message: {} }}", self.message)
    }
}

///
pub type StorageResult<T> = Result<T, StorageError>;
