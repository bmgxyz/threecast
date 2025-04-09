use std::{
    error::Error,
    fs,
    io::{Read, stdin},
};

use clap::{Parser, Subcommand};
use digital_precip_rate::{PrecipRate, inch_per_hour, parse_dpr};
use geo::{CoordsIter, Polygon as GeoPolygon};
use geojson::{Feature, FeatureCollection, GeoJson, JsonObject, JsonValue};
use shapefile::{
    Point, Polygon as ShapefilePolygon, PolygonRing, Writer,
    dbase::{self, Record, TableWriterBuilder},
};
use uom::si::f32::Velocity;

fn convert_to_geojson(dpr: PrecipRate, skip_zeros: bool) -> Result<(), Box<dyn Error>> {
    let dpr_bins: Vec<(GeoPolygon<f32>, Velocity)> = dpr.to_polygons(skip_zeros);
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
        /// Path to the output Shapefile; e.g., /path/to/foo becomes /path/to/foo{.shp,.shx,.dbf}
        output: String,
    },
}

fn convert_to_shapefile(
    dpr: PrecipRate,
    skip_zeros: bool,
    output: &str,
) -> Result<(), Box<dyn Error>> {
    const PRECIP_RATE_FIELD_NAME: &str = "Precip Rate";
    let table_builder =
        TableWriterBuilder::new().add_float_field(PRECIP_RATE_FIELD_NAME.try_into().unwrap(), 5, 3);
    let mut writer = Writer::from_path(output.to_string() + ".shp", table_builder)?;
    let mut record = Record::default();
    let dpr_bins: Vec<(GeoPolygon<f32>, Velocity)> = dpr.to_polygons(skip_zeros);
    for bin in dpr_bins {
        let (geometry, precip_rate) = bin;
        let polygon = ShapefilePolygon::new(PolygonRing::Outer(
            geometry
                .coords_iter()
                .map(|c| Point::new(c.x.into(), c.y.into()))
                .collect::<Vec<Point>>(),
        ));
        record.insert(
            PRECIP_RATE_FIELD_NAME.to_string(),
            dbase::FieldValue::Float(Some(precip_rate.get::<inch_per_hour>())),
        );
        writer.write_shape_and_record(&polygon, &record)?;
    }
    Ok(())
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
            println!("{}", dpr);
        }
        Action::ToGeojson { input, skip_zeros } => {
            let dpr = read_and_convert(&input)?;
            convert_to_geojson(dpr, skip_zeros)?
        }
        Action::ToShapefile {
            input,
            skip_zeros,
            output,
        } => {
            let dpr = read_and_convert(&input)?;
            convert_to_shapefile(dpr, skip_zeros, &output)?
        }
    };

    Ok(())
}
