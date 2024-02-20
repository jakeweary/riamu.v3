use std::f32::consts::TAU;

use super::*;

#[derive(Clone, Copy)]
pub struct HSL {
  pub h: f32,
  pub s: f32,
  pub l: f32,
}

impl From<HSL> for RGB {
  fn from(hsl: HSL) -> Self {
    Lab::from(hsl).into()
  }
}

impl From<RGB> for HSL {
  fn from(rgb: RGB) -> Self {
    Lab::from(rgb).into()
  }
}

impl From<HSL> for Lab {
  fn from(HSL { h, s, l }: HSL) -> Self {
    // TODO: wound ne nice to get rid of these
    if l == 1.0 {
      return Lab { L: 1.0, a: 0.0, b: 0.0 };
    }
    if l == 0.0 {
      return Lab { L: 0.0, a: 0.0, b: 0.0 };
    }

    let L = toe::inv(l);
    let (aʹ, bʹ) = ((TAU * h).cos(), (TAU * h).sin());
    let (C_0, C_mid, C_max) = get_Cs(L, aʹ, bʹ);

    let mid = 0.8;
    let mid_inv = 1.25;

    let (t, k_0, k_1, k_2) = if s < mid {
      let t = mid_inv * s;
      let k_0 = 0.0;
      let k_1 = mid * C_0;
      let k_2 = 1.0 - k_1 / C_mid;
      (t, k_0, k_1, k_2)
    } else {
      let t = (s - mid) / (1.0 - mid);
      let k_0 = C_mid;
      let k_1 = (1.0 - mid) * C_mid * C_mid * mid_inv * mid_inv / C_0;
      let k_2 = 1.0 - k_1 / (C_max - C_mid);
      (t, k_0, k_1, k_2)
    };

    let C = k_0 + t * k_1 / (1.0 - k_2 * t);
    let (a, b) = (C * aʹ, C * bʹ);
    Lab { L, a, b }
  }
}

impl From<Lab> for HSL {
  fn from(Lab { L, a, b }: Lab) -> Self {
    let h = 0.5 + (-b).atan2(-a) / TAU;
    let C = a.hypot(b);
    let (aʹ, bʹ) = (a / C, b / C);
    let (C_0, C_mid, C_max) = get_Cs(L, aʹ, bʹ);

    let mid = 0.8;
    let mid_inv = 1.25;

    // FIXME: apparently `s` can be NaN
    // need to fix it somehow on my own because
    // the og C++ implementation has the same bug
    let s = if C < C_mid {
      let k_1 = mid * C_0;
      let k_2 = 1.0 - k_1 / C_mid;
      let t = C / (k_1 + k_2 * C);
      t * mid
    } else {
      let k_0 = C_mid;
      let k_1 = (1.0 - mid) * C_mid * C_mid * mid_inv * mid_inv / C_0;
      let k_2 = 1.0 - k_1 / (C_max - C_mid);
      let t = (C - k_0) / (k_1 + k_2 * (C - k_0));
      mid + (1.0 - mid) * t
    };

    let l = toe::f(L);
    HSL { h, s, l }
  }
}
