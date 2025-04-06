use std::io;

use chrono::{DateTime, Utc};
use geo::Point;
use product_description::{OperationalMode, ProductDescription};
use product_symbology::ProductSymbology;
use radials::Radial;
use uom::si::f32::Length;

mod error;
mod product_description;
mod product_symbology;
mod radials;
mod utils;

pub use error::DprError;
use product_description::product_description;
use product_symbology::product_symbology;
use utils::*;

pub type ParseResult<'a, T> = Result<(T, &'a [u8]), DprError>;

#[derive(Debug)]
pub struct PrecipRate {
    pub station_code: String,
    pub capture_time: DateTime<Utc>,
    pub scan_number: u8,
    pub location: Point<f32>,
    pub operational_mode: OperationalMode,
    pub precip_detected: bool,
    pub bin_size: Length,
    pub range_to_first_bin: Length,
    pub radials: Vec<Radial>,
}

fn text_header(input: &[u8]) -> ParseResult<String> {
    let (_, tail) = take_bytes(input, 7)?;
    let (station_code, tail) = take_bytes(tail, 4)?;
    let (_, tail) = take_bytes(tail, 19)?;
    match String::from_utf8(station_code.to_vec()) {
        Ok(s) => Ok((s, tail)),
        Err(e) => Err(e.into()),
    }
}

fn message_header(input: &[u8]) -> ParseResult<()> {
    let (_, tail) = take_bytes(input, 18)?;
    Ok(((), tail))
}

pub fn parse_dpr(input: &[u8]) -> Result<PrecipRate, DprError> {
    let (station_code, tail) = text_header(input)?;
    let (_, tail) = message_header(tail)?;

    let (
        ProductDescription {
            location,
            operational_mode,
            precip_detected,
            uncompressed_size,
        },
        tail,
    ) = product_description(tail)?;

    // decompress remaining input, which should all be compressed with bzip2
    let mut uncompressed_payload = Vec::with_capacity(uncompressed_size as usize);
    let mut reader = bzip2_rs::DecoderReader::new(tail);
    io::copy(&mut reader, &mut uncompressed_payload)?;

    let (
        ProductSymbology {
            range_to_first_bin,
            bin_size,
            scan_number,
            capture_time,
            radials,
        },
        _,
    ) = product_symbology(&uncompressed_payload)?;

    Ok(PrecipRate {
        station_code,
        capture_time,
        scan_number,
        location,
        operational_mode,
        precip_detected,
        bin_size,
        range_to_first_bin,
        radials,
    })
}
