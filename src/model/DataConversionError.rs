use std::fmt;

/// Custom error type for failed conversions
#[derive(Debug)]
pub enum ConversionError {
    MissingField(String),
    InvalidEngine,
    InvalidGearbox,
    ParseError(String),
    ErrorMessage(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ConversionError::InvalidEngine => write!(f, "Invalid engine type"),
            ConversionError::InvalidGearbox => write!(f, "Invalid gearbox type"),
            ConversionError::ParseError(field) => write!(f, "Failed to parse field: {}", field),
            ConversionError::ErrorMessage(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ConversionError {}
