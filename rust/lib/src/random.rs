// https://en.wikipedia.org/wiki/Xorshift
// https://jstatsoft.org/v08/i14/paper

pub struct XorShift64 {
  pub n: u64,
}

impl rand_core::SeedableRng for XorShift64 {
  type Seed = [u8; 8];

  fn from_seed(seed: Self::Seed) -> Self {
    let n = u64::from_ne_bytes(seed);
    Self { n }
  }
}

impl rand_core::RngCore for XorShift64 {
  fn next_u32(&mut self) -> u32 {
    self.next_u64() as u32
  }

  fn next_u64(&mut self) -> u64 {
    self.n ^= self.n << 13;
    self.n ^= self.n >> 7;
    self.n ^= self.n << 17;
    self.n
  }

  fn fill_bytes(&mut self, dest: &mut [u8]) {
    let len = dest.len() / 8;

    let (chunks, last) = dest.split_at_mut(len * 8);
    let chunks_ptr = chunks.as_mut_ptr().cast();
    let chunks = unsafe { std::slice::from_raw_parts_mut(chunks_ptr, len) };

    for dst in chunks {
      let src = self.next_u64().to_ne_bytes();
      *dst = src;
    }

    let src = self.next_u64().to_ne_bytes();
    let (src, dst, n) = (src.as_ptr(), last.as_mut_ptr(), last.len());
    unsafe { std::ptr::copy_nonoverlapping(src, dst, n) }
  }

  fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
    self.fill_bytes(dest);
    Ok(())
  }
}
