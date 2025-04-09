use std::{fmt::Display, io};

use chrono::{DateTime, Utc};
use geo::{Destination, Haversine, Point, Polygon, polygon};
use product_description::{OperationalMode, ProductDescription};
use product_symbology::ProductSymbology;
use radials::Radial;
use uom::si::{
    angle::degree,
    f32::{Length, Velocity},
    length::meter,
};

#[macro_use]
extern crate uom;

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
    /// Longitude/latitude coordinates of the radar station in degrees
    ///
    /// Note that the coordinates are reversed from the perhaps more typical latitude/longitude.
    /// This is to match the underlying convention of the `geo` crate, which ensures that the first
    /// coordinate `x` maps to the horizontal value (longitude) and the second coordinate `y` maps
    /// to the vertical value (latitude).
    pub location: Point<f32>,
    pub operational_mode: OperationalMode,
    pub precip_detected: bool,
    pub bin_size: Length,
    pub range_to_first_bin: Length,
    pub radials: Vec<Radial>,
}

unit! {
    system: uom::si;
    quantity: uom::si::velocity;

    @inch_per_hour: 0.09144; "in/hr", "inch per hour", "inches per hour";
}

impl PrecipRate {
    pub fn to_polygons(self, skip_zeros: bool) -> Vec<(Polygon<f32>, Velocity)> {
        let PrecipRate {
            location,
            bin_size,
            range_to_first_bin,
            radials,
            ..
        } = self;
        let origin = location;
        let mut bins = vec![];
        for radial in radials {
            let Radial {
                azimuth,
                width,
                precip_rates,
                ..
            } = radial;
            for (bin_idx, precip_rate) in precip_rates.into_iter().enumerate() {
                if skip_zeros && precip_rate.get::<inch_per_hour>() == 0. {
                    continue;
                }
                let center_azimuth = azimuth;
                let center_inner = Haversine.destination(
                    origin,
                    center_azimuth.get::<degree>(),
                    range_to_first_bin.get::<meter>()
                        + bin_size.get::<meter>() * (bin_idx as f32 - 0.5),
                );
                let center_outer = Haversine.destination(
                    origin,
                    center_azimuth.get::<degree>(),
                    range_to_first_bin.get::<meter>()
                        + bin_size.get::<meter>() * (bin_idx as f32 + 0.5),
                );

                let left_azimuth = center_azimuth - width / 2.;
                let left_inner = Haversine.destination(
                    origin,
                    left_azimuth.get::<degree>(),
                    range_to_first_bin.get::<meter>()
                        + bin_size.get::<meter>() * (bin_idx as f32 - 0.5),
                );
                let left_outer = Haversine.destination(
                    origin,
                    left_azimuth.get::<degree>(),
                    range_to_first_bin.get::<meter>()
                        + bin_size.get::<meter>() * (bin_idx as f32 + 0.5),
                );

                let right_azimuth = center_azimuth + width / 2.;
                let right_inner = Haversine.destination(
                    origin,
                    right_azimuth.get::<degree>(),
                    range_to_first_bin.get::<meter>()
                        + bin_size.get::<meter>() * (bin_idx as f32 - 0.5),
                );
                let right_outer = Haversine.destination(
                    origin,
                    right_azimuth.get::<degree>(),
                    range_to_first_bin.get::<meter>()
                        + bin_size.get::<meter>() * (bin_idx as f32 + 0.5),
                );

                let bin_shape = if center_inner == right_inner || center_inner == left_inner {
                    polygon!(center_inner.into(), right_outer.into(), left_outer.into(),)
                } else {
                    polygon!(
                        center_inner.into(),
                        right_inner.into(),
                        right_outer.into(),
                        center_outer.into(),
                        left_outer.into(),
                        left_inner.into()
                    )
                };
                bins.push((bin_shape, precip_rate));
            }
        }
        bins
    }
}

impl Display for PrecipRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Station Code:            {}\n", self.station_code)?;
        write!(f, "Capture Time:            {}\n", self.capture_time)?;
        write!(f, "Operational Mode:        {}\n", self.operational_mode)?;
        write!(f, "Scan Number:             {: >2}\n", self.scan_number)?;
        write!(
            f,
            "Precipitation Detected:  {}\n",
            if self.precip_detected { "Yes" } else { "No" }
        )?;
        write!(
            f,
            "Bin Size:                {: >3} m\n",
            self.bin_size.get::<meter>()
        )?;
        write!(f, "Number of Radials:      {: >4}\n", self.radials.len())?;
        write!(
            f,
            "Range to First Bin:      {: >3} m",
            self.range_to_first_bin.get::<meter>()
        )
    }
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
