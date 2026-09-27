use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unable to open the local application data directory.")]
    DataDirectoryUnavailable,
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Password hashing failed.")]
    PasswordHash,
    #[error("Sensitive field encryption failed.")]
    Crypto,
    #[error("Invalid username or password.")]
    InvalidCredentials,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
