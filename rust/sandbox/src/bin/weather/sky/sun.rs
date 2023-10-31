#![allow(non_snake_case)]

use chrono::prelude::*;

// https://en.wikipedia.org/wiki/Solar_zenith_angle
// https://en.wikipedia.org/wiki/Solar_azimuth_angle
// https://doi.org/10.1016/j.renene.2021.03.047
pub fn zenith_azimuth(utc: NaiveDateTime, lat: f64, lon: f64) -> (f64, f64) {
  // Astronomical Almanac for the Year 2019, Page C5
  // https://archive.org/details/binder1_202003/page/n213/mode/1up
  let n = days_since_J2000(utc);
  let L = (280.460 + 0.9856474 * n).to_radians();
  let g = (357.528 + 0.9856003 * n).to_radians();
  let λ = L + (1.915 * g.sin() + 0.020 * (2.0 * g).sin()).to_radians();
  let ϵ = (23.439 - 0.0000004 * n).to_radians();
  let α = (ϵ.cos() * λ.sin()).atan2(λ.cos());
  let δ = (ϵ.sin() * λ.sin()).asin();
  let E = L - α;

  // the equation of time in minutes and the GMT in hours
  let E_min = (E.to_degrees() + 180.0) % 360.0 - 180.0;
  let T_GMT = hours_since_midnight(utc);

  // the latitude and longitude of the observer
  let ϕo = lat.to_radians();
  let λo = lon.to_radians();

  // the latitude and longitude of the subsolar point
  let ϕs = δ;
  let λs = -15.0 * (T_GMT - 12.0 + E_min / 60.0).to_radians();

  // the unit vector pointing toward the Sun
  let Sx = ϕs.cos() * (λs - λo).sin();
  let Sy = ϕo.cos() * ϕs.sin() - ϕo.sin() * ϕs.cos() * (λs - λo).cos();
  let Sz = ϕo.sin() * ϕs.sin() + ϕo.cos() * ϕs.cos() * (λs - λo).cos();

  // the solar zenith angle
  let Z = Sz.acos();
  let Z = Z.to_degrees();

  // the solar azimuth angle
  // let γs = Sy.atan2(Sx); // East-Counterclockwise Convention
  // let γs = Sx.atan2(Sy); // North-Clockwise Convention
  let γs = (-Sx).atan2(-Sy); // South-Clockwise Convention
  let γs = γs.to_degrees();

  (Z, γs)
}

fn days_since_J2000(dt: NaiveDateTime) -> f64 {
  let d = NaiveDate::from_yo_opt(2000, 1).unwrap();
  let t = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
  let dur = dt.signed_duration_since(d.and_time(t));
  dur.num_seconds() as f64 / (24 * 60 * 60) as f64
}

fn hours_since_midnight(dt: NaiveDateTime) -> f64 {
  dt.num_seconds_from_midnight() as f64 / (60 * 60) as f64
}
