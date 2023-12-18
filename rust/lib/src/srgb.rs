#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct sRGB<T, const N: usize>([T; N]);

// sRGB<u8, 3> ←→ u32

impl From<u32> for sRGB<u8, 3> {
  fn from(argb: u32) -> Self {
    let [b, g, r, _] = argb.to_le_bytes();
    Self([r, g, b])
  }
}

impl From<sRGB<u8, 3>> for u32 {
  fn from(srgb: sRGB<u8, 3>) -> Self {
    let [r, g, b] = srgb.0;
    u32::from_le_bytes([b, g, r, 0])
  }
}

// sRGB<u8, 4> ←→ u32

impl From<u32> for sRGB<u8, 4> {
  fn from(argb: u32) -> Self {
    Self(argb.to_be_bytes())
  }
}

impl From<sRGB<u8, 4>> for u32 {
  fn from(srgb: sRGB<u8, 4>) -> Self {
    u32::from_be_bytes(srgb.0)
  }
}

// sRGB<T, N> ←→ [T; N]

impl<T, const N: usize> From<[T; N]> for sRGB<T, N> {
  fn from(argb: [T; N]) -> Self {
    Self(argb)
  }
}

impl<T, const N: usize> From<sRGB<T, N>> for [T; N] {
  fn from(srgb: sRGB<T, N>) -> Self {
    srgb.0
  }
}

// sRGB<u8, N> ←→ sRGB<N, f32>
// sRGB<u8, N> ←→ sRGB<N, f64>
// and transfer functions

macro_rules! impls(($T:ident) => {
  impl<const N: usize> From<sRGB<$T, N>> for sRGB<u8, N> {
    fn from(srgb: sRGB<$T, N>) -> Self {
      sRGB(srgb.0.map(|x| (x * 0x100 as $T) as u8))
    }
  }

  impl<const N: usize> From<sRGB<u8, N>> for sRGB<$T, N> {
    fn from(srgb: sRGB<u8, N>) -> Self {
      sRGB(srgb.0.map(|x| x as $T / 0xff as $T))
    }
  }

  impl<const N: usize> sRGB<$T, N> {
    pub fn eotf(self) -> Self {
      Self(self.0.map(self::$T::eotf))
    }

    pub fn oetf(self) -> Self {
      Self(self.0.map(self::$T::oetf))
    }
  }

  pub mod $T {
    #![allow(clippy::excessive_precision)]

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

impls!(f32);
impls!(f64);
