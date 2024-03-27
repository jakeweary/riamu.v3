use std::time::{self, Duration, SystemTime};
use std::{ptr, slice};

pub use xorshift32::*;
pub use xorshift64::*;

mod xorshift32;
mod xorshift64;

pub trait Random {
  fn from_time() -> Self;
  fn u32(&mut self) -> u32;
  fn u64(&mut self) -> u64;

  fn f32(&mut self) -> f32 {
    f32(self.u32())
  }

  fn f64(&mut self) -> f64 {
    f64(self.u64())
  }

  fn fill(&mut self, dst: &mut [u8]) {
    fill(dst, || self.u64().to_ne_bytes());
  }
}

fn f32(n: u32) -> f32 {
  f32::from_bits(0x3f8 << 20 | n >> 9) - 1.0
}

fn f64(n: u64) -> f64 {
  f64::from_bits(0x3ff << 52 | n >> 12) - 1.0
}

fn fill<const N: usize>(dst: &mut [u8], mut src: impl FnMut() -> [u8; N]) {
  let n = dst.len() / N;

  let (chunks, last) = dst.split_at_mut(n * N);
  let chunks_ptr = chunks.as_mut_ptr().cast();
  let chunks = unsafe { slice::from_raw_parts_mut(chunks_ptr, n) };

  for dst in chunks {
    *dst = src();
  }

  let src = src();
  let (src, dst, n) = (src.as_ptr(), last.as_mut_ptr(), last.len());
  unsafe { ptr::copy_nonoverlapping(src, dst, n) }
}

fn unix_time() -> Duration {
  let d = SystemTime::now().duration_since(time::UNIX_EPOCH);
  unsafe { d.unwrap_unchecked() }
}
