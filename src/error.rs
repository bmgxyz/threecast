use std::{
    array::TryFromSliceError,
    error::Error,
    fmt::{Debug, Display},
    io,
    string::FromUtf8Error,
};

#[derive(Debug)]
pub enum DiprError {
    InvalidOperationalMode(i16),
    InvalidCaptureTime(u32),
    DecompressionFailed(io::Error),
    InvalidUtf8String(FromUtf8Error),
    InvalidByteSlice(TryFromSliceError),
    ValueOutOfRange(String),
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
