use super::*;

#[derive(Clone, Copy)]
pub struct LMSʹ {
  pub lʹ: f32,
  pub mʹ: f32,
  pub sʹ: f32,
}

impl From<LMSʹ> for RGB {
  fn from(LMSʹ { lʹ, mʹ, sʹ }: LMSʹ) -> Self {
    let (l, m, s) = (lʹ.powi(3), mʹ.powi(3), sʹ.powi(3));
    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;
    RGB { r, g, b }
  }
}

impl From<RGB> for LMSʹ {
  fn from(RGB { r, g, b }: RGB) -> Self {
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
    let (lʹ, mʹ, sʹ) = (l.cbrt(), m.cbrt(), s.cbrt());
    LMSʹ { lʹ, mʹ, sʹ }
  }
}
