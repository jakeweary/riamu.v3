use super::*;

pub fn f32_to_srgb8(x: f32) -> u8 {
  let almost_one = f32::from_bits(0x3f7fffff); // 1-eps
  let lut_thresh = f32::from_bits(0x3b800000); // 2^(-8)
  let linear_sc = f32::from_bits(0x454c5d00);
  let float_to_int = f32::from_bits((127 + 23) << 23);

  // Clamp to [0, 1-eps]; these two values map to 0 and 1, respectively.
  // The tests are carefully written so that NaNs map to 0, same as in the reference
  // implementation.
  let f = clamp(x, 0.0, almost_one);

  // Check which region this value falls into
  if f < lut_thresh {
    // linear region
    // use "magic value" to get float->int with rounding. (float_to_int)
    f.mul_add(linear_sc, float_to_int).to_bits() as u8
  } else {
    // non-linear region
    let i = (f.to_bits() >> 20) % 64;
    let packed = unsafe { *LUT.get_unchecked(i as usize) };
    lerp(packed, f)
  }
}

pub fn f32x4_to_srgb8x4(x: [f32; 4]) -> [u8; 4] {
  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  if is_x86_feature_detected!("sse4.1") {
    return unsafe { f32x4_to_srgb8x4_sse41(x) };
  }

  x.map(f32_to_srgb8)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse4.1")]
unsafe fn f32x4_to_srgb8x4_sse41(x: [f32; 4]) -> [u8; 4] {
  #[cfg(target_arch = "x86")]
  use std::arch::x86 as simd;
  #[cfg(target_arch = "x86_64")]
  use std::arch::x86_64 as simd;
  use std::mem::transmute;

  // Initial clamp
  let acc = transmute(x);
  let acc = simd::_mm_max_ps(acc, transmute([0x00000000; 4]));
  let acc = simd::_mm_min_ps(acc, transmute([0x3f7fffff; 4]));

  // Table index
  let indices = simd::_mm_srli_epi32(transmute(acc), 20);
  let indices = simd::_mm_and_si128(indices, transmute([63; 4]));
  let indices = transmute::<_, [u32; 4]>(indices);

  // Table lookup
  let values = indices.map(|i| *LUT.get_unchecked(i as usize));

  // Linear part of ramp
  let acc0 = simd::_mm_mul_ps(acc, transmute([0x454c5d00; 4]));
  let acc0 = simd::_mm_cvtps_epi32(acc0);

  // Table finisher
  let acc1 = simd::_mm_srli_epi32(transmute(acc), 12);
  let acc1 = simd::_mm_and_si128(acc1, transmute([0x000000ff; 4]));
  let acc1 = simd::_mm_or_si128(acc1, transmute([0x02000000; 4]));
  let acc1 = simd::_mm_madd_epi16(acc1, transmute(values));
  let acc1 = simd::_mm_srli_epi32(acc1, 16);

  // Combine linear+table
  let mask = simd::_mm_cmplt_ps(acc, transmute([0x3b800000; 4]));
  let acc0 = simd::_mm_and_si128(acc0, transmute(mask));
  let acc1 = simd::_mm_andnot_si128(transmute(mask), acc1);
  let acc = simd::_mm_or_si128(acc0, acc1);

  // Extract 4 bytes
  let acc = simd::_mm_shuffle_epi8(acc, transmute(0x0c080400_u128));
  let acc = simd::_mm_extract_epi32::<0>(acc);
  return transmute(acc);
}

#[rustfmt::skip]
const LUT: [u32; 64] = [
  0x0b0f01cb, 0x0bf401ae, 0x0ccb0195, 0x0d950180, 0x0e56016e, 0x0f0d015e, 0x0fbc0150, 0x10630143,
  0x11070264, 0x1238023e, 0x1357021d, 0x14660201, 0x156601e9, 0x165a01d3, 0x174401c0, 0x182401af,
  0x18fe0331, 0x1a9602fe, 0x1c1502d2, 0x1d7e02ad, 0x1ed4028d, 0x201a0270, 0x21520256, 0x227d0240,
  0x239f0443, 0x25c003fe, 0x27bf03c4, 0x29a10392, 0x2b6a0367, 0x2d1d0341, 0x2ebe031f, 0x304d0300,
  0x31d105b0, 0x34a80555, 0x37520507, 0x39d504c5, 0x3c37048b, 0x3e7c0458, 0x40a8042a, 0x42bd0401,
  0x44c20798, 0x488e071e, 0x4c1c06b6, 0x4f76065d, 0x52a50610, 0x55ac05cc, 0x5892058f, 0x5b590559,
  0x5e0c0a23, 0x631c0980, 0x67db08f6, 0x6c55087f, 0x70940818, 0x74a007bd, 0x787d076c, 0x7c330723,
  0x06970158, 0x07420142, 0x07e30130, 0x087b0120, 0x090b0112, 0x09940106, 0x0a1700fc, 0x0a9500f2,
];
