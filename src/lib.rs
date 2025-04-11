use std::{fmt::Display, io};

use chrono::{DateTime, Utc};
use geo::{Point, Polygon, polygon};
use product_description::{OperationalMode, ProductDescription};
use product_symbology::ProductSymbology;
use uom::si::{
    angle::radian,
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

pub use error::DiprError;
use product_description::product_description;
use product_symbology::product_symbology;
pub use radials::Radial;
use utils::*;

pub type ParseResult<'a, T> = Result<(T, &'a [u8]), DiprError>;

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
    pub max_precip_rate: Velocity,
    pub bin_size: Length,
    pub range_to_first_bin: Length,
    pub radials: Vec<Radial>,
}

unit! {
    system: uom::si;
    quantity: uom::si::velocity;

    @inch_per_hour: 0.09144; "in/hr", "inch per hour", "inches per hour";
}

fn destination(
    origin_rad: Point<f32>,
    origin_lat_sin: f32,
    origin_lat_cos: f32,
    bearing_rad: f32,
    meters: f32,
) -> Point<f32> {
    const EARTH_RADIUS_METERS: f32 = 6371008.8;

    let origin_lng = origin_rad.x();

    let rad = meters / EARTH_RADIUS_METERS;

    let lat =
        { origin_lat_sin * rad.cos() + origin_lat_cos * rad.sin() * bearing_rad.cos() }.asin();
    let y = bearing_rad.sin() * rad.sin() * origin_lat_cos;
    let x = rad.cos() - origin_lat_sin * lat.sin();
    let y_div_x = y / x;
    // approximate atan2 with the identity function for small values; accurate within 0.01%
    let lng = if y_div_x < 0.017322 {
        y_div_x
    } else {
        y.atan2(x)
    } + origin_lng;

    // normalize longitude
    let lng = if lng > 180. {
        lng - 180.
    } else if lng < -180. {
        lng + 180.
    } else {
        lng
    };

    Point::new(lng.to_degrees(), lat.to_degrees())
}

impl PrecipRate {
    pub fn into_bins_iter(
        self,
        skip_zeros: bool,
    ) -> impl Iterator<Item = (Polygon<f32>, Velocity)> {
        let PrecipRate {
            location,
            bin_size,
            range_to_first_bin,
            radials,
            ..
        } = self;
        let origin = location;
        radials.into_iter().flat_map(move |radial| {
            let Radial {
                azimuth,
                width,
                precip_rates,
                ..
            } = radial;
            let origin_rad = origin.to_radians();
            let origin_lat_sin = origin_rad.y().sin();
            let origin_lat_cos = origin_rad.y().cos();
            let center_azimuth = azimuth;
            let left_azimuth = center_azimuth - width / 2.;
            let right_azimuth = center_azimuth + width / 2.;
            precip_rates
                .into_iter()
                .enumerate()
                .flat_map(move |(bin_idx, precip_rate)| {
                    if skip_zeros && precip_rate.get::<inch_per_hour>() == 0. {
                        return None;
                    }

                    let distance_inner_meters = range_to_first_bin.get::<meter>()
                        + bin_size.get::<meter>() * (bin_idx as f32 - 0.5);
                    let distance_outer_meters = range_to_first_bin.get::<meter>()
                        + bin_size.get::<meter>() * (bin_idx as f32 + 0.5);

                    let center_inner = destination(
                        origin_rad,
                        origin_lat_sin,
                        origin_lat_cos,
                        center_azimuth.get::<radian>(),
                        distance_inner_meters,
                    );
                    let center_outer = destination(
                        origin_rad,
                        origin_lat_sin,
                        origin_lat_cos,
                        center_azimuth.get::<radian>(),
                        distance_outer_meters,
                    );

                    let left_inner = destination(
                        origin_rad,
                        origin_lat_sin,
                        origin_lat_cos,
                        left_azimuth.get::<radian>(),
                        distance_inner_meters,
                    );
                    let left_outer = destination(
                        origin_rad,
                        origin_lat_sin,
                        origin_lat_cos,
                        left_azimuth.get::<radian>(),
                        distance_outer_meters,
                    );

                    let right_inner = destination(
                        origin_rad,
                        origin_lat_sin,
                        origin_lat_cos,
                        right_azimuth.get::<radian>(),
                        distance_inner_meters,
                    );
                    let right_outer = destination(
                        origin_rad,
                        origin_lat_sin,
                        origin_lat_cos,
                        right_azimuth.get::<radian>(),
                        distance_outer_meters,
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
                    Some((bin_shape, precip_rate))
                })
        })
    }
}

impl Display for PrecipRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Station Code:        {}", self.station_code)?;
        writeln!(f, "Capture Time:        {}", self.capture_time)?;
        writeln!(f, "Operational Mode:    {}", self.operational_mode)?;
        writeln!(
            f,
            "Precip Detected:     {}",
            if self.precip_detected { "Yes" } else { "No" }
        )?;
        writeln!(f, "Scan Number:         {}", self.scan_number)?;
        writeln!(
            f,
            "Max Precip Rate:     {} in/hr",
            self.max_precip_rate.get::<inch_per_hour>()
        )?;
        writeln!(
            f,
            "Bin Size:            {: >3} m",
            self.bin_size.get::<meter>()
        )?;
        writeln!(f, "Number of Radials:  {: >4}", self.radials.len())?;
        write!(
            f,
            "Range to First Bin:  {: >3} m",
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

pub fn parse_dipr(input: &[u8]) -> Result<PrecipRate, DiprError> {
    let (station_code, tail) = text_header(input)?;
    let (_, tail) = message_header(tail)?;

    let (
        ProductDescription {
            location,
            operational_mode,
            precip_detected,
            max_precip_rate,
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
        max_precip_rate,
        bin_size,
        range_to_first_bin,
        radials,
    })
}
