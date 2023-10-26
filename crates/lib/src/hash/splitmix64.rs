// https://nullprogram.com/blog/2018/07/31/
// https://xoshiro.di.unimi.it/splitmix64.c

pub fn mix(mut x: u64) -> u64 {
  x = (x ^ x >> 30).wrapping_mul(0xbf58476d1ce4e5b9);
  x = (x ^ x >> 27).wrapping_mul(0x94d049bb133111eb);
  x ^ x >> 31
}

pub fn unmix(mut x: u64) -> u64 {
  x = (x ^ x >> 31 ^ x >> 62).wrapping_mul(0x319642b2d24d8ec3);
  x = (x ^ x >> 27 ^ x >> 54).wrapping_mul(0x96de1b173f119089);
  x ^ x >> 30 ^ x >> 60
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mix_unmix() {
    for n in (0..64).map(|n| 1 << n) {
      assert_eq!(unmix(mix(n)), n);
    }
  }

  #[test]
  fn correctness() {
    // https://rosettacode.org/wiki/Pseudo-random_numbers/Splitmix64

    let mut state: u64 = 1234567;
    let mut next = || {
      state = state.wrapping_add(0x9e3779b97f4a7c15);
      mix(state)
    };

    assert_eq!(next(), 6457827717110365317);
    assert_eq!(next(), 3203168211198807973);
    assert_eq!(next(), 9817491932198370423);
    assert_eq!(next(), 4593380528125082431);
    assert_eq!(next(), 16408922859458223821);
  }
}
