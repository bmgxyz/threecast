use regex::Regex;
use std::{error::Error, io::Read};

/// Get the complete listing of all data files available for a given station.
/// This is useful for answering the question "which files are the two most
/// recent available?"
pub fn get_data_file_listing(station_code: &str) -> Result<String, Box<dyn Error>> {
    let mut resp = reqwest::blocking::get(format!(
        "https://tgftp.nws.noaa.gov/SL.us008001/DF.of/DC.radar/DS.176pr/SI.{}/",
        station_code.to_lowercase()
    ))?;
    let mut file_listing = String::new();
    match resp.status() {
        reqwest::StatusCode::OK => resp.read_to_string(&mut file_listing),
        status => {
            return Err(format!(
                "Failed to get data file 'sn.last' for station code '{}': server responded with {}",
                station_code, status
            )
            .into())
        }
    }?;
    Ok(file_listing)
}

/// Given a station code (e.g. KGYX), try to download the specified radar data
/// for that station from the NWS. The data is on an NWS Web server [here][0].
/// The station codes are the last four characters of the directory names. The
/// station directories contain data from the last day or so, and the most
/// recent data file is always called `sn.last`.
/// 
/// `data_file_index` must be either `"last"` or between `"0000"` and `"0250"`,
/// inclusive.
///
/// [0]: https://tgftp.nws.noaa.gov/SL.us008001/DF.of/DC.radar/DS.176pr/
pub fn get_data_by_station(
    station_code: &str,
    data_file_index: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let resp = reqwest::blocking::get(format!(
        "https://tgftp.nws.noaa.gov/SL.us008001/DF.of/DC.radar/DS.176pr/SI.{}/sn.{}",
        station_code.to_lowercase(),
        data_file_index
    ))?;
    let sn_data = match resp.status() {
        reqwest::StatusCode::OK => resp.bytes()?.to_vec(),
        status => {
            return Err(format!(
                "Failed to get data file 'sn.{}' for station code '{}': server responded with {}",
                data_file_index, station_code, status
            )
            .into())
        }
    };
    Ok(sn_data)
}

/// Queries the NWS radar station status server and returns a `Vec` containing
/// tuples of station codes and a boolean. The boolean indicates whether or not
/// the station is online and operating, according to the status server.
pub fn get_station_statuses() -> Result<Vec<(String, bool)>, Box<dyn Error>> {
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
