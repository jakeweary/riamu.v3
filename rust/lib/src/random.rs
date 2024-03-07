pub mod weighted;
pub mod xorshift64;

pub fn f64(n: u64) -> f64 {
  f64::from_bits(0x3ff << 52 | n >> 12) - 1.0
}

pub fn f32(n: u32) -> f32 {
  f32::from_bits(0x3f8 << 20 | n >> 9) - 1.0
}

pub fn fill<const N: usize>(dst: &mut [u8], mut src: impl FnMut() -> [u8; N]) {
  let n = dst.len() / N;

  let (chunks, last) = dst.split_at_mut(n * N);
  let chunks_ptr = chunks.as_mut_ptr().cast();
  let chunks = unsafe { std::slice::from_raw_parts_mut(chunks_ptr, n) };

  for dst in chunks {
    *dst = src();
  }

  let src = src();
  let (src, dst, n) = (src.as_ptr(), last.as_mut_ptr(), last.len());
  unsafe { std::ptr::copy_nonoverlapping(src, dst, n) }
}
