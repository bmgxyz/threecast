//! Convert the National Weather Service's (NWS) Digital Instantaneous Precipitation Rate (DIPR)
//! radar product from its native data format into more common vector GIS formats
//!
//! The DIPR radar product is useful for observing and predicting precipitation on small time and
//! distance scales (less than 10 km or 60 minutes). This forecasting niche is called nowcasting.
//!
//! NWS defines the DIPR format in [this specification document][spec].
//!
//! [spec]: https://www.roc.noaa.gov/public-documents/icds/2620001T.pdf

use std::{fmt::Display, io};

use chrono::{DateTime, Utc};
use geo::{CoordsIter, Point as GeoPoint, Polygon as GeoPolygon, polygon};
use geojson::{Feature, JsonObject, JsonValue};
use product_description::{OperationalMode, ProductDescription};
use product_symbology::ProductSymbology;
use shapefile::{
    Point as ShapefilePoint, Polygon as ShapefilePolygon, PolygonRing,
    dbase::{self, FieldValue},
    record::polygon::GenericPolygon,
};
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

/// Convenient wrapper around [`Result`]
///
/// The `Ok` case returns some `T` that the returning function intends to parse, along with the
/// remaining input which has advanced past the parsed value. This is similar to how [`nom`][nom]
/// works, but less fancy.
///
/// [nom]: https://docs.rs/nom/latest/nom/
pub type ParseResult<'a, T> = Result<(T, &'a [u8]), DiprError>;

#[derive(Debug)]
/// Semantically useful representation of a DIPR product file
///
/// Create this struct with [parse_dipr].
pub struct PrecipRate {
    /// Radar station where this file was generated
    ///
    /// Station codes are usually four letters long. A list is available [here][station codes].
    ///
    /// [station codes]: https://www.weather.gov/media/tg/wsr88d-radar-list.pdf
    pub station_code: String,
    /// Moment when the scan in this file began
    pub capture_time: DateTime<Utc>,
    /// Incrementing counter to disambiguate scans
    ///
    /// This value is always between 1 and 80 inclusive.
    pub scan_number: u8,
    /// Longitude/latitude coordinates of the radar station in degrees
    ///
    /// Note that the coordinates are reversed from the perhaps more typical latitude/longitude.
    /// This is to match the underlying convention of the `geo` crate, which ensures that the first
    /// coordinate `x` maps to the "horizontal" value (longitude) and the second coordinate `y` maps
    /// to the "vertical" value (latitude).
    pub location: GeoPoint<f32>,
    /// Condition of the radar station
    pub operational_mode: OperationalMode,
    /// Whether the radar station measured any precipitation anywhere in its coverage area
    pub precip_detected: bool,
    /// Highest precipitation rate found in this file
    pub max_precip_rate: Velocity,
    /// Distance between the inner and outer extents of each bin measured radially
    pub bin_size: Length,
    /// Distance between the radar station and the center of the nearest bin
    pub range_to_first_bin: Length,
    /// All precipitation data contained in this DIPR file organized by azimuth
    pub radials: Vec<Radial>,
}

unit! {
    system: uom::si;
    quantity: uom::si::velocity;

    @inch_per_hour: 0.09144; "in/hr", "inch per hour", "inches per hour";
}

fn destination(
    origin_rad: GeoPoint<f32>,
    origin_lat_sin: f32,
    origin_lat_cos: f32,
    bearing_rad: f32,
    meters: f32,
) -> GeoPoint<f32> {
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

    GeoPoint::new(lng.to_degrees(), lat.to_degrees())
}

impl PrecipRate {
    /// Iterate over all bins, giving each of their boundaries and precipitation rates in a tuple
    ///
    /// Note that while the bins are officially bounded by circle sectors, this function
    /// approximates the bin shapes with polygons composed of line segments. Order is not guaranteed
    /// but is likely to be identical to the input file. That is, bins within each radial are given
    /// in increasing order of distance from the radar station, and radials are given in increasing
    /// order of azimuth angle.
    pub fn into_bins_iter(
        self,
        skip_zeros: bool,
    ) -> impl Iterator<Item = (GeoPolygon<f32>, Velocity)> {
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
    /// Iterate over all precipitation bins as in [`PrecipRate::into_bins_iter`], but also convert
    /// the results into values that are useful with the [`shapefile`] crate
    pub fn into_shapefile_iter(
        self,
        skip_zeros: bool,
    ) -> impl Iterator<Item = (GenericPolygon<ShapefilePoint>, FieldValue)> {
        self.into_bins_iter(skip_zeros)
            .map(|(polygon, precip_rate)| {
                (
                    ShapefilePolygon::new(PolygonRing::Outer(
                        polygon
                            .coords_iter()
                            .map(|c| ShapefilePoint::new(c.x.into(), c.y.into()))
                            .collect::<Vec<ShapefilePoint>>(),
                    )),
                    dbase::FieldValue::Float(Some(precip_rate.get::<inch_per_hour>())),
                )
            })
    }
    /// Iterate over all precipitation bins as in [`PrecipRate::into_bins_iter`], but also convert
    /// the results into values that are useful with the [`geojson`] crate
    pub fn into_geojson_iter(self, skip_zeros: bool) -> impl Iterator<Item = Feature> {
        self.into_bins_iter(skip_zeros)
            .map(|(polygon, precip_rate)| {
                let mut properties = JsonObject::new();
                properties.insert(
                    "precipRate".to_string(),
                    JsonValue::from(precip_rate.get::<inch_per_hour>()),
                );
                Feature {
                    geometry: Some((&polygon).into()),
                    properties: Some(properties),
                    ..Default::default()
                }
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
            "Max Precip Rate:     {:.3} in/hr",
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

/// Convert a byte slice into a [`PrecipRate`] or return an error
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
