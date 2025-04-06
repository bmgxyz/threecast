use std::ops::RangeInclusive;

use chrono::{DateTime, Utc};
use uom::si::{f32::Length, length::meter};

use crate::{
    DprError, ParseResult,
    radials::{Radial, radial},
    utils::*,
};

pub(crate) struct ProductSymbology {
    pub(crate) range_to_first_bin: Length,
    pub(crate) bin_size: Length,
    pub(crate) scan_number: u8,
    pub(crate) capture_time: DateTime<Utc>,
    pub(crate) radials: Vec<Radial>,
}

impl ProductSymbology {
    const NAME: &'static str = "product symbology";
    const SCAN_NUMBER_RANGE: RangeInclusive<i32> = 1..=80;
    const RADIAL_COMPONENT_TYPE_VALUE: i32 = 1;
    const BIN_SIZE_RANGE: RangeInclusive<f32> = (0.)..=1000.;
    // It seems like the specified range may be incorrect for actual DPR files
    // const RANGE_TO_FIRST_BIN_RANGE: RangeInclusive<f32> = (1000.)..=460000.;
    const NUM_RADIALS_RANGE: RangeInclusive<i32> = 0..=800;
}

pub(crate) fn product_symbology(input: &[u8]) -> ParseResult<ProductSymbology> {
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
    let (capture_time, tail) = take_u32(tail)?; // volume scan start time
    let (_, tail) = take_bytes(tail, 8)?;
    let (scan_number, tail) = take_i32(tail)?;
    check_range_inclusive(
        ProductSymbology::SCAN_NUMBER_RANGE,
        scan_number,
        "scan number",
        ProductSymbology::NAME,
    )?;
    let (_, tail) = take_bytes(tail, 24)?;
    let (number_of_components, tail) = take_i32(tail)?;
    if number_of_components != 1 {
        return Err(DprError::Unsupported(format!(
            "found number of components in product symbology not equal to 1 (got {}); DPR files containing multiple components are not supported",
            number_of_components
        )));
    }
    let (_, tail) = take_bytes(tail, (number_of_components * 8) as u16)?;

    // Radial Component Data Structure (Figure E-3)
    let (radial_component_type, tail) = take_i32(tail)?;
    check_value(
        ProductSymbology::RADIAL_COMPONENT_TYPE_VALUE,
        radial_component_type,
        "radial component type",
        ProductSymbology::NAME,
    )?;
    let (_, tail) = take_string(tail)?; // description
    let (bin_size, tail) = take_float(tail)?;
    check_range_inclusive(
        ProductSymbology::BIN_SIZE_RANGE,
        bin_size,
        "bin size",
        ProductSymbology::NAME,
    )?;
    let (range_to_first_bin, tail) = take_float(tail)?;
    let (_, tail) = take_bytes(tail, 8)?;
    let (num_radials, mut tail) = take_i32(tail)?;
    check_range_inclusive(
        ProductSymbology::NUM_RADIALS_RANGE,
        num_radials,
        "num radials",
        ProductSymbology::NAME,
    )?;

    // parse the radials themselves
    let mut radials: Vec<Radial> = Vec::with_capacity(num_radials as usize);
    for _ in 0..num_radials {
        let tmp = radial(tail)?;
        radials.push(tmp.0);
        tail = tmp.1;
    }

    let range_to_first_bin = Length::new::<meter>(range_to_first_bin);
    let bin_size = Length::new::<meter>(bin_size);
    let scan_number = scan_number as u8;
    let capture_time = match DateTime::from_timestamp(capture_time as i64, 0) {
        Some(t) => t,
        None => return Err(DprError::InvalidCaptureTime(capture_time)),
    };

    Ok((
        ProductSymbology {
            range_to_first_bin,
            bin_size,
            scan_number,
            capture_time,
            radials,
        },
        tail,
    ))
}
