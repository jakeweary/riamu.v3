#![allow(dead_code)]

use glam::{Mat3, Vec3};

// https://github.com/selfshadow/ltc_code/blob/master/webgl/shaders/ltc/ltc_blit.fs
// https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
pub fn aces(color: Vec3) -> Vec3 {
  // sRGB => XYZ => D65_2_D60 => AP1 => RRT_SAT
  const M1: Mat3 = Mat3::from_cols_array_2d(&[
    [0.59719, 0.07600, 0.02840],
    [0.35458, 0.90834, 0.13383],
    [0.04823, 0.01566, 0.83777],
  ]);

  // ODT_SAT => XYZ => D60_2_D65 => sRGB
  const M2: Mat3 = Mat3::from_cols_array_2d(&[
    [1.60475, -0.10208, -0.00327],
    [-0.53108, 1.10813, -0.07276],
    [-0.07367, -0.00605, 1.07602],
  ]);

  // RRT and ODT fit
  let v = M1 * color;
  let a = v * (v + 0.0245786) - 0.000090537;
  let b = v * (0.983729 * v + 0.432_951) + 0.238081;
  M2 * (a / b)
}

pub fn srgb_oetf(value: f32) -> f32 {
  if value > 0.003_130_668_5 {
    value.powf(1.0 / 2.4).mul_add(1.055, -0.055)
  } else {
    12.92 * value
  }
}

pub fn srgb_eotf(value: f32) -> f32 {
  if value > 0.040_448_237 {
    ((value + 0.055) / 1.055).powf(2.4)
  } else {
    value / 12.92
  }
}

pub fn to_bytes(rgb: Vec3) -> [u8; 3] {
  let r = srgb_oetf(rgb.x).mul_add(255.0, 0.5);
  let g = srgb_oetf(rgb.y).mul_add(255.0, 0.5);
  let b = srgb_oetf(rgb.z).mul_add(255.0, 0.5);
  [r as u8, g as u8, b as u8]
}

pub fn from_bytes([r, g, b]: [u8; 3]) -> Vec3 {
  let r = srgb_eotf(r as f32 / 255.0);
  let g = srgb_eotf(g as f32 / 255.0);
  let b = srgb_eotf(b as f32 / 255.0);
  Vec3::new(r, g, b)
}
