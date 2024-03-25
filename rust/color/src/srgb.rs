#![allow(clippy::transmute_num_to_bytes)]

use std::{mem, slice};

use super::convert;

pub use self::f32_to_srgb8::*;
pub use self::srgb8_to_f32::*;

mod f32_to_srgb8;
mod srgb8_to_f32;

pub mod transfer_fns;

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct sRGB<T, const N: usize>(pub [T; N]);

// ---

pub trait CastFrom<T> {
  fn cast_from(x: T) -> Self;
}

pub trait CastInto<T> {
  fn cast_into(self) -> T;
}

impl<T, U: CastFrom<T>> CastInto<U> for T {
  fn cast_into(self) -> U {
    U::cast_from(self)
  }
}

// sRGB<T, N> ←→ [T; N]

impl<T, const N: usize> From<[T; N]> for sRGB<T, N> {
  fn from(srgb: [T; N]) -> Self {
    Self(srgb)
  }
}

impl<T, const N: usize> From<sRGB<T, N>> for [T; N] {
  fn from(srgb: sRGB<T, N>) -> Self {
    srgb.0
  }
}

// sRGB<u8, 4> ←→ u32

impl From<u32> for sRGB<u8, 4> {
  fn from(srgb: u32) -> Self {
    unsafe { mem::transmute(srgb) }
  }
}

impl From<sRGB<u8, 4>> for u32 {
  fn from(srgb: sRGB<u8, 4>) -> Self {
    unsafe { mem::transmute(srgb) }
  }
}

// sRGB<u8, 3> ←→ u32

impl From<u32> for sRGB<u8, 3> {
  fn from(srgb: u32) -> Self {
    let [b0, b1, b2, _] = unsafe { mem::transmute(srgb) };
    Self([b0, b1, b2])
  }
}

impl From<sRGB<u8, 3>> for u32 {
  fn from(srgb: sRGB<u8, 3>) -> Self {
    let [b0, b1, b2] = srgb.0;
    unsafe { mem::transmute([b0, b1, b2, 0]) }
  }
}

// &sRGB<u8, 4> ←→ &u32
// &sRGB<u8, 4> ←→ &[u8; 4]
// &sRGB<u8, 3> ←→ &[u8; 3]
//
// &mut sRGB<u8, 4> ←→ &mut u32
// &mut sRGB<u8, 4> ←→ &mut [u8; 4]
// &mut sRGB<u8, 3> ←→ &mut [u8; 3]

macro_rules! impls {
  ($T:ty, $U:ty) => {
    impls!($T => $U);
    impls!($U => $T);
  };
  ($T:ty => $U:ty) => {
    impl<'a> CastFrom<&'a $T> for &'a $U {
      fn cast_from(srgb: &'a $T) -> Self {
        let ptr = (srgb as *const $T).cast();
        unsafe { &*ptr }
      }
    }

    impl<'a> CastFrom<&'a mut $T> for &'a mut $U {
      fn cast_from(srgb: &'a mut $T) -> Self {
        let ptr = (srgb as *mut $T).cast();
        unsafe { &mut *ptr }
      }
    }
  };
}

impls!(sRGB<u8, 4>, u32);
impls!(sRGB<u8, 4>, [u8; 4]);
impls!(sRGB<u8, 3>, [u8; 3]);

// &[sRGB<u8, 4>] ←→ &[u32]
// &[sRGB<u8, 4>] ←→ &[[u8; 4]]
// &[sRGB<u8, 3>] ←→ &[[u8; 3]]
//
// &mut [sRGB<u8, 4>] ←→ &mut [u32]
// &mut [sRGB<u8, 4>] ←→ &mut [[u8; 4]]
// &mut [sRGB<u8, 3>] ←→ &mut [[u8; 3]]

macro_rules! impls {
  ($T:ty, $U:ty) => {
    impls!($T => $U);
    impls!($U => $T);
  };
  ($T:ty => $U:ty) => {
    impl<'a> CastFrom<&'a [$T]> for &'a [$U] {
      fn cast_from(srgb: &'a [$T]) -> Self {
        let (ptr, len) = (srgb.as_ptr().cast(), srgb.len());
        unsafe { slice::from_raw_parts(ptr, len) }
      }
    }

    impl<'a> CastFrom<&'a mut [$T]> for &'a mut [$U] {
      fn cast_from(srgb: &'a mut [$T]) -> Self {
        let (ptr, len) = (srgb.as_mut_ptr().cast(), srgb.len());
        unsafe { slice::from_raw_parts_mut(ptr, len) }
      }
    }
  };
}

impls!(sRGB<u8, 4>, u32);
impls!(sRGB<u8, 4>, [u8; 4]);
impls!(sRGB<u8, 3>, [u8; 3]);

// sRGB<f32, N> ←→ sRGB<f64, N>
// sRGB<f32, N> ←→ sRGB<u8, N>
// sRGB<f64, N> ←→ sRGB<u8, N>

macro_rules! impls(($T:ident, $U:ident) => {
  impl<const N: usize> From<sRGB<$U, N>> for sRGB<$T, N> {
    fn from(srgb: sRGB<$U, N>) -> Self {
      sRGB(srgb.0.map(|x| x as $T))
    }
  }

  impl<const N: usize> From<sRGB<u8, N>> for sRGB<$T, N> {
    fn from(srgb: sRGB<u8, N>) -> Self {
      sRGB(srgb.0.map(convert::unorm8::$T))
    }
  }

  impl<const N: usize> From<sRGB<$T, N>> for sRGB<u8, N> {
    fn from(srgb: sRGB<$T, N>) -> Self {
      sRGB(srgb.0.map(convert::$T::unorm8))
    }
  }
});

impls!(f32, f64);
impls!(f64, f32);

// sRGB<f32, N> → oetf → sRGB<f32, N>
// sRGB<f32, N> ← eotf ← sRGB<f32, N>

macro_rules! impls(($T:ident) => {
  impl<const N: usize> sRGB<$T, N> {
    pub fn eotf(self) -> Self {
      Self(self.0.map(transfer_fns::$T::eotf))
    }

    pub fn oetf(self) -> Self {
      Self(self.0.map(transfer_fns::$T::oetf))
    }
  }
});

impls!(f32);
impls!(f64);

// sRGB<f32, N> → oetf → sRGB<u8, N>
// sRGB<f32, N> ← eotf ← sRGB<u8, N>

impl<const N: usize> sRGB<f32, N> {
  pub fn oetf_u8(self) -> sRGB<u8, N> {
    sRGB(self.0.map(f32_to_srgb8_v2))
  }
}

impl<const N: usize> sRGB<u8, N> {
  pub fn eotf_f32(self) -> sRGB<f32, N> {
    sRGB(self.0.map(srgb8_to_f32))
  }
}

// ---

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cast_one() {
    let a = &sRGB([4, 3, 2, 1]);
    let b = <&u32>::cast_from(a);
    let c = <&sRGB<u8, 4>>::cast_from(b);

    assert_eq!(b, &0x01020304);
    assert_eq!(a, c);
  }

  #[test]
  fn cast_many() {
    let a = &[sRGB([4, 3, 2, 1]), sRGB([8, 7, 6, 5])];
    let b = <&[u32]>::cast_from(a);
    let c = <&[sRGB<u8, 4>]>::cast_from(b);

    assert_eq!(b, &[0x01020304, 0x05060708]);
    assert_eq!(a, c);
  }
}
