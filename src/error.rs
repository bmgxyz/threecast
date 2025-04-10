use std::{
    array::TryFromSliceError,
    error::Error,
    fmt::{Debug, Display},
    io,
    string::FromUtf8Error,
};

#[derive(Debug)]
pub enum DprError {
    InvalidOperationalMode(i16),
    InvalidCaptureTime(u32),
    DecompressionFailed(io::Error),
    InvalidUtf8String(FromUtf8Error),
    InvalidByteSlice(TryFromSliceError),
    ValueOutOfRange(String),
    Unsupported(String),
}

impl Display for DprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DprError::InvalidOperationalMode(o) => write!(
                f,
                "Failed to parse operational mode: expected 0, 1, or 2, but got {}",
                o
            ),
            DprError::InvalidCaptureTime(t) => {
                write!(f, "Failed to parse capture time: 0x{:02x}", t)
            }
            DprError::DecompressionFailed(d) => {
                write!(f, "Failed to decompress product symbology: {}", d)
            }
            DprError::InvalidUtf8String(u) => write!(f, "Failed to parse UTF-8 string: {}", u),
            DprError::InvalidByteSlice(s) => write!(f, "Failed to parse byte slice: {}", s),
            DprError::ValueOutOfRange(s) => write!(f, "Value out of specified range: {}", s),
            DprError::Unsupported(s) => write!(f, "{}", s),
        }
    }
}

impl Error for DprError {}

impl From<TryFromSliceError> for DprError {
    fn from(value: TryFromSliceError) -> Self {
        DprError::InvalidByteSlice(value)
    }
}

impl From<FromUtf8Error> for DprError {
    fn from(value: FromUtf8Error) -> Self {
        DprError::InvalidUtf8String(value)
    }
}

impl From<io::Error> for DprError {
    fn from(value: io::Error) -> Self {
        DprError::DecompressionFailed(value)
    }
}
