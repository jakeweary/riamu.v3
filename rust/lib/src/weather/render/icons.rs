use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;

use ::c::bindings as c;
use ::c::rsvg::Handle as Rsvg;

pub fn icon<T>(name: &'static str, f: &dyn Fn(&Rsvg, c::RsvgRectangle) -> T) -> T {
  thread_local! {
    static ICONS: RefCell<HashMap<&'static str, Rsvg>> = HashMap::new().into();
  }

  ICONS.with(|icons| {
    let mut icons = icons.borrow_mut();
    let icon = icons.entry(name).or_insert_with(|| {
      let path = format!("assets/icons/weather/{name}.svg");
      let file = fs::read(path).unwrap();
      Rsvg::from_data(&file).unwrap()
    });
    let size = icon.intrinsic_dimensions().viewbox.unwrap();
    f(icon, size)
  })
}

pub fn openweather<T>(name: &str, f: &dyn Fn(&Rsvg, c::RsvgRectangle) -> T) -> T {
  let name = match name {
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
    _ => unreachable!(),
  };
  icon(name, f)
}
