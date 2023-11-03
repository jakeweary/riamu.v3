use std::fs;

use c::bindings::RsvgRectangle;
use c::rsvg::Handle as Rsvg;

use super::Result;

pub fn icon(name: &'static str) -> Result<(Rsvg, RsvgRectangle)> {
  let file = fs::read(format!("assets/icons/weather/{name}.svg"))?;
  let icon = Rsvg::from_data(&file)?;
  let size = icon.intrinsic_dimensions().viewbox.unwrap();
  Ok((icon, size))
}

pub fn openweather(name: &str) -> Result<(Rsvg, RsvgRectangle)> {
  icon(match name {
    "01d" => "clear-day",
    "01n" => "clear-night",
    "02d" => "partly-cloudy-day",
    "02n" => "partly-cloudy-night",
    "03d" | "03n" | "04d" | "04n" => "cloudy",
    "09d" | "09n" => "rain",
    "10d" => "partly-cloudy-day-rain",
    "10n" => "partly-cloudy-night-rain",
    "11d" | "11n" => "thunderstorms",
    "13d" => "partly-cloudy-day-snow",
    "13n" => "partly-cloudy-night-snow",
    "50d" | "50n" => "mist",
    _ => panic!(),
  })
}
