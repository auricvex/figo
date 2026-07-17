use std::io;

/// The unified error type for all figo operations.
#[derive(Debug, thiserror::Error)]
pub enum FigoError {
    /// The requested chart dimensions are invalid (width too small, etc.)
    #[error("invalid dimensions: {0}")]
    InvalidDimensions(String),

    /// An unsupported or invalid character set was specified.
    #[error("invalid character set: {0}")]
    InvalidCharset(String),

    /// User-supplied input failed validation (unknown enum tag, etc.).
    /// Returned by JSON command handlers when a field cannot be
    /// mapped to a supported library variant.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// One or more required fields were not provided.
    #[error("missing required fields: {0}")]
    MissingFields(String),

    /// Text content cannot fit within the given constraints.
    #[error("text overflow: {0}")]
    TextOverflow(String),

    /// An I/O error occurred (file read/write, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// The clipboard operation failed.
    #[error("clipboard error: {0}")]
    Clipboard(String),

    /// Failed to parse JSON input.
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// A generic error for unexpected conditions.
    #[error("{0}")]
    General(String),
}

/// Convenience alias for `Result` with `FigoError`.
pub type Result<T> = std::result::Result<T, FigoError>;
