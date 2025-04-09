use std::{fmt::Display, ops::RangeInclusive};

use geo::Point;

use crate::{DprError, ParseResult, utils::*};

#[derive(Debug)]
pub enum OperationalMode {
    Maintenance,
    CleanAir,
    Precipitation,
}

impl Display for OperationalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationalMode::CleanAir => write!(f, "Clean Air"),
            OperationalMode::Maintenance => write!(f, "Maintenance"),
            OperationalMode::Precipitation => write!(f, "Precipitation"),
        }
    }
}

impl TryFrom<i16> for OperationalMode {
    type Error = DprError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OperationalMode::Maintenance),
            1 => Ok(OperationalMode::CleanAir),
            2 => Ok(OperationalMode::Precipitation),
            v => Err(DprError::ValueOutOfRange(format!(
                "operational mode: got {v}, expected {:?}",
                ProductDescription::OPERATIONAL_MODE_RANGE
            ))),
        }
    }
}

pub(crate) struct ProductDescription {
    pub(crate) location: Point<f32>,
    pub(crate) operational_mode: OperationalMode,
    pub(crate) precip_detected: bool,
    pub(crate) uncompressed_size: u32,
}

impl ProductDescription {
    const NAME: &'static str = "product description block";
    const BLOCK_DIVIDER_VALUE: i16 = -1;
    const LATITUDE_RANGE: RangeInclusive<i32> = -90_000..=90_000;
    const LONGITUDE_RANGE: RangeInclusive<i32> = -180_000..=180_000;
    const OPERATIONAL_MODE_RANGE: RangeInclusive<i16> = 0..=2;
    const PRECIP_DETECTED_RANGE: RangeInclusive<i8> = 0..=1;
}

/// Parse Product Description
///
/// Figure 3-6: Graphic Product Message (Sheet 6) and Table V
pub(crate) fn product_description(input: &[u8]) -> ParseResult<ProductDescription> {
    let (block_divider, tail) = take_i16(input)?;
    check_value(
        ProductDescription::BLOCK_DIVIDER_VALUE,
        block_divider,
        "block divider",
        ProductDescription::NAME,
    )?;

    let (latitude_int, tail) = take_i32(tail)?;
    check_range_inclusive(
        ProductDescription::LATITUDE_RANGE,
        latitude_int,
        "latitude",
        ProductDescription::NAME,
    )?;

    let (longitude_int, tail) = take_i32(tail)?;
    check_range_inclusive(
        ProductDescription::LONGITUDE_RANGE,
        longitude_int,
        "longitude",
        ProductDescription::NAME,
    )?;

    let (_, tail) = take_bytes(tail, 4)?;

    let (operational_mode_int, tail) = take_i16(tail)?;
    check_range_inclusive(
        ProductDescription::OPERATIONAL_MODE_RANGE,
        operational_mode_int,
        "operational mode",
        ProductDescription::NAME,
    )?;

    let (_, tail) = take_bytes(tail, 24)?;

    let (precip_detected_int, tail) = take_i8(tail)?;
    check_range_inclusive(
        ProductDescription::PRECIP_DETECTED_RANGE,
        precip_detected_int,
        "precipitation detected",
        ProductDescription::NAME,
    )?;

    let (_, tail) = take_bytes(tail, 43)?;
    let (uncompressed_size, tail) = take_i32(tail)?;
    let (_, tail) = take_bytes(tail, 14)?;

    let location = Point::new(longitude_int as f32 / 1000., latitude_int as f32 / 1000.);
    let operational_mode = operational_mode_int.try_into()?;
    let precip_detected = precip_detected_int != 0;
    let uncompressed_size = uncompressed_size as u32;

    Ok((
        ProductDescription {
            location,
            operational_mode,
            precip_detected,
            uncompressed_size,
        },
        tail,
    ))
}
