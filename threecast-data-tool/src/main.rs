use clap::{App, Arg, SubCommand};
use std::error::Error;
use threecast::parse::PrecipRate;
use threecast::stations::STATIONS;

fn compute_precip_fraction(dpr: &PrecipRate) -> f32 {
    let mut rainy_bins = 0.0;
    let mut total_bins = 0.0;
    for radial in dpr.radials.iter() {
        for bin in radial.precip_rates.iter() {
            if bin > &0.0 {
                rainy_bins += 1.0;
            }
            total_bins += 1.0;
        }
    }
    rainy_bins / total_bins
}

fn collect_data(station: &str, target_precip_fraction: f32) {
    let sleep_duration_sec = 180;
    let mut first_run = true;
    let mut last_scan_number = -1; // scan numbers are between 1 and 80, inclusive
    loop {
        if !first_run {
            // sleep for a random-ish amount of time
            // without this, the threads tend to collect into time clusters
            let random_extra_seconds = (chrono::offset::Utc::now().timestamp_nanos() % 30) as u64;
            println!(
                "[{}] sleeping for {} seconds",
                station,
                sleep_duration_sec + random_extra_seconds
            );
            std::thread::sleep(std::time::Duration::from_secs(180 + random_extra_seconds));
        }
        first_run = false;
        let dpr_data = match threecast::net::get_data_by_station(station, "last") {
            Ok(d) => {
                println!("[{}] got data", station);
                d
            }
            Err(e) => {
                println!("[{}] failed to get data: {}", station, e);
                continue;
            }
        };
        let dpr = match threecast::parse::parse_dpr(dpr_data.clone()) {
            Ok(d) => {
                println!("[{}] parsed data", station);
                d
            }
            Err(e) => {
                println!("[{}] failed to parse data: {}", station, e);
                continue;
            }
        };
        if dpr.scan_number != last_scan_number {
            println!("[{}] data file is new", station);
            last_scan_number = dpr.scan_number;
            let precip_fraction = compute_precip_fraction(&dpr);
            if precip_fraction >= target_precip_fraction {
                println!(
                    "[{}] data file exceeds precipitation threshold ({:.4} >= {:.4})",
                    station, precip_fraction, target_precip_fraction
                );
                let write_result = std::fs::write(
                    format!(
                        "./{}-{}-{:0>2}.nexrad", // TODO: use path from CLI arg
                        station.to_uppercase(),
                        dpr.capture_time.format("%Y-%m-%dT%H:%M:%SZ"),
                        dpr.scan_number
                    ),
                    dpr_data,
                );
                if let Err(e) = write_result {
                    println!("[{}] failed to write data file to disk: {}", station, e);
                } else {
                    println!("[{}] wrote data file to disk", station);
                }
            } else {
                println!(
                    "[{}] data file does not exceed preciptation threshold ({:.4} < {:.4})",
                    station, precip_fraction, target_precip_fraction
                );
            }
        } else {
            println!("[{}] data file is old", station);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("threecast-data-tool")
        .version("0.1.0")
        .author("Bradley Gannon <bradley@bradleygannon.com>")
        .about("Makes it easier to gather DPR data and test prediction methods")
        .subcommand(
            SubCommand::with_name("collect")
                .about("gather DPR data from the NWS Web server")
                .arg(
                    Arg::with_name("stations")
                        .short("s")
                        .long("stations")
                        .value_name("STATIONS")
                        .help("Comma-separated list of stations to consider (e.g. KGYX,KLWX)")
                        .takes_value(true)
                        .default_value("all")
                        .conflicts_with("exclude"),
                )
                .arg(
                    Arg::with_name("precip-threshold")
                        .short("p")
                        .long("precip-threshold")
                        .value_name("THRESHOLD")
                        .help("Fraction that must have nonzero precip (e.g. 0.3)")
                        .takes_value(true)
                        .default_value("0.0"),
                )
                .arg(
                    Arg::with_name("output-dir")
                        .short("o")
                        .long("outdir")
                        .value_name("OUTDIR")
                        .help("Directory to collect data in")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("test")
                .about("run prediction algorithms on a dataset to compare their accuracy")
                .arg(
                    Arg::with_name("dataset")
                        .short("d")
                        .long("dataset")
                        .value_name("DATASET")
                        .help("Directory containing input data files")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("collect") {
        // collect data for each station independently
        // note that we need heap strings here because we'll need to move them into other threads later on
        let stations: Vec<String> = match matches.value_of("stations") {
            None | Some("all") => STATIONS.iter().map(|s| s.code.to_string()).collect(),
            Some(s) => {
                let split_stations: Vec<String> = s.split(',').map(|s| s.to_string()).collect();
                for station in split_stations.iter() {
                    if !STATIONS
                        .iter()
                        .map(|s| s.code.to_string())
                        .any(|x| &x == station)
                    {
                        return Err(format!("'{}' is not a valid station", station).into());
                    }
                }
                split_stations
            }
        };

        let precip_threshold = match matches.value_of("precip-threshold").unwrap().parse::<f32>() {
            Ok(p) => p,
            Err(_) => return Err("Failed to parse precipitation threshold".into()),
        };

        let output_directory = std::path::Path::new(matches.value_of("output-dir").unwrap());
        if !output_directory.exists() {
            return Err(
                format!("Directory doesn't exist: '{}'", output_directory.display()).into(),
            );
        }

        for station in stations {
            std::thread::spawn(move || {
                collect_data(&station, precip_threshold);
            });
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        loop {
            std::thread::sleep(std::time::Duration::from_secs(999));
        }
    } else if let Some(_matches) = matches.subcommand_matches("test") {
        unimplemented!();
    }
    Ok(())
}
