// https://gist.github.com/rygorous/2203834
// https://officedaytime.com/simd512e/

pub use v1::f32_to_srgb8 as f32_to_srgb8_v1;
pub use v2::f32_to_srgb8 as f32_to_srgb8_v2;

pub use v1::f32x4_to_srgb8x4 as f32x4_to_srgb8x4_v1;
pub use v2::f32x4_to_srgb8x4 as f32x4_to_srgb8x4_v2;

mod helpers;
mod v1;
mod v2;

#[test]
fn test() {
  for u in 0..=u8::MAX {
    let f = crate::srgb::srgb8_to_f32(u);
    assert_eq!(u, f32_to_srgb8_v1(f));
    assert_eq!(u, f32_to_srgb8_v2(f));
  }
}
