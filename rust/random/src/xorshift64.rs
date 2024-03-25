// https://en.wikipedia.org/wiki/Xorshift
// https://jstatsoft.org/v08/i14/paper

use rand_core::{Error, RngCore, SeedableRng};

use super::*;

pub fn xorshift64(mut n: u64) -> u64 {
  n ^= n << 13;
  n ^= n >> 7;
  n ^= n << 17;
  n
}

pub struct XorShift64(pub u64);

impl Random for XorShift64 {
  fn from_time() -> Self {
    Self(unix_time().as_nanos() as u64)
  }

  fn u64(&mut self) -> u64 {
    self.0 = xorshift64(self.0);
    self.0
  }

  fn u32(&mut self) -> u32 {
    self.u64() as u32
  }
}

impl SeedableRng for XorShift64 {
  type Seed = [u8; 8];

  fn seed_from_u64(seed: u64) -> Self {
    Self(seed)
  }

  fn from_seed(seed: Self::Seed) -> Self {
    Self(u64::from_ne_bytes(seed))
  }

  fn from_rng<R: RngCore>(mut rng: R) -> Result<Self, Error> {
    Ok(Self(rng.next_u64()))
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
