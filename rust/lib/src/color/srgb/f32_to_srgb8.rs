// https://gist.github.com/rygorous/2203834
// https://officedaytime.com/simd512e/

pub use v1::f32_to_srgb8 as f32_to_srgb8_v1;
pub use v2::f32_to_srgb8 as f32_to_srgb8_v2;

pub use v1::f32x4_to_srgb8x4 as f32x4_to_srgb8x4_v1;
pub use v2::f32x4_to_srgb8x4 as f32x4_to_srgb8x4_v2;

mod v1;
mod v2;

fn lerp(packed: u32, f: f32) -> u8 {
  // Unpack bias, scale
  let bias = (packed >> 16) << 9;
  let scale = packed & 0xffff;

  // Grab next-highest mantissa bits and perform linear interpolation
  let t = (f.to_bits() >> 12) & 0xff;
  ((bias + scale * t) >> 16) as u8
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
  match x {
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    x if !(x > min) => min, // written this way to catch NaNs
    x if x > max => max,
    x => x,
  }
}

#[test]
fn test() {
  for u in 0..=u8::MAX {
    let f = super::srgb8_to_f32(u);
    assert_eq!(u, f32_to_srgb8_v1(f));
    assert_eq!(u, f32_to_srgb8_v2(f));
  }
}
