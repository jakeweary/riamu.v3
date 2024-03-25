#[derive(Clone, Copy)]
pub struct RGB {
  pub r: f32,
  pub g: f32,
  pub b: f32,
}

#[derive(Clone, Copy)]
pub struct Lab {
  pub L: f32,
  pub a: f32,
  pub b: f32,
}

#[derive(Clone, Copy)]
pub struct LMSʹ {
  pub lʹ: f32,
  pub mʹ: f32,
  pub sʹ: f32,
}

impl From<Lab> for RGB {
  fn from(Lab { L, a, b }: Lab) -> Self {
    let lʹ = L + 0.3963377774 * a + 0.2158037573 * b;
    let mʹ = L - 0.1055613458 * a - 0.0638541728 * b;
    let sʹ = L - 0.0894841775 * a - 1.2914855480 * b;
    LMSʹ { lʹ, mʹ, sʹ }.into()
  }
}

impl From<RGB> for Lab {
  fn from(rgb: RGB) -> Self {
    let LMSʹ { lʹ, mʹ, sʹ } = rgb.into();
    let L = 0.2104542553 * lʹ + 0.7936177850 * mʹ - 0.0040720468 * sʹ;
    let a = 1.9779984951 * lʹ - 2.4285922050 * mʹ + 0.4505937099 * sʹ;
    let b = 0.0259040371 * lʹ + 0.7827717662 * mʹ - 0.8086757660 * sʹ;
    Lab { L, a, b }
  }
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
