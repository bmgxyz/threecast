use std::{
    error::Error,
    fs,
    io::{Read, stdin},
};

use clap::{Parser, Subcommand};
use digital_precip_rate::{PrecipRate, inch_per_hour, parse_dpr};
use geo::Polygon;
use geojson::{Feature, FeatureCollection, GeoJson, JsonObject, JsonValue};
use uom::si::f32::Velocity;

fn print_dpr_info(dpr: PrecipRate) {
    println!("{}", dpr);
}

fn convert_to_geojson(dpr: PrecipRate, skip_zeros: bool) -> Result<(), Box<dyn Error>> {
    let dpr_bins: Vec<(Polygon<f32>, Velocity)> = dpr.to_polygons(skip_zeros);
    let mut features = Vec::with_capacity(dpr_bins.len());
    for bin in dpr_bins {
        let (geometry, precip_rate) = bin;
        let mut properties = JsonObject::new();
        properties.insert(
            "precipRate".to_string(),
            JsonValue::from(precip_rate.get::<inch_per_hour>()),
        );
        features.push(Feature {
            geometry: Some((&geometry).into()),
            properties: Some(properties),
            ..Default::default()
        });
    }
    println!(
        "{}",
        GeoJson::FeatureCollection(FeatureCollection {
            features,
            ..Default::default()
        })
        .to_string()
    );
    Ok(())
}

/// Convert the NWS Digital Instantaneous Precipitation Rate product to common vector GIS formats
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct DiprCli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Parses the input DIPR product and prints a summary of its contents
    Info {
        /// Path to the DIPR product; if omitted or - (hyphen), read from stdin
        input: String,
    },
    /// Converts the input DIPR product to GeoJSON and writes it to stdout
    ToGeojson {
        /// Path to the DIPR product; if omitted or - (hyphen), read from stdin
        input: String,
        /// When producing the GeoJSON output, don't include bins with zero precipitation
        #[arg(long)]
        skip_zeros: bool,
    },
    /// Converts the input DIPR product to a Shapefile and writes it to the provided paths
    ToShapefile {
        /// Path to the DIPR product; if omitted or - (hyphen), read from stdin
        input: String,
        /// When producing the Shapefile output, don't include bins with zero precipitation
        #[arg(long)]
        skip_zeros: bool,
    },
}

fn convert_to_shapefile(dpr: PrecipRate, skip_zeros: bool) -> Result<(), Box<dyn Error>> {
    todo!()
}

fn read_and_convert(input: &str) -> Result<PrecipRate, Box<dyn Error>> {
    if input.is_empty() || input == "-" {
        let mut input_buf = vec![];
        stdin().read_to_end(&mut input_buf)?;
        Ok(parse_dpr(&input_buf)?)
    } else {
        let input_file = fs::read(input)?;
        Ok(parse_dpr(&input_file)?)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = DiprCli::parse();

    match args.action {
        Action::Info { input } => {
            let dpr = read_and_convert(&input)?;
            print_dpr_info(dpr)
        }
        Action::ToGeojson { input, skip_zeros } => {
            let dpr = read_and_convert(&input)?;
            convert_to_geojson(dpr, skip_zeros)?
        }
        Action::ToShapefile { input, skip_zeros } => {
            let dpr = read_and_convert(&input)?;
            convert_to_shapefile(dpr, skip_zeros)?
        }
    };

    Ok(())
}
