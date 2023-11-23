//! An advanced implementation of [Generic Cell Rate Algorithm][wiki]
//! from [Traffic Management Specification Version 4.0][pdf].
//!
//! Significantly improved and adapted for modern use cases.
//!
//! Useful resources: [pdf] [wiki], and some blog posts: [1] [2] [3].
//!
//! [pdf]: https://broadband-forum.org/download/af-tm-0056.000.pdf
//! [wiki]: https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm
//! [1]: https://brandur.org/rate-limiting
//! [2]: https://blog.ian.stapletoncordas.co/2018/12/understanding-generic-cell-rate-limiting
//! [3]: https://smarketshq.com/implementing-gcra-in-python-5df1f11aaa96

use std::ops::Div;
use std::time::{self, Duration, SystemTime};

pub use self::{hours as h, minutes as m, seconds as s};
pub use self::{micros as us, millis as ms, nanos as ns};

pub type Result = std::result::Result<(), Retry>;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Retry {
  After(Duration),
  Never,
}

// ---

pub struct Info {
  pub result: Result,
  pub rate: Rate,
  pub reset: u64, // nanoseconds
}

impl Info {
  pub fn reset(&self) -> Duration {
    Duration::from_nanos(self.reset)
  }

  pub fn ratio(&self) -> f64 {
    self.reset as f64 / self.rate.period as f64
  }

  pub fn used(&self) -> f64 {
    self.reset as f64 / self.rate.as_increment()
  }

  pub fn remaining(&self) -> f64 {
    (self.rate.period as f64 - self.reset as f64) / self.rate.as_increment()
  }
}

// ---

// here comes a slight deviation from the original thing:
// GCRA is defined as I (increment) and L (limit)
// but here quota and period are used instead
//
// I = period / quota
// L = period

#[derive(Clone, Copy)]
pub struct Rate {
  pub quota: f64,  // some abstract units
  pub period: u64, // nanoseconds
}

impl Rate {
  pub const fn new(quota: f64, period: Duration) -> Self {
    let period = period.as_nanos() as u64;
    Self { quota, period }
  }

  fn as_increment(&self) -> f64 {
    self.period as f64 / self.quota
  }
}

// ---

pub struct Quota(pub f64);

impl Div<Duration> for Quota {
  type Output = Rate;

  fn div(self, period: Duration) -> Self::Output {
    Rate::new(self.0, period)
  }
}

// ---

#[derive(Default)]
pub struct State {
  pub tat: u64,
}

impl State {
  pub fn scale(&mut self, old: Rate, new: Rate) {
    self.scale_at(old, new, unix_epoch_ns())
  }

  pub fn update(&mut self, rate: Rate, n: f64) -> Info {
    self.update_at(rate, n, unix_epoch_ns(), false)
  }

  pub fn forced_update(&mut self, rate: Rate, n: f64) -> Info {
    self.update_at(rate, n, unix_epoch_ns(), true)
  }

  fn scale_at(&mut self, old: Rate, new: Rate, t_arrived: u64) {
    // scales `tat` according to the difference in provided rates
    // has to be used when, e.g., user buys premium subscription
    // (isn't a part of the original algorithm)

    let q = old.quota / new.quota;
    let p = new.period as f64 / old.period as f64;
    let scaled = q * p * self.tat.saturating_sub(t_arrived) as f64;
    self.tat = t_arrived + new.period.min(scaled as u64);
  }

  #[inline(never)]
  fn update_at(&mut self, rate: Rate, n: f64, t_arrived: u64, forced: bool) -> Info {
    let result = 'r: {
      let inc = rate.as_increment();

      // zero `n` is used to to get current info,
      // negative `n` is used to move `tat` backwards
      // (isn't a part of the original algorithm)
      if n <= 0.0 {
        let dec_n = (inc * -n) as u64;
        self.tat = self.tat.saturating_sub(dec_n);
        break 'r Ok(());
      }

      let inc_n = (inc * n) as u64;

      // non-conforming (`n` is too big)
      // (isn't really a part of the original algorithm)
      if inc_n > rate.period && !forced {
        break 'r Err(Retry::Never);
      }

      let tat = self.tat.max(t_arrived) + inc_n;
      let tat_threshold = t_arrived + rate.period;

      // non-conforming (rate limited)
      if tat > tat_threshold && !forced {
        let after = Duration::from_nanos(tat - tat_threshold);
        break 'r Err(Retry::After(after));
      }

      // conforming
      self.tat = tat;
      Ok(())
    };

    let reset = self.tat.saturating_sub(t_arrived);
    Info { result, rate, reset }
  }
}

// ---

macro_rules! periods(($($f:ident => $g:ident * $s:expr,)+) => {
  $(pub const fn $f(n: u64) -> Duration { Duration::$g(n * $s) })+
});

periods! {
  nanos   => from_nanos  * 1,
  micros  => from_micros * 1,
  millis  => from_millis * 1,
  seconds => from_secs   * 1,
  minutes => from_secs   * 60,
  hours   => from_secs   * 60 * 60,
  days    => from_secs   * 60 * 60 * 24,
  weeks   => from_secs   * 60 * 60 * 24 * 7,
  months  => from_secs   * 60 * 60 * 24 * 30,
  years   => from_secs   * 60 * 60 * 24 * 365,
}

#[inline(never)]
fn unix_epoch_ns() -> u64 {
  let now = SystemTime::now();
  let epoch = now.duration_since(time::UNIX_EPOCH);
  epoch.map_or(0, |d| d.as_nanos() as u64)
}

// ---

#[cfg(test)]
mod tests {
  use super::*;

  const NORMAL: bool = false;
  const FORCED: bool = true;

  #[test]
  fn basics() {
    let rate = Quota(2.0) / hours(1);
    let mut state = State::default();

    assert!(state.update(rate, 1.0).result.is_ok());
    assert!(state.update(rate, 1.0).result.is_ok());
    assert!(state.update(rate, 1.0).result.is_err());
  }

  #[test]
  fn retry() {
    let rate = Quota(2.0) / ns(2);
    let mut state = State::default();

    assert_eq!(state.update_at(rate, 2.0, 1, NORMAL).result, Ok(()));
    assert_eq!(state.update_at(rate, 2.0, 1, NORMAL).result, Err(Retry::After(ns(2))));
    assert_eq!(state.update_at(rate, 1.0, 1, NORMAL).result, Err(Retry::After(ns(1))));
    assert_eq!(state.update_at(rate, 2.0, 2, NORMAL).result, Err(Retry::After(ns(1))));
    assert_eq!(state.update_at(rate, 1.0, 2, NORMAL).result, Ok(()));
    assert_eq!(state.update_at(rate, 3.0, 2, NORMAL).result, Err(Retry::Never));
  }

  #[test]
  fn back_and_forth() {
    let rate = Quota(5.0) / ns(5);
    let mut state = State::default();

    assert_eq!(state.update_at(rate, -1.0, 1, NORMAL).reset(), ns(0));
    assert_eq!(state.update_at(rate, 0.0, 1, NORMAL).reset(), ns(0));
    assert_eq!(state.update_at(rate, 1.0, 1, NORMAL).reset(), ns(1));
    assert_eq!(state.update_at(rate, 2.0, 1, NORMAL).reset(), ns(3));
    assert_eq!(state.update_at(rate, -1.0, 1, NORMAL).reset(), ns(2));
    assert_eq!(state.update_at(rate, -10.0, 1, NORMAL).reset(), ns(0));
    assert_eq!(state.update_at(rate, 1.0, 1, NORMAL).reset(), ns(1));
  }

  #[test]
  fn info() {
    let rate = Quota(4.0) / ns(4);
    let mut state = State::default();

    let info = state.update_at(rate, 1.0, 1, NORMAL);
    assert_eq!((info.ratio(), info.used(), info.remaining()), (0.25, 1.0, 3.0));

    let info = state.update_at(rate, 1.0, 1, NORMAL);
    assert_eq!((info.ratio(), info.used(), info.remaining()), (0.50, 2.0, 2.0));

    let info = state.update_at(rate, 1.0, 1, NORMAL);
    assert_eq!((info.ratio(), info.used(), info.remaining()), (0.75, 3.0, 1.0));
  }

  #[test]
  fn floats() {
    let rate = Quota(1.0) / ns(1000);
    let mut state = State::default();

    let info = state.update_at(rate, 0.123, 1, NORMAL);
    assert_eq!(info.result, Ok(()));
    assert_eq!(info.reset(), ns(123));
    assert_eq!((info.ratio(), info.used(), info.remaining()), (0.123, 0.123, 0.877));
  }

  #[test]
  fn forced() {
    let rate = Quota(5.0) / ns(5);
    let mut state = State::default();

    let info = state.update_at(rate, 100.0, 1, FORCED);
    assert_eq!(info.result, Ok(()));
    assert_eq!(info.reset(), ns(100));
    assert_eq!((info.ratio(), info.used(), info.remaining()), (20.0, 100.0, -95.0));
  }

  #[test]
  fn scale() {
    let short = Quota(10.0) / seconds(1);
    let long = Quota(100.0) / minutes(1);
    let mut state = State::default();

    let info = state.update_at(short, 9.0, 1, NORMAL);
    assert_eq!((info.used(), info.remaining()), (9.0, 1.0));

    state.scale_at(short, long, 1);

    let info = state.update_at(long, 90.0, 1, NORMAL);
    assert_eq!((info.used(), info.remaining()), (99.0, 1.0));

    state.scale_at(long, short, 1);

    let info = state.update_at(short, 0.0, 1, NORMAL);
    assert_eq!((info.used(), info.remaining()), (10.0, 0.0));
  }
}
