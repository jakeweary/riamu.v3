// https://en.wikipedia.org/wiki/Xorshift
// https://jstatsoft.org/v08/i14/paper

use std::{ptr, slice};

pub struct XorShift64 {
  n: u64,
}

impl XorShift64 {
  pub fn new(seed: u64) -> Self {
    Self { n: seed }
  }

  pub fn u64(&mut self) -> u64 {
    self.n ^= self.n << 13;
    self.n ^= self.n >> 7;
    self.n ^= self.n << 17;
    self.n
  }

  pub fn u32(&mut self) -> u32 {
    self.u64() as u32
  }

  pub fn f64(&mut self) -> f64 {
    let n = self.u64();
    f64::from_bits(0x3ff << 52 | n >> 12) - 1.0
  }

  pub fn f32(&mut self) -> f32 {
    let n = self.u64() as u32;
    f32::from_bits(0x3f8 << 20 | n >> 9) - 1.0
  }

  pub fn fill(&mut self, dst: &mut [u8]) {
    let n = dst.len() / 8;

    let (chunks, last) = dst.split_at_mut(n * 8);
    let chunks_ptr = chunks.as_mut_ptr().cast();
    let chunks = unsafe { slice::from_raw_parts_mut(chunks_ptr, n) };

    for dst in chunks {
      let src = self.u64().to_ne_bytes();
      *dst = src;
    }

    let src = self.u64().to_ne_bytes();
    let (src, dst, n) = (src.as_ptr(), last.as_mut_ptr(), last.len());
    unsafe { ptr::copy_nonoverlapping(src, dst, n) }
  }
}

impl rand_core::SeedableRng for XorShift64 {
  type Seed = [u8; 8];

  fn from_seed(seed: Self::Seed) -> Self {
    Self::new(u64::from_ne_bytes(seed))
  }

  fn seed_from_u64(seed: u64) -> Self {
    Self::new(seed)
  }
}

impl rand_core::RngCore for XorShift64 {
  fn next_u32(&mut self) -> u32 {
    self.u32()
  }

  fn next_u64(&mut self) -> u64 {
    self.u64()
  }

  fn fill_bytes(&mut self, dst: &mut [u8]) {
    self.fill(dst);
  }

  fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), rand_core::Error> {
    self.fill(dst);
    Ok(())
  }
}
