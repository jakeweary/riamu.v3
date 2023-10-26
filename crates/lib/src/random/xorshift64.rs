#[rustfmt::skip]
pub fn xorshift(seed: Option<u64>) -> impl FnMut() -> u64 {
  let mut n = seed.unwrap_or(1);
  move || { n ^= n << 13; n ^= n >> 7; n ^= n << 17; n }
}

pub fn random_bytes(seed: Option<u64>) -> impl Iterator<Item = u8> {
  let mut next = xorshift(seed);
  std::iter::from_fn(move || Some(next())).flat_map(u64::to_ne_bytes)
}
