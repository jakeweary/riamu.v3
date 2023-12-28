macro_rules! impl_fns(($T:ident) => {
  pub mod $T {
    pub fn eotf(x: $T) -> $T {
      match x {
        x if x > 0.0404482362771082 => ((x + 0.055) / 1.055).powf(2.4),
        x => x / 12.92,
      }
    }

    pub fn oetf(x: $T) -> $T {
      match x {
        x if x > 0.00313066844250063 => 1.055 * x.powf(1.0 / 2.4) - 0.055,
        x => 12.92 * x,
      }
    }
  }
});

impl_fns!(f32);
impl_fns!(f64);
