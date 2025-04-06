use std::{
    fmt::{Debug, Display},
    ops::RangeInclusive,
};

use crate::{DprError, ParseResult};

/// Pop `n` bytes off the front of `input` and return the two pieces
pub(crate) fn take_bytes(input: &[u8], n: u16) -> ParseResult<&[u8]> {
    let x = input.split_at(n as usize);
    Ok((x.0, x.1))
}

/// Consume one byte from `input` and parse an `i8`
pub(crate) fn take_i8(input: &[u8]) -> ParseResult<i8> {
    let (number, tail) = take_bytes(input, 1)?;
    let buf: [u8; 1] = number.try_into()?;
    Ok((i8::from_be_bytes(buf), tail))
}

/// Consume two bytes from `input` and parse an `i16`
pub(crate) fn take_i16(input: &[u8]) -> ParseResult<i16> {
    let (number, tail) = take_bytes(input, 2)?;
    let buf: [u8; 2] = number.try_into()?;
    Ok((i16::from_be_bytes(buf), tail))
}

/// Consume four bytes from `input` and parse an `i32`
pub(crate) fn take_i32(input: &[u8]) -> ParseResult<i32> {
    let (number, tail) = take_bytes(input, 4)?;
    let buf: [u8; 4] = number.try_into()?;
    Ok((i32::from_be_bytes(buf), tail))
}

/// Consume four bytes from `input` and parse a `u32`
pub(crate) fn take_u32(input: &[u8]) -> ParseResult<u32> {
    let (number, tail) = take_bytes(input, 4)?;
    let buf: [u8; 4] = number.try_into()?;
    Ok((u32::from_be_bytes(buf), tail))
}

/// Parse an XDR string from the head of the input
///
/// XDR strings are not null-terminated. Instead, they start with an unsigned
/// four-byte integer that contains the total string length. Then, the contents
/// of the string follow, padded with zero bytes to a multiple of four.
///
/// For more information, see [RFC 1832](https://datatracker.ietf.org/doc/html/rfc1832#section-3.11).
pub(crate) fn take_string(input: &[u8]) -> ParseResult<String> {
    let (length, tail) = take_u32(input)?;
    // grab the string
    let (string_bytes, tail) = take_bytes(tail, length as u16)?;
    let string = String::from_utf8(string_bytes.to_vec())?;
    // pad out to the next four-byte boundary if needed
    if length % 4 != 0 {
        let (_, tail) = take_bytes(tail, (4 - (length % 4)) as u16)?;
        Ok((string, tail))
    } else {
        Ok((string, tail))
    }
}

/// Consume four bytes from `input` and parse an `f32`
pub(crate) fn take_float(input: &[u8]) -> ParseResult<f32> {
    let (number, tail) = take_bytes(input, 4)?;
    let buf: [u8; 4] = number.try_into()?;
    Ok((f32::from_be_bytes(buf), tail))
}

pub(crate) fn check_value<T: Display + PartialEq>(
    expected: T,
    actual: T,
    name: &str,
    func: &str,
) -> Result<(), DprError> {
    if expected != actual {
        Err(DprError::ValueOutOfRange(format!(
            "{name} in {func}: got {actual}, expected {expected}"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn check_range_inclusive<T: Debug + Display + PartialOrd>(
    expected: RangeInclusive<T>,
    actual: T,
    name: &str,
    func: &str,
) -> Result<(), DprError> {
    if !expected.contains(&actual) {
        Err(DprError::ValueOutOfRange(format!(
            "{name} in {func}: got {actual}, expected {expected:?}"
        )))
    } else {
        Ok(())
    }
}
