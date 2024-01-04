// https://en.wikipedia.org/wiki/Xorshift
// https://jstatsoft.org/v08/i14/paper

use rand_core::{Error, RngCore, SeedableRng};

use super::*;

pub struct XorShift64(pub u64);

impl XorShift64 {
  pub fn u64(&mut self) -> u64 {
    self.0 ^= self.0 << 13;
    self.0 ^= self.0 >> 7;
    self.0 ^= self.0 << 17;
    self.0
  }

  pub fn u32(&mut self) -> u32 {
    self.u64() as u32
  }

  pub fn f64(&mut self) -> f64 {
    f64(self.u64())
  }

  pub fn f32(&mut self) -> f32 {
    f32(self.u64() as u32)
  }

  pub fn fill(&mut self, dst: &mut [u8]) {
    fill(dst, || self.u64().to_ne_bytes());
  }
}

impl SeedableRng for XorShift64 {
  type Seed = [u8; 8];

  fn from_seed(seed: Self::Seed) -> Self {
    Self(u64::from_ne_bytes(seed))
  }

  fn seed_from_u64(seed: u64) -> Self {
    Self(seed)
  }
}

impl RngCore for XorShift64 {
  fn next_u32(&mut self) -> u32 {
    self.u32()
  }

  fn next_u64(&mut self) -> u64 {
    self.u64()
  }

  fn fill_bytes(&mut self, dst: &mut [u8]) {
    self.fill(dst);
  }

  fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Error> {
    self.fill(dst);
    Ok(())
  }
}
