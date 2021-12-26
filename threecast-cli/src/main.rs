use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use threecast::predict::predict_two;

use threecast::geomath::get_distance_between_points;
use threecast::net::{get_data_by_station, get_data_file_listing, get_station_statuses};
use threecast::parse::parse_dpr;
use threecast::stations::{find_nearest_stations, STATIONS};
use threecast::util::find_pixel_by_lat_long;

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
                .conflicts_with("station")
                .number_of_values(2),
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

    let latitude = match matches.value_of("latitude").unwrap().parse::<f32>() {
        Ok(lat) => lat,
        Err(_) => return Err("Failed to parse latitude".into()),
    };
    let longitude = match matches.value_of("longitude").unwrap().parse::<f32>() {
        Ok(lon) => lon,
        Err(_) => return Err("Failed to parse longitude".into()),
    };

    if latitude >= 90. || latitude <= -90. {
        return Err(format!("Latitude must be between -90 and 90 (got {})", latitude).into());
    }
    if longitude >= 180. || longitude <= -180. {
        return Err(format!("Longitude must be between -180 and 180 (got {})", longitude).into());
    }

    let input = if matches.is_present("file") {
        let files: Vec<&str> = matches.values_of("file").unwrap().collect();
        (std::fs::read(files[0])?, std::fs::read(files[1])?)
    } else {
        let station_code = if matches.is_present("station") {
            let station_code = matches.value_of("station").unwrap().to_lowercase();
            if !STATIONS.iter().any(|s| s.code == station_code) {
                return Err(format!("'{}' is not a valid station code", station_code).into());
            }
            let statuses = get_station_statuses()?;
            if !statuses.iter().find(|s| s.0 == station_code).unwrap().1 {
                return Err(format!("Station {} is offline", station_code).into());
            }
            station_code
        } else {
            let nearby_stations = match find_nearest_stations(latitude, longitude) {
                Some(s) => s,
                None => {
                    return Err(String::from(
                        "Given location is not within range of any radar stations",
                    )
                    .into())
                }
            };
            let station_statuses = get_station_statuses()?;
            let mut nearest_station = None;
            for station in nearby_stations {
                if station_statuses.iter().find(|s| s.0 == station).unwrap().1 {
                    nearest_station = Some(station.to_lowercase());
                    break;
                }
            }
            if nearest_station.is_none() {
                return Err(String::from(
                    "All radar stations within range of this location are offline",
                )
                .into());
            }
            nearest_station.unwrap()
        };
        let file_listing = get_data_file_listing(&station_code)?;
        // parse the file listing and determine the number of the second-most recent file
        let re = Regex::new(
            r#"sn\.(0\d{3}|last)</a></td><td align="right">(\d{2}-\w{3}-\d{4} \d{2}:\d{2})"#,
        )
        .unwrap();
        let mut files: Vec<(chrono::NaiveDateTime, String)> = re
            .captures_iter(&file_listing)
            .map(|cap| {
                (
                    chrono::NaiveDateTime::parse_from_str(&cap[2], "%d-%b-%Y %H:%M").unwrap(),
                    cap[1].to_string(),
                )
            })
            .collect();
        files.sort_by(|a, b| b.0.cmp(&a.0));
        let second_to_last_index = files[2].1.as_str();
        let sn_last = get_data_by_station(&station_code.to_lowercase(), "last")?;
        let sn_second_to_last =
            get_data_by_station(&station_code.to_lowercase(), second_to_last_index)?;
        (sn_second_to_last, sn_last)
    };

    let dpr_second_last = parse_dpr(input.0)?;
    let dpr_last = parse_dpr(input.1)?;
    let precip_second_last = dpr_second_last.sample_radials_to_equirectangular(256, 256);
    let precip_last = dpr_last.sample_radials_to_equirectangular(256, 256);
    let coords = {
        if matches.is_present("file") {
            let distance_from_station = get_distance_between_points(
                (latitude, longitude),
                (dpr_last.latitude, dpr_last.longitude),
            );
            if distance_from_station > 230. {
                return Err(format!(
                    "Supplied file contains data for station {}, but supplied point is outside coverage area ({} km away)",
                    dpr_last.station_code,
                    distance_from_station.round()).into());
            }
        }
        find_pixel_by_lat_long(&precip_last, latitude, longitude)?
    };

    let delta_t_image = (dpr_last.capture_time - dpr_second_last.capture_time).num_seconds() as u16;
    let delta_t_now = (chrono::Utc::now().timestamp() - dpr_last.capture_time.timestamp()) as u16;
    for (idx, prediction) in predict_two(
        [&precip_second_last, &precip_last],
        delta_t_image,
        delta_t_now,
    )
    .iter()
    .enumerate()
    {
        let precip_at_coords = prediction[coords.0][coords.1].1;
        match idx {
            0 => print!(" right now: "),
            _ => print!("in {: >2} mins: ", idx * 5),
        };
        println!(
            "{:.3} in/hr ({})",
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
    }

    Ok(())
}
