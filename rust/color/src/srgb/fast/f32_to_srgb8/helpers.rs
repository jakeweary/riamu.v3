pub fn lerp(packed: u32, f: f32) -> u8 {
  // Unpack bias, scale
  let bias = (packed >> 16) << 9;
  let scale = packed & 0xffff;

  // Grab next-highest mantissa bits and perform linear interpolation
  let t = (f.to_bits() >> 12) & 0xff;
  ((bias + scale * t) >> 16) as u8
}

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
  match x {
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    x if !(x > min) => min, // written this way to catch NaNs
    x if x > max => max,
    x => x,
  }
}

pub fn u32x4_to_u8x4(x: [u32; 4]) -> [u8; 4] {
  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  if is_x86_feature_detected!("sse4.1") {
    return unsafe { u32x4_to_u8x4_sse41(x) };
  }

  x.map(|n| n as u8)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse4.1")]
pub unsafe fn u32x4_to_u8x4_sse41(x: [u32; 4]) -> [u8; 4] {
  #[cfg(target_arch = "x86")]
  use std::arch::x86 as simd;
  #[cfg(target_arch = "x86_64")]
  use std::arch::x86_64 as simd;
  use std::mem::transmute;

  let acc = transmute(x);
  let acc = simd::_mm_shuffle_epi8(acc, transmute(0x0c080400u128));
  let acc = simd::_mm_extract_epi32::<0>(acc);
  return transmute(acc);
}
