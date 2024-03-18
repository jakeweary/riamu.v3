use std::f32::consts::TAU as τ;

use super::*;

#[derive(Clone, Copy)]
pub struct LCh {
  pub L: f32,
  pub C: f32,
  pub h: f32,
}

impl From<LCh> for RGB {
  fn from(lch: LCh) -> Self {
    Lab::from(lch).into()
  }
}

impl From<RGB> for LCh {
  fn from(rgb: RGB) -> Self {
    Lab::from(rgb).into()
  }
}

impl From<LCh> for Lab {
  fn from(LCh { L, C, h }: LCh) -> Self {
    let (b, a) = (τ * h).sin_cos();
    let (b, a) = (C * b, C * a);
    Self { L, a, b }
  }
}

impl From<Lab> for LCh {
  fn from(Lab { L, a, b }: Lab) -> Self {
    // `atan2(-y, -x) + π` is the same as `atan2(y, x)`
    // but instead of -π..π range it gives 0..2π
    let h = 0.5 + (-b).atan2(-a) / τ;
    let C = a.hypot(b);
    Self { L, C, h }
  }
}
