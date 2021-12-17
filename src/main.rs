use clap::{App, Arg};
use regex::Regex;
use std::convert::TryInto;
use std::error::Error;

mod geomath;
use geomath::{get_distance_between_points, get_point_bearing_distance};

mod stations;
use stations::STATIONS;

#[derive(Debug)]
enum OperationalMode {
    Maintenance,
    CleanAir,
    Precipitation,
}

#[derive(Debug)]
struct Radial {
    azimuth: f32,
    elevation: f32,
    width: f32,
    precip_rates: Vec<f32>,
}

#[derive(Debug)]
struct PrecipRate {
    station_code: String,
    capture_time: chrono::NaiveDateTime,
    latitude: f32,
    longitude: f32,
    operational_mode: OperationalMode,
    precip_detected: bool,
    bin_size: f32,
    range_to_first_bin: f32,
    radials: Vec<Radial>,
}

type DataPoint = ([i64; 2], f32);
type GridData = Vec<Vec<DataPoint>>;

fn coord_as_i64(coord: f32) -> i64 {
    (coord * 10000.) as i64
}

impl PrecipRate {
    /// Given a desired height and width in pixels, convert the precip data in
    /// the existing radials to an [equirectangular][0] grid of points.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Equirectangular_projection
    fn sample_radials_to_equirectangular(&self, height: usize, width: usize) -> GridData {
        // first, convert every point from azimuth/bin to lat/lon
        let mut radials_equirectangular: Vec<DataPoint> = Vec::new();
        let mut coords: (f32, f32);
        for radial in self.radials.iter() {
            for (idx, bin) in radial.precip_rates.iter().enumerate() {
                coords = get_point_bearing_distance(
                    (self.latitude, self.longitude),
                    radial.azimuth,
                    self.bin_size * idx as f32 + 1. + self.range_to_first_bin,
                );
                radials_equirectangular
                    .push(([coord_as_i64(coords.0), coord_as_i64(coords.1)], *bin));
            }
        }
        // next, rearrange Vec<DataPoint> into a k-d tree for faster querying
        let radials_kdmap: kd_tree::KdMap<[i64; 2], f32> =
            kd_tree::KdMap::build(radials_equirectangular);
        // finally, sample the radial data into a grid
        let (mut current_lat, start_lon) =
            get_point_bearing_distance((self.latitude, self.longitude), 315., 325.2691);
        let mut coords;
        let mut samples: GridData = Vec::new();
        let mut current_sample: kd_tree::ItemAndDistance<DataPoint, i64>;
        for y in 0..height {
            // TODO: refactor get_point_bearing_distance such that the latitude and
            // longitude computations are separate; in these loops, we only need one
            // or the other at a time, so it would be more efficient to just compute
            // the one we need, instead of both every time
            samples.push(Vec::new());
            coords = (current_lat, start_lon);
            for x in 0..width {
                // we use current_lat instead of coords.0 here because get_point_bearing_distance
                // seems to have some latitude error even when bearing == 90 degrees
                // but since we know the latitude shouldn't change as we go east, we can just fix its value
                current_sample = radials_kdmap
                    .nearest(&[coord_as_i64(current_lat), coord_as_i64(coords.1)])
                    .unwrap();
                samples[y].push((
                    [coord_as_i64(current_lat), coord_as_i64(coords.1)],
                    match current_sample.squared_distance {
                        d if d < 100000 => current_sample.item.1,
                        _ => 0.0,
                    },
                ));
                coords = get_point_bearing_distance(
                    (current_lat, start_lon),
                    90.0,
                    460.0 / (width as f32) * (x as f32),
                );
            }
            current_lat = {
                let new_start_coords = get_point_bearing_distance(
                    (current_lat, start_lon),
                    180.0,
                    460.0 / (height as f32),
                );
                new_start_coords.0
            };
        }
        samples
    }
}

#[allow(clippy::ptr_arg)]
fn write_image(data: &GridData, filename: &str) {
    let mut img: image::RgbImage = image::ImageBuffer::new(data[0].len() as u32, data.len() as u32);
    for y in 0..data.len() {
        for x in 0..data[0].len() {
            img.put_pixel(
                x as u32,
                y as u32,
                // TODO: use piecewise scaling logic here
                image::Rgb([(255.0 * data[y][x].1.sqrt() / 3.) as u8; 3]),
            )
        }
    }
    img.save(filename).unwrap();
}

type ParseResult<T> = Result<(T, Vec<u8>), String>;

/// Pop `n` bytes off the front of `input` and return the two pieces
fn take_bytes(input: Vec<u8>, n: u16) -> ParseResult<Vec<u8>> {
    let x = input.split_at(n as usize);
    Ok((x.0.to_vec(), x.1.to_vec()))
}

/// Consume one byte from `input` and parse an `i8`
fn take_i8(input: Vec<u8>) -> ParseResult<i8> {
    let (number, tail) = take_bytes(input, 1)?;
    let buf: [u8; 1] = number.try_into().unwrap(); // TODO: handle error
    Ok((i8::from_be_bytes(buf), tail))
}

/// Consume two bytes from `input` and parse an `i16`
fn take_i16(input: Vec<u8>) -> ParseResult<i16> {
    let (number, tail) = take_bytes(input, 2)?;
    let buf: [u8; 2] = number.try_into().unwrap(); // TODO: handle error
    Ok((i16::from_be_bytes(buf), tail))
}

/// Consume four bytes from `input` and parse an `i32`
fn take_i32(input: Vec<u8>) -> ParseResult<i32> {
    let (number, tail) = take_bytes(input, 4)?;
    let buf: [u8; 4] = number.try_into().unwrap(); // TODO: handle error
    Ok((i32::from_be_bytes(buf), tail))
}

/// Consume four bytes from `input` and parse a `u32`
fn take_u32(input: Vec<u8>) -> ParseResult<u32> {
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
fn take_string(input: Vec<u8>) -> ParseResult<String> {
    let (length, tail) = take_u32(input)?;
    // grab the string
    let (string_bytes, tail) = take_bytes(tail, length as u16)?;
    let string = match String::from_utf8(string_bytes) {
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
fn take_float(input: Vec<u8>) -> ParseResult<f32> {
    let (number, tail) = take_bytes(input, 4)?;
    let buf: [u8; 4] = number.try_into().unwrap(); // TODO: handle error
    Ok((f32::from_be_bytes(buf), tail))
}

fn text_header(input: Vec<u8>) -> ParseResult<String> {
    let (_, tail) = take_bytes(input, 7)?;
    let (station_code, tail) = take_bytes(tail, 4)?;
    let (_, tail) = take_bytes(tail, 19)?;
    match String::from_utf8(station_code) {
        Ok(s) => Ok((s, tail)),
        Err(e) => Err(format!("Failed to parse station code: {}", e)),
    }
}

fn message_header(input: Vec<u8>) -> ParseResult<()> {
    let (_, tail) = take_bytes(input, 18)?;
    Ok(((), tail))
}

fn product_description(input: Vec<u8>) -> ParseResult<(f32, f32, OperationalMode, bool, i32)> {
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
fn radial(input: Vec<u8>) -> ParseResult<Radial> {
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

fn product_symbology(
    input: Vec<u8>,
    uncompressed_size: i32,
) -> ParseResult<(f32, f32, chrono::NaiveDateTime, Vec<Radial>)> {
    // decompress remaining input, which should all be compressed with bzip2
    let mut tmp = Vec::with_capacity(uncompressed_size as usize);
    let mut reader = bzip2_rs::DecoderReader::new(input.as_slice());
    match std::io::copy(&mut reader, &mut tmp) {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed to decompress symbology block: {}", e)),
    };

    // header (Figure 3-6, Sheet 7)
    let (_, tail) = take_bytes(tmp, 16)?;

    // another header (Figure 3-15c)
    let (_, tail) = take_bytes(tail, 8)?;

    // Product Description Data Structure header (Figure E-1)
    let (_, tail) = take_string(tail)?; // name
    let (_, tail) = take_string(tail)?; // description
    let (_, tail) = take_bytes(tail, 12)?;
    let (_, tail) = take_string(tail)?; // radar name
    let (_, tail) = take_bytes(tail, 12)?;
    let (capture_time, tail) = take_u32(tail)?;
    let (_, tail) = take_bytes(tail, 48)?;

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

    Ok((
        (
            range_to_first_bin / 1000.,
            bin_size / 1000.,
            chrono::NaiveDateTime::from_timestamp(capture_time as i64, 0),
            radials,
        ),
        tail,
    ))
}

fn parse_dpr(input: Vec<u8>) -> Result<PrecipRate, String> {
    let (station_code, tail) = text_header(input)?;
    let (_, tail) = message_header(tail)?;
    let ((latitude, longitude, operational_mode, precip_detected, uncompressed_size), tail) =
        product_description(tail)?;
    let ((range_to_first_bin, bin_size, capture_time, radials), _) =
        product_symbology(tail, uncompressed_size)?;
    Ok(PrecipRate {
        station_code,
        capture_time,
        latitude,
        longitude,
        operational_mode,
        precip_detected,
        bin_size,
        range_to_first_bin,
        radials,
    })
}

/// Given a station code (e.g. KGYX), try to download the latest radar data for
/// that station from the NWS. The data is on an NWS Web server [here][0]. The
/// station codes are the last four characters of the directory names. The
/// station directories contain data from the last day or so, and the most
/// recent data file is always called `sn.last`.
///
/// [0]: https://tgftp.nws.noaa.gov/SL.us008001/DF.of/DC.radar/DS.176pr/
fn get_data_by_station(station_code: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let resp = reqwest::blocking::get(format!(
        "https://tgftp.nws.noaa.gov/SL.us008001/DF.of/DC.radar/DS.176pr/SI.{}/sn.last",
        station_code
    ))?;
    match resp.status() {
        reqwest::StatusCode::OK => Ok(resp.bytes()?.to_vec()),
        status => {
            return Err(format!(
                "Failed to get data for station code '{}': server responded with {}",
                station_code, status
            )
            .into())
        }
    }
}

#[allow(clippy::ptr_arg)]
fn find_pixel_by_lat_long(
    pixels: &GridData,
    latitude: f32,
    longitude: f32,
) -> Result<(usize, usize), Box<dyn Error>> {
    // TODO: replace these linear searches with binary search
    // since the data dimensions should only be 256 by 256, this is probably
    // fine for now, or maybe forever

    // first, find the latitude
    let mut y = 0;
    let target_lat = coord_as_i64(latitude);
    while pixels[y][0].0[0] > target_lat {
        y += 1;
    }

    // then, find the longitude
    let mut x = 0;
    let target_lon = coord_as_i64(longitude);
    while pixels[y][x].0[1] < target_lon {
        x += 1;
    }

    Ok((y, x))
}

/// Queries the NWS radar station status server and returns a `Vec` containing
/// tuples of station codes and a boolean. The boolean indicates whether or not
/// the station is online and operating, according to the status server.
fn get_station_statuses() -> Result<Vec<(String, bool)>, Box<dyn Error>> {
    let resp = reqwest::blocking::get("https://radar3pub.ncep.noaa.gov/")?;
    let status_data = match resp.status() {
        reqwest::StatusCode::OK => resp.bytes()?.to_vec(),
        status => {
            return Err(format!(
                "Failed to get station statuses, server responded with: {}",
                status
            )
            .into())
        }
    };
    let re = Regex::new(r"(33FF33|FFFF00|0000FF|FF0000).*([A-Z]{4})").unwrap();
    Ok(re
        .captures_iter(std::str::from_utf8(&status_data).unwrap())
        .map(|s| (s[2].to_owned(), &s[1] == "33FF33"))
        .collect())
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("threecast")
        .version("0.1.0")
        .author("Bradley Gannon <bradley@bradleygannon.com>")
        .about("Like a forecast, but smaller")
        .arg(
            Arg::with_name("station")
                .short("s")
                .long("station")
                .value_name("STATION")
                .help("Four-letter station code, e.g. KGYX")
                .takes_value(true)
                .conflicts_with("file"),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("Path to a NEXRAD Level III Product 176 data file")
                .takes_value(true)
                .conflicts_with("station"),
        )
        .arg(
            Arg::with_name("latitude")
                .short("y")
                .long("latitude")
                .value_name("LATITUDE")
                .help("e.g. \"51.4275\"")
                .takes_value(true)
                .required(true)
                .allow_hyphen_values(true),
        )
        .arg(
            Arg::with_name("longitude")
                .short("x")
                .long("longitude")
                .value_name("LONGITUDE")
                .help("e.g. \"-87.7660\"")
                .takes_value(true)
                .required(true)
                .allow_hyphen_values(true),
        )
        .arg(Arg::with_name("verbose").short("v").long("verbose"))
        .get_matches();

    let latitude = matches.value_of("latitude").unwrap().parse::<f32>()?;
    let longitude = matches.value_of("longitude").unwrap().parse::<f32>()?;

    let input = if matches.is_present("station") {
        let station_code = matches.value_of("station").unwrap().to_lowercase();
        if !STATIONS.iter().any(|s| s.code == station_code) {
            return Err(format!("'{}' is not a valid station code", station_code).into());
        }
        let statuses = get_station_statuses()?;
        if !statuses.iter().find(|s| s.0 == station_code).unwrap().1 {
            return Err(format!("Station {} is offline", station_code).into());
        }
        get_data_by_station(&station_code)?
    } else if matches.is_present("file") {
        std::fs::read(matches.value_of("file").unwrap())?
    } else {
        let nearby_stations = match stations::find_nearest_stations(latitude, longitude) {
            Some(s) => s,
            None => {
                return Err(String::from(
                    "Given location is not within range of any radar stations",
                )
                .into())
            }
        };
        let station_statuses = get_station_statuses()?;
        let mut precip_data = None;
        for station in nearby_stations {
            if station_statuses.iter().find(|s| s.0 == station).unwrap().1 {
                precip_data = Some(get_data_by_station(&station.to_lowercase())?);
                break;
            }
        }
        if precip_data.is_none() {
            return Err(String::from(
                "All radar stations within range of this location are offline",
            )
            .into());
        }
        precip_data.unwrap()
    };

    let dpr = parse_dpr(input)?;
    let precip = dpr.sample_radials_to_equirectangular(256, 256);
    let coords = {
        if matches.is_present("file") {
            let distance_from_station =
                get_distance_between_points((latitude, longitude), (dpr.latitude, dpr.longitude));
            if distance_from_station > 230. {
                return Err(format!(
                    "Supplied file contains data for station {}, but supplied point is outside coverage area ({} km away)",
                    dpr.station_code,
                    distance_from_station.round()).into());
            }
        }
        find_pixel_by_lat_long(&precip, latitude, longitude)?
    };
    let precip_at_coords = precip[coords.0][coords.1].1;
    println!(
        "Current precipitation: {} in/hr ({})",
        precip_at_coords,
        match precip_at_coords {
            p if p == 0. => "none",
            p if p < 0.098 => "light",
            p if p < 0.35 => "moderate",
            p if p < 2. => "heavy",
            p if p >= 2. => "violent",
            _ => unreachable!(),
        }
    );

    Ok(())
}
