use std::error::Error;

use crate::geomath::get_distance_between_points;

pub struct Station {
    code: &'static str,
    latitude: f32,
    longitude: f32,
}

pub fn find_nearest_station(latitude: f32, longitude: f32) -> Result<&'static str, Box<dyn Error>> {
    let mut best_distance = 230.;
    let mut best_station_code = "";
    for station in STATIONS {
        let distance = get_distance_between_points(
            (latitude, longitude),
            (station.latitude, station.longitude),
        );
        if distance < best_distance {
            best_distance = distance;
            best_station_code = station.code;
        }
    }
    if best_distance < 230. {
        Ok(best_station_code)
    } else {
        Err(String::from("Given location is not within range of any radar stations").into())
    }
}

pub const STATIONS: [Station; 161] = [
    Station {
        code: "TJUA",
        latitude: 18.1155,
        longitude: -66.0780,
    },
    Station {
        code: "KCBW",
        latitude: 46.0391,
        longitude: -67.8066,
    },
    Station {
        code: "KGYX",
        latitude: 43.8913,
        longitude: -70.2565,
    },
    Station {
        code: "KCXX",
        latitude: 44.5109,
        longitude: -73.1664,
    },
    Station {
        code: "KBOX",
        latitude: 41.9558,
        longitude: -71.1369,
    },
    Station {
        code: "KENX",
        latitude: 42.5865,
        longitude: -74.0639,
    },
    Station {
        code: "KBGM",
        latitude: 42.1997,
        longitude: -75.9847,
    },
    Station {
        code: "KBUF",
        latitude: 42.9488,
        longitude: -78.7369,
    },
    Station {
        code: "KTYX",
        latitude: 43.7556,
        longitude: -75.6799,
    },
    Station {
        code: "KOKX",
        latitude: 40.8655,
        longitude: -72.8638,
    },
    Station {
        code: "KDOX",
        latitude: 38.8257,
        longitude: -75.4400,
    },
    Station {
        code: "KDIX",
        latitude: 39.9470,
        longitude: -74.4108,
    },
    Station {
        code: "KPBZ",
        latitude: 40.5316,
        longitude: -80.2179,
    },
    Station {
        code: "KCCX",
        latitude: 40.9228,
        longitude: -78.0038,
    },
    Station {
        code: "KRLX",
        latitude: 38.3110,
        longitude: -81.7229,
    },
    Station {
        code: "KAKQ",
        latitude: 36.9840,
        longitude: -77.0073,
    },
    Station {
        code: "KFCX",
        latitude: 37.0242,
        longitude: -80.2736,
    },
    Station {
        code: "KLWX",
        latitude: 38.9753,
        longitude: -77.4778,
    },
    Station {
        code: "KMHX",
        latitude: 34.7759,
        longitude: -76.8762,
    },
    Station {
        code: "KRAX",
        latitude: 35.6654,
        longitude: -78.4897,
    },
    Station {
        code: "KLTX",
        latitude: 33.9891,
        longitude: -78.4291,
    },
    Station {
        code: "KCLX",
        latitude: 32.6554,
        longitude: -81.0423,
    },
    Station {
        code: "KCAE",
        latitude: 33.9487,
        longitude: -81.1184,
    },
    Station {
        code: "KGSP",
        latitude: 34.8833,
        longitude: -82.2200,
    },
    Station {
        code: "KFFC",
        latitude: 33.3635,
        longitude: -84.5658,
    },
    Station {
        code: "KVAX",
        latitude: 30.8903,
        longitude: -83.0019,
    },
    Station {
        code: "KJGX",
        latitude: 32.6755,
        longitude: -83.3508,
    },
    Station {
        code: "KEVX",
        latitude: 30.5649,
        longitude: -85.9215,
    },
    Station {
        code: "KJAX",
        latitude: 30.4846,
        longitude: -81.7018,
    },
    Station {
        code: "KBYX",
        latitude: 24.5974,
        longitude: -81.7032,
    },
    Station {
        code: "KMLB",
        latitude: 28.1131,
        longitude: -80.6540,
    },
    Station {
        code: "KAMX",
        latitude: 25.6111,
        longitude: -80.4127,
    },
    Station {
        code: "KTLH",
        latitude: 30.3975,
        longitude: -84.3289,
    },
    Station {
        code: "KTBW",
        latitude: 27.7054,
        longitude: -82.4017,
    },
    Station {
        code: "KBMX",
        latitude: 33.1722,
        longitude: -86.7698,
    },
    Station {
        code: "KEOX",
        latitude: 31.4605,
        longitude: -85.4592,
    },
    Station {
        code: "KHTX",
        latitude: 34.9305,
        longitude: -86.0837,
    },
    Station {
        code: "KMXX",
        latitude: 32.5366,
        longitude: -85.7897,
    },
    Station {
        code: "KMOB",
        latitude: 30.6795,
        longitude: -88.2397,
    },
    Station {
        code: "KDGX",
        latitude: 32.2797,
        longitude: -89.9846,
    },
    Station {
        code: "KGWX",
        latitude: 33.8967,
        longitude: -88.3293,
    },
    Station {
        code: "KMRX",
        latitude: 36.1685,
        longitude: -83.4017,
    },
    Station {
        code: "KNQA",
        latitude: 35.3447,
        longitude: -89.8734,
    },
    Station {
        code: "KOHX",
        latitude: 36.2472,
        longitude: -86.5625,
    },
    Station {
        code: "KHPX",
        latitude: 36.7368,
        longitude: -87.2854,
    },
    Station {
        code: "KJKL",
        latitude: 37.5907,
        longitude: -83.3130,
    },
    Station {
        code: "KLVX",
        latitude: 37.9753,
        longitude: -85.9438,
    },
    Station {
        code: "KPAH",
        latitude: 37.0683,
        longitude: -88.7720,
    },
    Station {
        code: "KILN",
        latitude: 39.4202,
        longitude: -83.8216,
    },
    Station {
        code: "KCLE",
        latitude: 41.4131,
        longitude: -81.8597,
    },
    Station {
        code: "KDTX",
        latitude: 42.6999,
        longitude: -83.4718,
    },
    Station {
        code: "KAPX",
        latitude: 44.9071,
        longitude: -84.7198,
    },
    Station {
        code: "KGRR",
        latitude: 42.8938,
        longitude: -85.5449,
    },
    Station {
        code: "KMQT",
        latitude: 46.5311,
        longitude: -87.5487,
    },
    Station {
        code: "KVWX",
        latitude: 38.2603,
        longitude: -87.7246,
    },
    Station {
        code: "KIND",
        latitude: 39.7074,
        longitude: -86.2803,
    },
    Station {
        code: "KIWX",
        latitude: 41.3586,
        longitude: -85.7000,
    },
    Station {
        code: "KLOT",
        latitude: 41.6044,
        longitude: -88.0843,
    },
    Station {
        code: "KILX",
        latitude: 40.1505,
        longitude: -89.3368,
    },
    Station {
        code: "KGRB",
        latitude: 44.4984,
        longitude: -88.1111,
    },
    Station {
        code: "KARX",
        latitude: 43.8227,
        longitude: -91.1915,
    },
    Station {
        code: "KMKX",
        latitude: 42.9678,
        longitude: -88.5506,
    },
    Station {
        code: "KDLH",
        latitude: 46.8368,
        longitude: -92.2097,
    },
    Station {
        code: "KMPX",
        latitude: 44.8488,
        longitude: -93.5654,
    },
    Station {
        code: "KDVN",
        latitude: 41.6115,
        longitude: -90.5809,
    },
    Station {
        code: "KDMX",
        latitude: 41.7311,
        longitude: -93.7229,
    },
    Station {
        code: "KEAX",
        latitude: 38.8102,
        longitude: -94.2644,
    },
    Station {
        code: "KSGF",
        latitude: 37.2352,
        longitude: -93.4006,
    },
    Station {
        code: "KLSX",
        latitude: 38.6986,
        longitude: -90.6828,
    },
    Station {
        code: "KSRX",
        latitude: 35.2904,
        longitude: -94.3619,
    },
    Station {
        code: "KLZK",
        latitude: 34.8365,
        longitude: -92.2621,
    },
    Station {
        code: "KPOE",
        latitude: 31.1556,
        longitude: -92.9762,
    },
    Station {
        code: "KLCH",
        latitude: 30.1253,
        longitude: -93.2161,
    },
    Station {
        code: "KLIX",
        latitude: 30.3367,
        longitude: -89.8256,
    },
    Station {
        code: "KSHV",
        latitude: 32.4508,
        longitude: -93.8412,
    },
    Station {
        code: "KAMA",
        latitude: 35.2334,
        longitude: -101.7092,
    },
    Station {
        code: "KEWX",
        latitude: 29.7039,
        longitude: -98.0285,
    },
    Station {
        code: "KBRO",
        latitude: 25.9159,
        longitude: -97.4189,
    },
    Station {
        code: "KCRP",
        latitude: 27.7840,
        longitude: -97.5112,
    },
    Station {
        code: "KFWS",
        latitude: 32.5730,
        longitude: -97.3031,
    },
    Station {
        code: "KDYX",
        latitude: 32.5386,
        longitude: -99.2542,
    },
    Station {
        code: "KEPZ",
        latitude: 31.8731,
        longitude: -106.6979,
    },
    Station {
        code: "KGRK",
        latitude: 30.7217,
        longitude: -97.3829,
    },
    Station {
        code: "KHGX",
        latitude: 29.4718,
        longitude: -95.0788,
    },
    Station {
        code: "KDFX",
        latitude: 29.2730,
        longitude: -100.2802,
    },
    Station {
        code: "KLBB",
        latitude: 33.6541,
        longitude: -101.8141,
    },
    Station {
        code: "KMAF",
        latitude: 31.9433,
        longitude: -102.1894,
    },
    Station {
        code: "KSJT",
        latitude: 31.3712,
        longitude: -100.4925,
    },
    Station {
        code: "KFDR",
        latitude: 34.3620,
        longitude: -98.9766,
    },
    Station {
        code: "KTLX",
        latitude: 35.3333,
        longitude: -97.2778,
    },
    Station {
        code: "KOUN",
        latitude: 35.2358,
        longitude: -97.4622,
    },
    Station {
        code: "KINX",
        latitude: 36.1750,
        longitude: -95.5642,
    },
    Station {
        code: "KVNX",
        latitude: 36.7406,
        longitude: -98.1279,
    },
    Station {
        code: "KDDC",
        latitude: 37.7608,
        longitude: -99.9688,
    },
    Station {
        code: "KGLD",
        latitude: 39.3667,
        longitude: -101.7004,
    },
    Station {
        code: "KTWX",
        latitude: 38.9969,
        longitude: -96.2326,
    },
    Station {
        code: "KICT",
        latitude: 37.6545,
        longitude: -97.4431,
    },
    Station {
        code: "KUEX",
        latitude: 40.3209,
        longitude: -98.4418,
    },
    Station {
        code: "KLNX",
        latitude: 41.9579,
        longitude: -100.5759,
    },
    Station {
        code: "KOAX",
        latitude: 41.3202,
        longitude: -96.3667,
    },
    Station {
        code: "KABR",
        latitude: 45.4558,
        longitude: -98.4132,
    },
    Station {
        code: "KUDX",
        latitude: 44.1248,
        longitude: -102.8298,
    },
    Station {
        code: "KFSD",
        latitude: 43.5877,
        longitude: -96.7293,
    },
    Station {
        code: "KBIS",
        latitude: 46.7709,
        longitude: -100.7605,
    },
    Station {
        code: "KMVX",
        latitude: 47.5279,
        longitude: -97.3256,
    },
    Station {
        code: "KMBX",
        latitude: 48.3930,
        longitude: -100.8644,
    },
    Station {
        code: "KBLX",
        latitude: 45.8537,
        longitude: -108.6068,
    },
    Station {
        code: "KGGW",
        latitude: 48.2064,
        longitude: -106.6252,
    },
    Station {
        code: "KTFX",
        latitude: 47.4595,
        longitude: -111.3855,
    },
    Station {
        code: "KMSX",
        latitude: 47.0412,
        longitude: -113.9864,
    },
    Station {
        code: "KCYS",
        latitude: 41.1519,
        longitude: -104.806,
    },
    Station {
        code: "KRIW",
        latitude: 43.0660,
        longitude: -108.4773,
    },
    Station {
        code: "KFTG",
        latitude: 39.7866,
        longitude: -104.5458,
    },
    Station {
        code: "KGJX",
        latitude: 39.0619,
        longitude: -108.2137,
    },
    Station {
        code: "KPUX",
        latitude: 38.4595,
        longitude: -104.1816,
    },
    Station {
        code: "KABX",
        latitude: 35.1497,
        longitude: -106.8239,
    },
    Station {
        code: "KFDX",
        latitude: 34.6341,
        longitude: -103.6186,
    },
    Station {
        code: "KHDX",
        latitude: 33.0768,
        longitude: -106.12,
    },
    Station {
        code: "KFSX",
        latitude: 34.5744,
        longitude: -111.1983,
    },
    Station {
        code: "KIWA",
        latitude: 33.2891,
        longitude: -111.67,
    },
    Station {
        code: "KEMX",
        latitude: 31.8937,
        longitude: -110.6304,
    },
    Station {
        code: "KYUX",
        latitude: 32.4953,
        longitude: -114.6567,
    },
    Station {
        code: "KICX",
        latitude: 37.5908,
        longitude: -112.8622,
    },
    Station {
        code: "KMTX",
        latitude: 41.2627,
        longitude: -112.448,
    },
    Station {
        code: "KCBX",
        latitude: 43.4902,
        longitude: -116.236,
    },
    Station {
        code: "KSFX",
        latitude: 43.1055,
        longitude: -112.686,
    },
    Station {
        code: "KLRX",
        latitude: 40.7396,
        longitude: -116.8025,
    },
    Station {
        code: "KESX",
        latitude: 35.7012,
        longitude: -114.8918,
    },
    Station {
        code: "KRGX",
        latitude: 39.7541,
        longitude: -119.462,
    },
    Station {
        code: "KBBX",
        latitude: 39.4956,
        longitude: -121.6316,
    },
    Station {
        code: "KEYX",
        latitude: 35.0979,
        longitude: -117.5608,
    },
    Station {
        code: "KBHX",
        latitude: 40.4986,
        longitude: -124.2918,
    },
    Station {
        code: "KVTX",
        latitude: 34.4116,
        longitude: -119.1795,
    },
    Station {
        code: "KDAX",
        latitude: 38.5011,
        longitude: -121.6778,
    },
    Station {
        code: "KNKX",
        latitude: 32.9189,
        longitude: -117.0418,
    },
    Station {
        code: "KMUX",
        latitude: 37.1551,
        longitude: -121.8984,
    },
    Station {
        code: "KHNX",
        latitude: 36.3142,
        longitude: -119.632,
    },
    Station {
        code: "KSOX",
        latitude: 33.8176,
        longitude: -117.6359,
    },
    Station {
        code: "KVBG",
        latitude: 34.8383,
        longitude: -120.3977,
    },
    Station {
        code: "PHKI",
        latitude: 21.8938,
        longitude: -159.5524,
    },
    Station {
        code: "PHKM",
        latitude: 20.1254,
        longitude: -155.778,
    },
    Station {
        code: "PHMO",
        latitude: 21.1327,
        longitude: -157.1802,
    },
    Station {
        code: "PHWA",
        latitude: 19.0950,
        longitude: -155.5688,
    },
    Station {
        code: "KMAX",
        latitude: 42.0810,
        longitude: -122.7173,
    },
    Station {
        code: "KPDT",
        latitude: 45.6906,
        longitude: -118.8529,
    },
    Station {
        code: "KRTX",
        latitude: 45.7150,
        longitude: -122.965,
    },
    Station {
        code: "KLGX",
        latitude: 47.1168,
        longitude: -124.1062,
    },
    Station {
        code: "KATX",
        latitude: 48.1945,
        longitude: -122.4957,
    },
    Station {
        code: "KOTX",
        latitude: 47.6803,
        longitude: -117.6267,
    },
    Station {
        code: "PABC",
        latitude: 60.7919,
        longitude: -161.8765,
    },
    Station {
        code: "PAPD",
        latitude: 65.0351,
        longitude: -147.5014,
    },
    Station {
        code: "PAHG",
        latitude: 60.6156,
        longitude: -151.2832,
    },
    Station {
        code: "PAKC",
        latitude: 58.6794,
        longitude: -156.6293,
    },
    Station {
        code: "PAIH",
        latitude: 59.4619,
        longitude: -146.3011,
    },
    Station {
        code: "PAEC",
        latitude: 64.5114,
        longitude: -165.2949,
    },
    Station {
        code: "PACG",
        latitude: 56.8521,
        longitude: -135.5524,
    },
    Station {
        code: "PGUA",
        latitude: 13.4559,
        longitude: 144.8111,
    },
    Station {
        code: "LPLA",
        latitude: 38.7302,
        longitude: -27.3216,
    },
    Station {
        code: "RKJK",
        latitude: 35.9241,
        longitude: 126.6222,
    },
    Station {
        code: "RKSG",
        latitude: 37.2076,
        longitude: 127.2856,
    },
    Station {
        code: "RODN",
        latitude: 26.3077,
        longitude: 127.9034,
    },
];
