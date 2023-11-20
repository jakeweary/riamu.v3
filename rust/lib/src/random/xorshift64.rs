pub fn xorshift(seed: u64) -> impl FnMut() -> u64 {
  let mut n = seed;
  move || {
    n ^= n << 13;
    n ^= n >> 7;
    n ^= n << 17;
    n
  }
}

pub fn bytes(seed: u64) -> impl Iterator<Item = u8> {
  let mut next = xorshift(seed);
  std::iter::from_fn(move || Some(next())).flat_map(u64::to_ne_bytes)
}

pub fn f64(n: u64) -> f64 {
  f64::from_bits(0x3ff << 52 | n >> 12) - 1.0
}

pub fn f32(n: u32) -> f32 {
  f32::from_bits(0x3f8 << 20 | n >> 9) - 1.0
}
