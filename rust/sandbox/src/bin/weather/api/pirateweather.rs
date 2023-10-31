use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
  pub latitude: f64,
  pub longitude: f64,
  pub timezone: String,
  pub offset: f64,
  pub elevation: i64,
  pub currently: Currently,
  pub minutely: Minutely,
  pub hourly: Hourly,
  pub daily: Daily,
  pub flags: Flags,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currently {
  pub time: i64,
  pub summary: String,
  pub icon: String,
  pub nearest_storm_distance: i64,
  pub nearest_storm_bearing: i64,
  pub precip_intensity: f64,
  pub precip_probability: f64,
  pub precip_intensity_error: f64,
  pub precip_type: String,
  pub temperature: f64,
  pub apparent_temperature: f64,
  pub dew_point: f64,
  pub humidity: f64,
  pub pressure: f64,
  pub wind_speed: f64,
  pub wind_gust: f64,
  pub wind_bearing: i64,
  pub cloud_cover: f64,
  pub uv_index: f64,
  pub visibility: f64,
  pub ozone: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Minutely {
  pub summary: String,
  pub icon: String,
  pub data: Vec<Minute>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Minute {
  pub time: i64,
  pub precip_intensity: f64,
  pub precip_probability: f64,
  pub precip_intensity_error: f64,
  pub precip_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hourly {
  pub summary: String,
  pub icon: String,
  pub data: Vec<Hour>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hour {
  pub time: i64,
  pub icon: String,
  pub summary: String,
  pub precip_intensity: f64,
  pub precip_probability: f64,
  pub precip_intensity_error: f64,
  pub precip_accumulation: f64,
  pub precip_type: String,
  pub temperature: f64,
  pub apparent_temperature: f64,
  pub dew_point: f64,
  pub humidity: f64,
  pub pressure: f64,
  pub wind_speed: f64,
  pub wind_gust: f64,
  pub wind_bearing: i64,
  pub cloud_cover: f64,
  pub uv_index: f64,
  pub visibility: f64,
  pub ozone: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daily {
  pub summary: String,
  pub icon: String,
  pub data: Vec<Day>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
  pub time: i64,
  pub icon: String,
  pub summary: String,
  pub sunrise_time: i64,
  pub sunset_time: i64,
  pub moon_phase: f64,
  pub precip_intensity: f64,
  pub precip_intensity_max: f64,
  pub precip_intensity_max_time: i64,
  pub precip_probability: f64,
  pub precip_accumulation: f64,
  pub precip_type: String,
  pub temperature_high: f64,
  pub temperature_high_time: i64,
  pub temperature_low: f64,
  pub temperature_low_time: i64,
  pub apparent_temperature_high: f64,
  pub apparent_temperature_high_time: i64,
  pub apparent_temperature_low: f64,
  pub apparent_temperature_low_time: i64,
  pub dew_point: f64,
  pub humidity: f64,
  pub pressure: f64,
  pub wind_speed: f64,
  pub wind_gust: f64,
  pub wind_gust_time: i64,
  pub wind_bearing: i64,
  pub cloud_cover: f64,
  pub uv_index: f64,
  pub uv_index_time: i64,
  pub visibility: f64,
  pub temperature_min: f64,
  pub temperature_min_time: i64,
  pub temperature_max: f64,
  pub temperature_max_time: i64,
  pub apparent_temperature_min: f64,
  pub apparent_temperature_min_time: i64,
  pub apparent_temperature_max: f64,
  pub apparent_temperature_max_time: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
  pub sources: Vec<String>,
  pub source_times: SourceTimes,
  #[serde(rename = "nearest-station")]
  pub nearest_station: i64,
  pub units: String,
  pub version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTimes {
  pub gfs: String,
  pub gefs: String,
}
