use std::f32::consts::TAU;

use super::*;

#[derive(Clone, Copy)]
pub struct HSV {
  pub h: f32,
  pub s: f32,
  pub v: f32,
}

impl From<HSV> for RGB {
  fn from(HSV { h, s, v }: HSV) -> Self {
    let (aʹ, bʹ) = ((TAU * h).cos(), (TAU * h).sin());
    let (L_cusp, C_cusp) = find_cusp(aʹ, bʹ);
    let (S_max, T_max) = to_ST(L_cusp, C_cusp);
    let S_0 = 0.5;
    let k = 1.0 - S_0 / S_max;

    // first we compute L and V as if the gamut is a perfect triangle:

    // L, C when v==1:
    let kʹ = s * S_0 / (S_0 + T_max - T_max * k * s);
    let (L_v, C_v) = (1.0 - kʹ, T_max * kʹ);
    let (L, C) = (L_v * v, C_v * v);

    // then we compensate for both toe and the curved top part of the triangle:
    let L_vt = toe::inv(L_v);
    let C_vt = C_v * L_vt / L_v;

    let Lʹ = toe::inv(L);
    let (L, C) = (Lʹ, Lʹ / L * C);

    let L_scale = {
      let (L, a, b) = (L_vt, C_vt * aʹ, C_vt * bʹ);
      let RGB { r, g, b } = Lab { L, a, b }.into();
      (1.0 / r.max(g).max(b).max(0.0)).cbrt()
    };

    let (L, C) = (L_scale * L, L_scale * C);
    let (a, b) = (C * aʹ, C * bʹ);
    Lab { L, a, b }.into()
  }
}

impl From<RGB> for HSV {
  fn from(rgb: RGB) -> Self {
    let Lab { L, a, b } = rgb.into();
    let h = 0.5 + (-b).atan2(-a) / TAU;
    let C = a.hypot(b);
    let (aʹ, bʹ) = (a / C, b / C);
    let (L_cusp, C_cusp) = find_cusp(aʹ, bʹ);
    let (S_max, T_max) = to_ST(L_cusp, C_cusp);
    let S_0 = 0.5;
    let k = 1.0 - S_0 / S_max;

    // first we find L_v, C_v, L_vt and C_vt
    let t = T_max / (C + L * T_max);
    let (L_v, C_v) = (t * L, t * C);

    let L_vt = toe::inv(L_v);
    let C_vt = C_v * L_vt / L_v;

    // we can then use these to invert the step that compensates for the toe
    // and the curved top part of the triangle:
    let L_scale = {
      let (L, a, b) = (L_vt, C_vt * aʹ, C_vt * bʹ);
      let RGB { r, g, b } = Lab { L, a, b }.into();
      (1.0 / r.max(g).max(b).max(0.0)).cbrt()
    };

    let L = toe::f(L / L_scale);

    // we can now compute v and s:
    let v = L / L_v;
    let s = (S_0 + T_max) * C_v / (T_max * S_0 + T_max * k * C_v);

    HSV { h, s, v }
  }
}
