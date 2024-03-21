// https://en.wikipedia.org/wiki/Xorshift
// https://jstatsoft.org/v08/i14/paper

use rand_core::{Error, RngCore, SeedableRng};

use super::*;

pub fn xorshift32(mut n: u32) -> u32 {
  n ^= n << 13;
  n ^= n >> 17;
  n ^= n << 5;
  n
}

pub struct XorShift32(pub u32);

impl Random for XorShift32 {
  fn from_time() -> Self {
    Self(unix_time().as_micros() as u32)
  }

  fn u32(&mut self) -> u32 {
    self.0 = xorshift32(self.0);
    self.0
  }

  fn u64(&mut self) -> u64 {
    let hi = self.u32() as u64;
    let lo = self.u32() as u64;
    lo | hi << 32
  }
}

impl SeedableRng for XorShift32 {
  type Seed = [u8; 4];

  fn seed_from_u64(seed: u64) -> Self {
    Self(seed as u32)
  }

  fn from_seed(seed: Self::Seed) -> Self {
    Self(u32::from_ne_bytes(seed))
  }

  fn from_rng<R: RngCore>(mut rng: R) -> Result<Self, Error> {
    Ok(Self(rng.next_u32()))
  }
}

impl RngCore for XorShift32 {
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
