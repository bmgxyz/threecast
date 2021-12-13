const EARTH_RADIUS_KM: f32 = 6371.;

/// Given a starting coordinate, a bearing, and a distance, compute the
/// destination coordinates. Coordinates are (latitude, longitude) in degrees,
/// bearing is in degrees clockwise from due north, and distance is in
/// kilometers. Should be accurate within 0.0005 degrees, but probably better.
///
/// Math copied from [here](http://www.movable-type.co.uk/scripts/latlong.html#dest-point).
pub fn get_point_bearing_distance(
    start_point: (f32, f32),
    bearing: f32,
    distance: f32,
) -> (f32, f32) {
    let (start_lat, start_lon) = (start_point.0.to_radians(), start_point.1.to_radians());
    let bearing_radians = bearing.to_radians();
    let delta = distance / EARTH_RADIUS_KM;
    let final_lat = (start_lat.sin() * delta.cos()
        + start_lat.cos() * delta.sin() * bearing_radians.cos())
    .asin();
    let final_lon = start_lon
        + (bearing_radians.sin() * delta.sin() * start_lat.cos())
            .atan2(delta.cos() - start_lat.sin() * final_lat.sin());
    (final_lat.to_degrees(), final_lon.to_degrees())
}

/// Given a pair of coordinates, compute the distance between the coordinates.
/// Coordinates are (latitude, longitude) in degrees and distance is in
/// kilometers.
///
/// Math copied from [here](http://www.movable-type.co.uk/scripts/latlong.html).
pub fn get_distance_between_points(start_point: (f32, f32), end_point: (f32, f32)) -> f32 {
    let (start_lat, start_lon) = (start_point.0.to_radians(), start_point.1.to_radians());
    let (end_lat, end_lon) = (end_point.0.to_radians(), end_point.1.to_radians());
    let haversine = ((end_lat - start_lat) / 2.).sin().powi(2)
        + start_lat.cos() * end_lat.cos() * ((end_lon - start_lon) / 2.).sin().powi(2);
    EARTH_RADIUS_KM * 2. * haversine.sqrt().atan2((1. - haversine).sqrt())
}

#[cfg(test)]
fn is_equal_within_error(test_value: f32, true_value: f32, error: f32) -> bool {
    test_value >= true_value - error && test_value <= true_value + error
}

#[test]
fn test_get_point_bearing_distance() {
    // https://xkcd.com/2170
    let error = 0.0005;
    let (lat, lon) = get_point_bearing_distance((53.320556, -1.729722), 96.021666667, 124.8);
    assert!(is_equal_within_error(lat, 53.188333, error));
    assert!(is_equal_within_error(lon, 0.133333, error));
    let (lat, lon) = get_point_bearing_distance((81.9289182, -126.645662), 38.848430, 198.5);
    assert!(is_equal_within_error(lat, 83.226667, error));
    assert!(is_equal_within_error(lon, -117.109167, error));
}

#[test]
fn test_get_distance_between_points() {
    let error = 0.1;
    let distance = get_distance_between_points((50.0664, -5.7147), (58.6439, -3.0700));
    assert!(is_equal_within_error(distance, 968.9, error));
    let distance = get_distance_between_points((32.1515, 1.5073), (33.2410, 1.7384));
    assert!(is_equal_within_error(distance, 123.1, error));
}
