use chrono::prelude::*;

pub mod cairo;

pub fn capitalize(text: &str) -> String {
  let (first, rest) = text.split_at(1);
  first.to_ascii_uppercase() + rest
}

pub fn datetime(offset: i32, timestamp: i64) -> DateTime<FixedOffset> {
  let tz = FixedOffset::east_opt(offset).unwrap();
  let dt = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
  DateTime::from_naive_utc_and_offset(dt, tz)
}

pub fn beaufort_scale(wind_speed_ms: f64) -> &'static str {
  const TABLE: [&str; 12] = [
    "calm",
    "light air",
    "light breeze",
    "gentle breeze",
    "moderate breeze",
    "fresh breeze",
    "strong breeze",
    "high wind, near gale",
    "gale",
    "severe gale",
    "storm",
    "violent storm",
  ];

  let f = (wind_speed_ms / 0.836).powf(2.0 / 3.0);
  let i = f.round() as usize;
  TABLE.get(i).map_or("hurricane", |&s| s)
}
