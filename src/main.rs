use std::{
    error::Error,
    fs,
    io::{Read, stdin},
};

use clap::{Parser, Subcommand};
use dipr::{PrecipRate, parse_dipr};
use geojson::{FeatureCollection, GeoJson};
use shapefile::{
    Writer,
    dbase::{Record, TableWriterBuilder},
};

fn read_and_convert(input: &str) -> Result<PrecipRate, Box<dyn Error>> {
    if input == "-" {
        let mut input_buf = vec![];
        stdin().read_to_end(&mut input_buf)?;
        Ok(parse_dipr(&input_buf)?)
    } else {
        let input_file = fs::read(input)?;
        Ok(parse_dipr(&input_file)?)
    }
}

fn convert_to_shapefile(
    dipr: PrecipRate,
    skip_zeros: bool,
    output: &str,
) -> Result<(), Box<dyn Error>> {
    const PRECIP_RATE_FIELD_NAME: &str = "Precip Rate";
    let table_builder =
        TableWriterBuilder::new().add_float_field(PRECIP_RATE_FIELD_NAME.try_into().unwrap(), 5, 3);
    let mut writer = Writer::from_path(output, table_builder)?;
    let mut record = Record::default();
    for bin in dipr.into_shapefile_iter(skip_zeros) {
        let (polygon, precip_rate) = bin;
        record.insert(PRECIP_RATE_FIELD_NAME.to_string(), precip_rate);
        writer.write_shape_and_record(&polygon, &record)?;
    }
    Ok(())
}

fn convert_to_geojson(dipr: PrecipRate, skip_zeros: bool) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        GeoJson::FeatureCollection(FeatureCollection {
            features: dipr.into_geojson_iter(skip_zeros).collect(),
            ..Default::default()
        })
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
        /// Path to the DIPR product; if equal to - (hyphen), read from stdin
        input: String,
    },
    /// Converts the input DIPR product to GeoJSON and writes it to stdout
    ToGeojson {
        /// Path to the DIPR product; if equal to - (hyphen), read from stdin
        input: String,
        /// When producing the GeoJSON output, don't include bins with zero precipitation
        #[arg(long)]
        skip_zeros: bool,
    },
    /// Converts the input DIPR product to a Shapefile and writes it to the provided paths
    ToShapefile {
        /// Path to the DIPR product; if equal to - (hyphen), read from stdin
        input: String,
        /// When producing the Shapefile output, don't include bins with zero precipitation
        #[arg(long)]
        skip_zeros: bool,
        /// Path to the output Shapefile; e.g., /path/to/foo.shp becomes
        /// /path/to/foo{.shp,.shx,.dbf}
        output: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = DiprCli::parse();

    match args.action {
        Action::Info { input } => {
            let dipr = read_and_convert(&input)?;
            println!("{}", dipr);
        }
        Action::ToGeojson { input, skip_zeros } => {
            let dipr = read_and_convert(&input)?;
            convert_to_geojson(dipr, skip_zeros)?;
        }
        Action::ToShapefile {
            input,
            skip_zeros,
            output,
        } => {
            let dipr = read_and_convert(&input)?;
            convert_to_shapefile(dipr, skip_zeros, &output)?
        }
    };

    Ok(())
}
