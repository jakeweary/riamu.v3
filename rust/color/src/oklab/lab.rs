use super::*;

#[derive(Clone, Copy)]
pub struct Lab {
  pub L: f32,
  pub a: f32,
  pub b: f32,
}

impl From<Lab> for RGB {
  fn from(lab: Lab) -> Self {
    LMSʹ::from(lab).into()
  }
}

impl From<RGB> for Lab {
  fn from(rgb: RGB) -> Self {
    LMSʹ::from(rgb).into()
  }
}

impl From<Lab> for LMSʹ {
  fn from(Lab { L, a, b }: Lab) -> Self {
    let lʹ = L + 0.3963377774 * a + 0.2158037573 * b;
    let mʹ = L - 0.1055613458 * a - 0.0638541728 * b;
    let sʹ = L - 0.0894841775 * a - 1.2914855480 * b;
    LMSʹ { lʹ, mʹ, sʹ }
  }
}

impl From<LMSʹ> for Lab {
  fn from(lmsʹ: LMSʹ) -> Self {
    let LMSʹ { lʹ, mʹ, sʹ } = lmsʹ;
    let L = 0.2104542553 * lʹ + 0.7936177850 * mʹ - 0.0040720468 * sʹ;
    let a = 1.9779984951 * lʹ - 2.4285922050 * mʹ + 0.4505937099 * sʹ;
    let b = 0.0259040371 * lʹ + 0.7827717662 * mʹ - 0.8086757660 * sʹ;
    Lab { L, a, b }
  }
}
