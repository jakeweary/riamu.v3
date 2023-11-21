//! An advanced implementation of [Generic Cell Rate Algorithm][wiki]
//! from [Traffic Management Specification Version 4.0][pdf].
//!
//! Significantly improved and adapted for modern use cases.
//!
//! ```
//! State::info()         // get current info
//! State::update()       // try to spend 1 unit of quota
//! State::update_n(3.0)  // try to spend 3 units of quota
//! State::update_n(-2.0) // rollback 2 units of quota
//!
//! Ok(())                // ok
//! Err(Retry::After(_))  // rejected (retry after the provided amount)
//! Err(Retry::Never)     // rejected forever (requested too much quota)
//!
//! Info::reset()         // shows when `used()` will be 0 again
//! Info::used()          // shows the used quota
//! Info::remaining()     // shows the remaining quota
//! ```
//!
//! Useful resources: [pdf] [wiki], and some blog posts: [1] [2] [3].
//!
//! [pdf]: https://broadband-forum.org/download/af-tm-0056.000.pdf
//! [wiki]: https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm
//! [1]: https://brandur.org/rate-limiting
//! [2]: https://blog.ian.stapletoncordas.co/2018/12/understanding-generic-cell-rate-limiting
//! [3]: https://smarketshq.com/implementing-gcra-in-python-5df1f11aaa96

use std::time::{self, Duration, SystemTime};

pub type Result = std::result::Result<(), Retry>;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Retry {
  After(Duration),
  Never,
}

pub struct Info {
  pub result: Result,
  pub rate: Rate,
  pub reset: u64, // nanoseconds
}

impl Info {
  pub fn reset(&self) -> Duration {
    Duration::from_nanos(self.reset)
  }

  pub fn used(&self) -> f64 {
    let n = self.reset;
    n as f64 / self.rate.as_increment()
  }

  pub fn remaining(&self) -> f64 {
    let n = self.rate.period - self.reset;
    n as f64 / self.rate.as_increment()
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

  pub const fn per_second(n: f64) -> Self {
    Self::new(n, Duration::from_secs(1))
  }

  pub const fn per_minute(n: f64) -> Self {
    Self::new(n, Duration::from_secs(60))
  }

  pub const fn per_hour(n: f64) -> Self {
    Self::new(n, Duration::from_secs(60 * 60))
  }

  pub const fn per_day(n: f64) -> Self {
    Self::new(n, Duration::from_secs(60 * 60 * 24))
  }

  pub const fn per_week(n: f64) -> Self {
    Self::new(n, Duration::from_secs(60 * 60 * 24 * 7))
  }

  pub const fn per_month(n: f64) -> Self {
    Self::new(n, Duration::from_secs(60 * 60 * 24 * 30))
  }

  pub const fn per_year(n: f64) -> Self {
    Self::new(n, Duration::from_secs(60 * 60 * 24 * 365))
  }

  fn as_increment(&self) -> f64 {
    self.period as f64 / self.quota
  }
}

// ---

#[derive(Default)]
pub struct State {
  pub tat: u64,
}

impl State {
  pub fn info(&mut self, rate: Rate) -> Info {
    self.update_n(rate, 0.0)
  }

  pub fn update(&mut self, rate: Rate) -> Info {
    self.update_n(rate, 1.0)
  }

  pub fn update_n(&mut self, rate: Rate, n: f64) -> Info {
    self.update_n_at(rate, n, unix_epoch_ns())
  }

  fn update_n_at(&mut self, rate: Rate, n: f64, t_arrived: u64) -> Info {
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
      if inc_n > rate.period {
        break 'r Err(Retry::Never);
      }

      let tat = self.tat.max(t_arrived) + inc_n;
      let tat_threshold = t_arrived + rate.period;

      // non-conforming (rate limited)
      if tat > tat_threshold {
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

fn unix_epoch_ns() -> u64 {
  let now = SystemTime::now();
  let epoch = now.duration_since(time::UNIX_EPOCH);
  epoch.map_or(0, |d| d.as_nanos() as u64)
}

// ---

#[cfg(test)]
mod tests {
  use super::*;

  fn ns(ns: u64) -> Duration {
    Duration::from_nanos(ns)
  }

  #[test]
  fn basic_usage() {
    let rate = Rate::per_second(2.0);
    let mut state = State::default();

    assert!(state.update(rate).result.is_ok());
    assert!(state.update(rate).result.is_ok());
    assert!(state.update(rate).result.is_err());
  }

  #[test]
  fn retry() {
    let rate = Rate::new(2.0, ns(2));
    let mut state = State::default();

    assert_eq!(state.update_n_at(rate, 2.0, 1).result, Ok(()));
    assert_eq!(state.update_n_at(rate, 2.0, 1).result, Err(Retry::After(ns(2))));
    assert_eq!(state.update_n_at(rate, 1.0, 1).result, Err(Retry::After(ns(1))));
    assert_eq!(state.update_n_at(rate, 2.0, 2).result, Err(Retry::After(ns(1))));
    assert_eq!(state.update_n_at(rate, 1.0, 2).result, Ok(()));
    assert_eq!(state.update_n_at(rate, 3.0, 2).result, Err(Retry::Never));
  }

  #[test]
  fn increment_and_decrement() {
    let rate = Rate::new(5.0, ns(5));
    let mut state = State::default();

    assert_eq!(state.update_n_at(rate, -1.0, 1).reset(), ns(0));
    assert_eq!(state.update_n_at(rate, 0.0, 1).reset(), ns(0));
    assert_eq!(state.update_n_at(rate, 1.0, 1).reset(), ns(1));
    assert_eq!(state.update_n_at(rate, 2.0, 1).reset(), ns(3));
    assert_eq!(state.update_n_at(rate, -1.0, 1).reset(), ns(2));
    assert_eq!(state.update_n_at(rate, -10.0, 1).reset(), ns(0));
    assert_eq!(state.update_n_at(rate, 1.0, 1).reset(), ns(1));
  }

  #[test]
  fn used_and_remaining() {
    let rate = Rate::new(10.0, ns(10));

    let mut state = State::default();
    assert_eq!(state.update_n_at(rate, 1.0, 1).used(), 1.0);
    assert_eq!(state.update_n_at(rate, 1.0, 1).used(), 2.0);
    assert_eq!(state.update_n_at(rate, 1.0, 1).used(), 3.0);

    let mut state = State::default();
    assert_eq!(state.update_n_at(rate, 1.0, 1).remaining(), 9.0);
    assert_eq!(state.update_n_at(rate, 1.0, 1).remaining(), 8.0);
    assert_eq!(state.update_n_at(rate, 1.0, 1).remaining(), 7.0);
  }
}
