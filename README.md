# threecast

*Like a forecast, but smaller*

This project used to be a mildly useful precipitation nowcaster, but I've removed a lot of features
and focused on [doing one thing well][unix philosophy]. That "one thing" is to convert the National
Weather Service's (NWS) Digital Instantaneous Precipitation Rate (DIPR) radar product from [its
native data format][spec] into more common vector GIS formats. The two supported target formats are
[Shapefile][shapefile] and [GeoJSON][geojson]. Other tools are better at downloading the data or
processing it after conversion.

[unix philosophy]: https://en.wikipedia.org/wiki/Unix_philosophy
[spec]: https://www.roc.noaa.gov/public-documents/icds/2620001T.pdf
[shapefile]: https://en.wikipedia.org/wiki/Shapefile
[geojson]: https://en.wikipedia.org/wiki/GeoJSON

## Usage

### As a CLI Tool

1. Download data from [here][data]. You'll need to know the [station code][nws stations wiki] for
   the radar you're interested in. The file called `sn.last` is the most recent scan. The scan files
   are updated in a [circular][circular buffer] fashion.
2. Follow the help text generated with `cargo run --release -- -h`. The three possible subcommands
   are `info`, `to-geojson`, and `to-shapefile`. Help text is available for each subcommand.
3. After converting the radar data to one of the supported target formats, use other GIS tools to
   view or process it. For example, you can rasterize the resulting GeoJSON data with something
   like:

```bash
gdal_rasterize -l foo -a precipRate -ts 1920 1080 -a_nodata 0.0 -ot Float32 -of GTiff foo.geojson foo.tif > /dev/null
```

[nws stations wiki]: https://en.wikipedia.org/wiki/List_of_National_Weather_Service_Weather_Forecast_Offices
[data]: https://tgftp.nws.noaa.gov/SL.us008001/DF.of/DC.radar/DS.176pr/
[circular buffer]: https://en.wikipedia.org/wiki/Circular_buffer

### As a Library

The `dipr` library only provides one public function, `parse_dpr`, which takes `&[u8]` as input and
returns `Result<PrecipRate, DprError>`. `DprError` is an enum that either indicates a
product-specific parsing error or wraps a lower-level error. `PrecipRate` is a structure that
contains a useful subset of the data in the original file. The precipitation bin data is available
in the `radials` field. `ParseDpr` also has a `to_polygons` method that allows the user to extract
the precipitation bin data as `Vec<(Polygon<f32>, Velocity)>`, where each tuple contains a
`geo_types::geometry::Polygon` defining the bin's boundary and a `uom::si::f32::Velocity` indicating
the precipitation rate for that bin. All fields in all structs use types that encode semantic or
unit-aware meaning where possible.

## License

Copyright 2021-2025 Bradley Gannon

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
