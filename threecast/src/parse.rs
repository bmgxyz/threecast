use chrono::{DateTime, Utc};

#[derive(Debug)]
pub enum OperationalMode {
    Maintenance,
    CleanAir,
    Precipitation,
}

#[derive(Debug)]
pub struct Radial {
    pub azimuth: f32,
    pub elevation: f32,
    pub width: f32,
    pub precip_rates: Vec<f32>,
}

#[derive(Debug)]
pub struct PrecipRate {
    pub station_code: String,
    pub capture_time: DateTime<Utc>,
    pub scan_number: i32,
    pub latitude: f32,
    pub longitude: f32,
    pub operational_mode: OperationalMode,
    pub precip_detected: bool,
    pub bin_size: f32,
    pub range_to_first_bin: f32,
    pub radials: Vec<Radial>,
}

type ParseResult<'a, T> = Result<(T, &'a [u8]), String>;

/// Pop `n` bytes off the front of `input` and return the two pieces
fn take_bytes(input: &[u8], n: u16) -> ParseResult<&[u8]> {
    let x = input.split_at(n as usize);
    Ok((x.0, x.1))
}

/// Consume one byte from `input` and parse an `i8`
fn take_i8(input: &[u8]) -> ParseResult<i8> {
    let (number, tail) = take_bytes(input, 1)?;
    let buf: [u8; 1] = number.try_into().unwrap(); // TODO: handle error
    Ok((i8::from_be_bytes(buf), tail))
}

/// Consume two bytes from `input` and parse an `i16`
fn take_i16(input: &[u8]) -> ParseResult<i16> {
    let (number, tail) = take_bytes(input, 2)?;
    let buf: [u8; 2] = number.try_into().unwrap(); // TODO: handle error
    Ok((i16::from_be_bytes(buf), tail))
}

/// Consume four bytes from `input` and parse an `i32`
fn take_i32(input: &[u8]) -> ParseResult<i32> {
    let (number, tail) = take_bytes(input, 4)?;
    let buf: [u8; 4] = number.try_into().unwrap(); // TODO: handle error
    Ok((i32::from_be_bytes(buf), tail))
}

/// Consume four bytes from `input` and parse a `u32`
fn take_u32(input: &[u8]) -> ParseResult<u32> {
    let (number, tail) = take_bytes(input, 4)?;
    let buf: [u8; 4] = number.try_into().unwrap(); // TODO: handle error
    Ok((u32::from_be_bytes(buf), tail))
}

/// Parse an XDR string from the head of the input
///
/// XDR strings are not null-terminated. Instead, they start with an unsigned
/// four-byte integer that contains the total string length. Then, the contents
/// of the string follow, padded with zero bytes to a multiple of four.
///
/// For more information, see [RFC 1832](https://datatracker.ietf.org/doc/html/rfc1832#section-3.11).
fn take_string(input: &[u8]) -> ParseResult<String> {
    let (length, tail) = take_u32(input)?;
    // grab the string
    let (string_bytes, tail) = take_bytes(tail, length as u16)?;
    let string = match String::from_utf8(string_bytes.to_vec()) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to parse string: {}", e)),
    };
    // pad out to the next four-byte boundary if needed
    if length % 4 != 0 {
        let (_, tail) = take_bytes(tail, 4 - (length % 4) as u16)?;
        Ok((string, tail))
    } else {
        Ok((string, tail))
    }
}

/// Consume four bytes from `input` and parse an `f32`
fn take_float(input: &[u8]) -> ParseResult<f32> {
    let (number, tail) = take_bytes(input, 4)?;
    let buf: [u8; 4] = number.try_into().unwrap(); // TODO: handle error
    Ok((f32::from_be_bytes(buf), tail))
}

fn text_header(input: &[u8]) -> ParseResult<String> {
    let (_, tail) = take_bytes(input, 7)?;
    let (station_code, tail) = take_bytes(tail, 4)?;
    let (_, tail) = take_bytes(tail, 19)?;
    match String::from_utf8(station_code.to_vec()) {
        Ok(s) => Ok((s, tail)),
        Err(e) => Err(format!("Failed to parse station code: {}", e)),
    }
}

fn message_header(input: &[u8]) -> ParseResult<()> {
    let (_, tail) = take_bytes(input, 18)?;
    Ok(((), tail))
}

fn product_description(input: &[u8]) -> ParseResult<(f32, f32, OperationalMode, bool, i32)> {
    let (_, tail) = take_bytes(input, 2)?;
    let (latitude_int, tail) = take_i32(tail)?;
    let (longitude_int, tail) = take_i32(tail)?;
    let (_, tail) = take_bytes(tail, 4)?;
    let (operational_mode_int, tail) = take_i16(tail)?;
    let (_, tail) = take_bytes(tail, 24)?;
    let (precip_detected_int, tail) = take_i8(tail)?;
    let (_, tail) = take_bytes(tail, 43)?;
    let (uncompressed_size, tail) = take_i32(tail)?;
    let (_, tail) = take_bytes(tail, 14)?;
    Ok((
        (
            latitude_int as f32 / 1000.0,
            longitude_int as f32 / 1000.0,
            match operational_mode_int {
                0 => OperationalMode::Maintenance,
                1 => OperationalMode::CleanAir,
                2 => OperationalMode::Precipitation,
                _ => OperationalMode::Maintenance, // TODO: throw error here
            },
            !matches!(precip_detected_int, 0),
            uncompressed_size,
        ),
        tail,
    ))
}

/// Parse Radial Information Data Structure (Figure E-4)
fn radial(input: &[u8]) -> ParseResult<Radial> {
    let (azimuth, tail) = take_float(input)?;
    let (elevation, tail) = take_float(tail)?;
    let (width, tail) = take_float(tail)?;
    let (num_bins, tail) = take_i32(tail)?;
    let (_attributes, tail) = take_string(tail)?;
    let (_, tail) = take_bytes(tail, 4)?;
    let mut precip_rates: Vec<f32> = Vec::with_capacity(num_bins as usize);
    let (precip_rate_bytes, tail) = take_bytes(tail, (num_bins * 4) as u16)?;
    for idx in 0..num_bins {
        let buf: [u8; 2] = precip_rate_bytes[(idx * 4 + 2) as usize..(idx * 4 + 4) as usize]
            .try_into()
            .unwrap();
        precip_rates.push(u16::from_be_bytes(buf) as f32 / 1000.0);
    }
    Ok((
        Radial {
            azimuth,
            elevation,
            width,
            precip_rates,
        },
        tail,
    ))
}

fn product_symbology(input: &[u8]) -> ParseResult<(f32, f32, i32, DateTime<Utc>, Vec<Radial>)> {
    // header (Figure 3-6, Sheet 7)
    let (_, tail) = take_bytes(input, 16)?;

    // another header (Figure 3-15c)
    let (_, tail) = take_bytes(tail, 8)?;

    // Product Description Data Structure header (Figure E-1)
    let (_, tail) = take_string(tail)?; // name
    let (_, tail) = take_string(tail)?; // description
    let (_, tail) = take_bytes(tail, 12)?;
    let (_, tail) = take_string(tail)?; // radar name
    let (_, tail) = take_bytes(tail, 12)?;
    let (capture_time, tail) = take_u32(tail)?;
    let (_, tail) = take_bytes(tail, 8)?;
    let (scan_number, tail) = take_i32(tail)?;
    let (_, tail) = take_bytes(tail, 36)?;

    // Radial Component Data Structure (Figure E-3)
    let (_, tail) = take_bytes(tail, 4)?;
    let (_, tail) = take_string(tail)?; // description
    let (bin_size, tail) = take_float(tail)?;
    let (range_to_first_bin, tail) = take_float(tail)?;
    let (_, tail) = take_bytes(tail, 8)?;
    let (num_radials, mut tail) = take_i32(tail)?;

    // parse the radials themselves
    let mut radials: Vec<Radial> = Vec::with_capacity(num_radials as usize);
    for _ in 0..num_radials {
        let tmp = radial(tail)?;
        radials.push(tmp.0);
        tail = tmp.1;
    }

    let timestamp = match DateTime::from_timestamp(capture_time as i64, 0) {
        Some(t) => t,
        None => return Err(format!("Failed to parse timestamp: {}", capture_time)),
    };

    Ok((
        (
            range_to_first_bin / 1000.,
            bin_size / 1000.,
            scan_number,
            timestamp,
            radials,
        ),
        tail,
    ))
}

pub fn parse_dpr(input: &[u8]) -> Result<PrecipRate, String> {
    let (station_code, tail) = text_header(input)?;
    let (_, tail) = message_header(tail)?;
    let ((latitude, longitude, operational_mode, precip_detected, uncompressed_size), tail) =
        product_description(tail)?;
    // decompress remaining input, which should all be compressed with bzip2
    let mut uncompressed_payload = Vec::with_capacity(uncompressed_size as usize);
    let mut reader = bzip2_rs::DecoderReader::new(tail);
    match std::io::copy(&mut reader, &mut uncompressed_payload) {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed to decompress symbology block: {}", e)),
    };
    let ((range_to_first_bin, bin_size, scan_number, capture_time, radials), _) =
        product_symbology(&uncompressed_payload)?;
    Ok(PrecipRate {
        station_code,
        capture_time,
        scan_number,
        latitude,
        longitude,
        operational_mode,
        precip_detected,
        bin_size,
        range_to_first_bin,
        radials,
    })
}
