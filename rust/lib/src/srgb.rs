#[derive(Clone, Copy)]
pub struct Srgb<T, const N: usize>([T; N]);

// Srgb<u8, 3> ↔ u32

impl From<u32> for Srgb<u8, 3> {
  fn from(argb: u32) -> Self {
    let [b, g, r, _] = argb.to_le_bytes();
    Self([r, g, b])
  }
}

impl From<Srgb<u8, 3>> for u32 {
  fn from(srgb: Srgb<u8, 3>) -> Self {
    let [r, g, b] = srgb.0;
    u32::from_le_bytes([b, g, r, 0])
  }
}

// Srgb<u8, 4> ↔ u32

impl From<u32> for Srgb<u8, 4> {
  fn from(argb: u32) -> Self {
    Self(argb.to_be_bytes())
  }
}

impl From<Srgb<u8, 4>> for u32 {
  fn from(srgb: Srgb<u8, 4>) -> Self {
    u32::from_be_bytes(srgb.0)
  }
}

// Srgb<T, N> ↔ [T; N]

impl<T, const N: usize> From<[T; N]> for Srgb<T, N> {
  fn from(argb: [T; N]) -> Self {
    Self(argb)
  }
}

impl<T, const N: usize> From<Srgb<T, N>> for [T; N] {
  fn from(srgb: Srgb<T, N>) -> Self {
    srgb.0
  }
}

// Srgb<u8, N> ↔ Srgb<N, f32>
// Srgb<u8, N> ↔ Srgb<N, f64>
// and transfer functions

macro_rules! impls(($ty:ident) => {
  impl<const N: usize> From<Srgb<$ty, N>> for Srgb<u8, N> {
    fn from(srgb: Srgb<$ty, N>) -> Self {
      Srgb(srgb.0.map(|x| (x * 0x100 as $ty) as u8))
    }
  }

  impl<const N: usize> From<Srgb<u8, N>> for Srgb<$ty, N> {
    fn from(srgb: Srgb<u8, N>) -> Self {
      Srgb(srgb.0.map(|x| x as $ty / 0xff as $ty))
    }
  }

  impl<const N: usize> Srgb<$ty, N> {
    pub fn eotf(self) -> Self {
      Self(self.0.map(self::$ty::eotf))
    }

    pub fn oetf(self) -> Self {
      Self(self.0.map(self::$ty::oetf))
    }
  }

  pub mod $ty {
    pub fn eotf(x: $ty) -> $ty {
      match x {
        x if x > 0.0404482362771082 => ((x + 0.055) / 1.055).powf(2.4),
        x => x / 12.92,
      }
    }

    pub fn oetf(x: $ty) -> $ty {
      match x {
        x if x > 0.00313066844250063 => 1.055 * x.powf(1.0 / 2.4) - 0.055,
        x => 12.92 * x,
      }
    }
  }
});

impls!(f32);
impls!(f64);
