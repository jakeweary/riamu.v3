//! An advanced implementation of generic cell rate algorithm,
//! significantly improved and adapted for modern use cases.
//!
//! Resources:
//! - Wiki: [Generic cell rate algorithm][1]
//! - PDF: [Traffic control and congestion control in B-ISDN, I.371][2] (page 87)
//! - PDF: [Traffic Management Specification, Version 4.0][3] (page 21)
//! - Article: [Rate Limiting, Cells, and GCRA][4]
//! - Article: [Understanding Generic Cell Rate Limiting][5]
//! - Article: [Implementing GCRA in Python][6]
//!
//! [1]: https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm
//! [2]: https://itu.int/rec/T-REC-I.371-200403-I
//! [3]: https://broadband-forum.org/download/af-tm-0056.000.pdf
//! [4]: https://brandur.org/rate-limiting
//! [5]: https://blog.ian.stapletoncordas.co/2018/12/understanding-generic-cell-rate-limiting
//! [6]: https://smarketshq.com/implementing-gcra-in-python-5df1f11aaa96

use std::time::{self, Duration, SystemTime};
use std::{ops::*, result};

pub use self::{hours as h, minutes as m, seconds as s};
pub use self::{micros as us, millis as ms, nanos as ns};

pub type Result = result::Result<(), Retry>;
pub type ResultAndInfo = (Result, Info);

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Copy)]
pub enum Retry {
  After(Duration),
  Never,
}

// ---

#[derive(Debug, Clone, Copy)]
pub struct Info {
  pub rate: Rate,
  pub reset: u64, // duration, nanoseconds
}

impl Info {
  pub fn ready(&self) -> Duration {
    Duration::from_nanos(self.reset.saturating_sub(self.rate.period))
  }

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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Default)]
pub struct State {
  pub tat: u64, // theoretical arrival time (unix time, nanoseconds)
}

impl State {
  pub fn info(&self, rate: Rate) -> Info {
    self.info_at(unix_time_ns(), rate)
  }

  pub fn scale(&mut self, old: Rate, new: Rate) -> Info {
    self.scale_at(unix_time_ns(), old, new, false)
  }

  pub fn saturating_scale(&mut self, old: Rate, new: Rate) -> Info {
    self.scale_at(unix_time_ns(), old, new, true)
  }

  pub fn update(&mut self, rate: Rate, amount: f64) -> ResultAndInfo {
    self.update_at(unix_time_ns(), rate, amount, false)
  }

  pub fn forced_update(&mut self, rate: Rate, amount: f64) -> Info {
    self.update_at(unix_time_ns(), rate, amount, true).1
  }

  // ---

  fn info_at(&self, t_arrived: u64, rate: Rate) -> Info {
    let reset = self.tat.saturating_sub(t_arrived);
    Info { rate, reset }
  }

  fn scale_at(&mut self, t_arrived: u64, old: Rate, new: Rate, saturate: bool) -> Info {
    // scales `tat` according to the difference in provided rates
    // has to be used when, e.g., a user buys premium subscription
    // (isn't a part of the original algorithm)

    let q = old.quota / new.quota;
    let p = new.period as f64 / old.period as f64;
    let scaled = q * p * self.tat.saturating_sub(t_arrived) as f64;
    let limit = if saturate { new.period } else { u64::MAX };
    self.tat = t_arrived + limit.min(scaled as u64);

    self.info_at(t_arrived, new)
  }

  fn update_at(&mut self, t_arrived: u64, rate: Rate, amount: f64, forced: bool) -> ResultAndInfo {
    let result = 'r: {
      let inc = rate.as_increment();

      // negative `amount` just moves `tat` backwards
      // (isn't a part of the original algorithm)
      if amount <= 0.0 {
        let dec_amount = (inc * -amount) as u64;
        self.tat = self.tat.saturating_sub(dec_amount);
        break 'r Ok(());
      }

      let inc_amount = (inc * amount) as u64;

      // non-conforming (`amount` is too big)
      // (isn't a part of the original algorithm)
      if inc_amount > rate.period && !forced {
        break 'r Err(Retry::Never);
      }

      let tat = self.tat.max(t_arrived) + inc_amount;
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

    let info = self.info_at(t_arrived, rate);
    (result, info)
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

fn unix_time_ns() -> u64 {
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
  const SATURATE: bool = true;

  #[test]
  fn basics() {
    let rate = Quota(2.0) / hours(1);
    let mut state = State::default();

    let (result, _) = state.update(rate, 1.0);
    assert!(result.is_ok());

    let (result, _) = state.update(rate, 1.0);
    assert!(result.is_ok());

    let (result, _) = state.update(rate, 1.0);
    assert!(result.is_err());
  }

  #[test]
  fn retry() {
    let rate = Quota(2.0) / ns(2);
    let mut state = State::default();

    let (result, _) = state.update_at(1, rate, 2.0, NORMAL);
    assert_eq!(result, Ok(()));

    let (result, _) = state.update_at(1, rate, 2.0, NORMAL);
    assert_eq!(result, Err(Retry::After(ns(2))));

    let (result, _) = state.update_at(1, rate, 1.0, NORMAL);
    assert_eq!(result, Err(Retry::After(ns(1))));

    let (result, _) = state.update_at(2, rate, 2.0, NORMAL);
    assert_eq!(result, Err(Retry::After(ns(1))));

    let (result, _) = state.update_at(2, rate, 1.0, NORMAL);
    assert_eq!(result, Ok(()));

    let (result, _) = state.update_at(2, rate, 3.0, NORMAL);
    assert_eq!(result, Err(Retry::Never));
  }

  #[test]
  fn back_and_forth() {
    let rate = Quota(5.0) / ns(5);
    let mut state = State::default();

    let (_, info) = state.update_at(1, rate, -1.0, NORMAL);
    assert_eq!(info.reset(), ns(0));

    let (_, info) = state.update_at(1, rate, 0.0, NORMAL);
    assert_eq!(info.reset(), ns(0));

    let (_, info) = state.update_at(1, rate, 1.0, NORMAL);
    assert_eq!(info.reset(), ns(1));

    let (_, info) = state.update_at(1, rate, 2.0, NORMAL);
    assert_eq!(info.reset(), ns(3));

    let (_, info) = state.update_at(1, rate, -1.0, NORMAL);
    assert_eq!(info.reset(), ns(2));

    let (_, info) = state.update_at(1, rate, -10.0, NORMAL);
    assert_eq!(info.reset(), ns(0));

    let (_, info) = state.update_at(1, rate, 1.0, NORMAL);
    assert_eq!(info.reset(), ns(1));
  }

  #[test]
  fn info() {
    let rate = Quota(4.0) / ns(4);
    let mut state = State::default();

    let (_, info) = state.update_at(1, rate, 1.0, NORMAL);
    assert_eq!((info.ratio(), info.used(), info.remaining()), (0.25, 1.0, 3.0));

    let (_, info) = state.update_at(1, rate, 1.0, NORMAL);
    assert_eq!((info.ratio(), info.used(), info.remaining()), (0.50, 2.0, 2.0));

    let (_, info) = state.update_at(1, rate, 1.0, NORMAL);
    assert_eq!((info.ratio(), info.used(), info.remaining()), (0.75, 3.0, 1.0));
  }

  #[test]
  fn floats() {
    let rate = Quota(1.0) / ns(1000);
    let mut state = State::default();

    let (result, info) = state.update_at(1, rate, 0.123, NORMAL);
    assert_eq!(result, Ok(()));
    assert_eq!(info.reset(), ns(123));
    assert_eq!((info.ratio(), info.used(), info.remaining()), (0.123, 0.123, 0.877));
  }

  #[test]
  fn forced() {
    let rate = Quota(5.0) / ns(5);
    let mut state = State::default();

    let (result, info) = state.update_at(1, rate, 100.0, FORCED);
    assert_eq!(result, Ok(()));
    assert_eq!(info.reset(), ns(100));
    assert_eq!((info.ratio(), info.used(), info.remaining()), (20.0, 100.0, -95.0));
  }

  #[test]
  fn scale() {
    let short = Quota(10.0) / seconds(1);
    let long = Quota(100.0) / minutes(1);
    let mut state = State::default();

    let (_, info) = state.update_at(1, short, 9.0, NORMAL);
    assert_eq!((info.used(), info.remaining()), (9.0, 1.0));

    state.scale_at(1, short, long, SATURATE);

    let (_, info) = state.update_at(1, long, 90.0, NORMAL);
    assert_eq!((info.used(), info.remaining()), (99.0, 1.0));

    state.scale_at(1, long, short, SATURATE);

    let (_, info) = state.update_at(1, short, 0.0, NORMAL);
    assert_eq!((info.used(), info.remaining()), (10.0, 0.0));
  }
}
