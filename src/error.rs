use std::{
    array::TryFromSliceError,
    error::Error,
    fmt::{Debug, Display},
    io,
    string::FromUtf8Error,
};

#[derive(Debug)]
/// Indicates a product-specific parsing error or wraps a lower-level error
pub enum DiprError {
    /// Found an invalid value for the station's operational mode
    InvalidOperationalMode(i16),
    /// Found an invalid value for the scan start time
    ///
    /// Since this value is defined as a Unix timestamp, this error variant should be unreachable.
    InvalidCaptureTime(u32),
    /// Failed to decompress the symbology block using [`bzip2_rs`]
    DecompressionFailed(io::Error),
    /// Failed to convert a byte slice to a [`String`] due to invalid UTF-8
    InvalidUtf8String(FromUtf8Error),
    /// Failed to convert a byte slice to a fixed-length array
    InvalidByteSlice(TryFromSliceError),
    /// Parsed value was outside its acceptable range defined in the specification
    ValueOutOfRange(String),
    /// Encountered a DIPR file variant that this crate doesn't support
    Unsupported(String),
}

impl Display for DiprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiprError::InvalidOperationalMode(o) => write!(
                f,
                "Failed to parse operational mode: expected 0, 1, or 2, but got {}",
                o
            ),
            DiprError::InvalidCaptureTime(t) => {
                write!(f, "Failed to parse capture time: 0x{:02x}", t)
            }
            DiprError::DecompressionFailed(d) => {
                write!(f, "Failed to decompress product symbology: {}", d)
            }
            DiprError::InvalidUtf8String(u) => write!(f, "Failed to parse UTF-8 string: {}", u),
            DiprError::InvalidByteSlice(s) => write!(f, "Failed to parse byte slice: {}", s),
            DiprError::ValueOutOfRange(s) => write!(f, "Value out of specified range: {}", s),
            DiprError::Unsupported(s) => write!(f, "{}", s),
        }
    }
}

impl Error for DiprError {}

impl From<TryFromSliceError> for DiprError {
    fn from(value: TryFromSliceError) -> Self {
        DiprError::InvalidByteSlice(value)
    }
}

impl From<FromUtf8Error> for DiprError {
    fn from(value: FromUtf8Error) -> Self {
        DiprError::InvalidUtf8String(value)
    }
}

impl From<io::Error> for DiprError {
    fn from(value: io::Error) -> Self {
        DiprError::DecompressionFailed(value)
    }
}
