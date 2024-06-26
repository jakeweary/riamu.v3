use serde::Deserialize;

#[rustfmt::skip]
#[derive(Debug, Deserialize)]
pub struct Root {
  pub lat: f64,
  pub lon: f64,

  pub timezone: String,
  pub timezone_offset: i32,            // seconds

  // #[serde(default)]
  // pub minutely: Vec<Minute>,
  pub hourly: Vec<Hour>,
  pub daily: Vec<Day>,
  pub current: Current,

  // #[serde(default)]
  // pub alerts: Vec<Alert>,
}

#[rustfmt::skip]
#[derive(Debug, Deserialize)]
pub struct Current {
  pub dt: i64,                         // unix time
  pub sunrise: Option<i64>,            // unix time
  pub sunset: Option<i64>,             // unix time

  pub temp: f64,                       // °C
  pub feels_like: f64,                 // °C
  pub dew_point: f64,                  // °C

  pub humidity: u8,                    // %
  // pub clouds: u8,                   // %
  pub pressure: u16,                   // hPa
  pub uvi: f64,                        // uv index
  pub visibility: Option<u16>,         // m

  // pub wind_deg: u16,                // degrees (meteorological)
  pub wind_speed: f64,                 // m/s
  pub wind_gust: Option<f64>,          // m/s

  pub weather: Vec<Weather>,
  // pub rain: Option<Precipitation>,
  // pub snow: Option<Precipitation>,
}

// #[rustfmt::skip]
// #[derive(Debug, Deserialize)]
// pub struct Minute {
//   pub dt: i64,                      // unix time
//   pub precipitation: f64,           // mm/h
// }

#[rustfmt::skip]
#[derive(Debug, Deserialize)]
pub struct Hour {
  pub dt: i64,                         // unix time

  pub temp: f64,                       // °C
  pub feels_like: f64,                 // °C
  pub dew_point: f64,                  // °C

  // pub humidity: u8,                 // %
  pub clouds: u8,                      // %
  // pub pressure: u16,                // hPa
  pub uvi: f64,                        // uv index
  // pub visibility: Option<u16>,      // m

  pub wind_deg: u16,                   // degrees (meteorological)
  pub wind_speed: f64,                 // m/s
  pub wind_gust: f64,                  // m/s

  pub pop: f64,                        // probability
  pub rain: Option<Precipitation>,
  pub snow: Option<Precipitation>,

  pub weather: Vec<Weather>,
}

#[rustfmt::skip]
#[derive(Debug, Deserialize)]
pub struct Day {
  pub dt: i64,                         // unix time
  // pub sunrise: i64,                 // unix time (can be zero)
  // pub sunset: i64,                  // unix time (can be zero)
  // pub moonrise: i64,                // unix time
  // pub moonset: i64,                 // unix time
  // pub moon_phase: f64,

  pub temp: DayTempMinMax,
  // pub feels_like: DayTemp,
  // pub dew_point: f64,               // °C

  // pub humidity: u8,                 // %
  // pub clouds: u8,                   // %
  // pub pressure: u16,                // hPa
  // pub uvi: f64,                     // uv index

  pub wind_deg: u16,                   // degrees (meteorological)
  pub wind_speed: f64,                 // m/s
  pub wind_gust: f64,                  // m/s

  // pub pop: f64,                     // probability
  // pub rain: Option<f64>,            // mm/h
  // pub snow: Option<f64>,            // mm/h

  pub weather: Vec<Weather>,
}

#[rustfmt::skip]
#[derive(Debug, Deserialize)]
pub struct Weather {
  pub id: u16,
  pub main: String,
  pub description: String,
  pub icon: String,
}

#[rustfmt::skip]
#[derive(Debug, Deserialize)]
pub struct Precipitation {
  #[serde(rename = "1h")]
  pub one_hour: f64,                   // mm/h
}

// #[rustfmt::skip]
// #[derive(Debug, Deserialize)]
// pub struct DayTemp {
//   pub morn: f64,                    // °C
//   pub day: f64,                     // °C
//   pub eve: f64,                     // °C
//   pub night: f64,                   // °C
// }

#[rustfmt::skip]
#[derive(Debug, Deserialize)]
pub struct DayTempMinMax {
  // pub morn: f64,                    // °C
  // pub day: f64,                     // °C
  // pub eve: f64,                     // °C
  // pub night: f64,                   // °C
  pub min: f64,                        // °C
  pub max: f64,                        // °C
}

// #[rustfmt::skip]
// #[derive(Debug, Deserialize)]
// pub struct Alert {
//   pub start: i64,                   // unix time
//   pub end: i64,                     // unix time

//   pub event: String,
//   pub description: String,
//   pub sender_name: String,
//   pub tags: Vec<String>,
// }
