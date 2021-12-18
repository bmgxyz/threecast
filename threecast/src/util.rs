use std::error::Error;

use crate::parse::{coord_as_i64, GridData};

#[allow(clippy::ptr_arg)]
pub fn find_pixel_by_lat_long(
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
