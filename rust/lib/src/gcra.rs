// a loose implementation of Generic Cell Rate Algorithm
// from Traffic Management Specification Version 4.0
// slightly improved and adapted for modern use cases
//
// useful resources:
// https://broadband-forum.org/download/af-tm-0056.000.pdf
// https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm
//
// and some blog posts:
// https://brandur.org/rate-limiting
// https://blog.ian.stapletoncordas.co/2018/12/understanding-generic-cell-rate-limiting
// https://smarketshq.com/implementing-gcra-in-python-5df1f11aaa96

use std::time::{self, Duration, SystemTime};

pub type Result = std::result::Result<(), Retry>;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Retry {
  After(Duration),
  Never,
}

// --

pub struct Info {
  pub result: Result,
  pub rate: Rate,
  pub reset: u64, // ns
}

impl Info {
  pub fn reset(&self) -> Duration {
    Duration::from_nanos(self.reset)
  }

  pub fn used(&self) -> f64 {
    let used = self.reset;
    used as f64 / self.rate.as_increment()
  }

  pub fn free(&self) -> f64 {
    let free = self.rate.limit - self.reset;
    free as f64 / self.rate.as_increment()
  }
}

// ---

#[derive(Clone, Copy)]
pub struct Rate {
  pub quota: f64,
  pub limit: u64, // ns
}

impl Rate {
  pub const fn new(quota: f64, period: Duration) -> Self {
    let limit = period.as_nanos() as u64;
    Self { quota, limit }
  }

  pub const fn per_second(quota: f64) -> Self {
    Self::new(quota, Duration::from_secs(1))
  }

  pub const fn per_minute(quota: f64) -> Self {
    Self::new(quota, Duration::from_secs(60))
  }

  pub const fn per_hour(quota: f64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60))
  }

  pub const fn per_day(quota: f64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60 * 24))
  }

  pub const fn per_week(quota: f64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60 * 24 * 7))
  }

  pub const fn per_month(quota: f64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60 * 24 * 30))
  }

  pub const fn per_year(quota: f64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60 * 24 * 365))
  }

  fn as_increment(&self) -> f64 {
    self.limit as f64 / self.quota
  }
}

// ---

#[derive(Default)]
pub struct State {
  pub tat: u64,
}

impl State {
  pub fn update(&mut self, rate: Rate) -> Info {
    self.update_n(rate, 1.0)
  }

  pub fn update_n(&mut self, rate: Rate, n: f64) -> Info {
    self.update_n_at(rate, n, unix_epoch_ns())
  }

  fn update_n_at(&mut self, rate: Rate, n: f64, t_arrived: u64) -> Info {
    let result = 'r: {
      let inc = rate.as_increment();
      if n < 0.0 {
        let dec_n = (inc * -n) as u64;
        self.tat = self.tat.saturating_sub(dec_n);
        break 'r Ok(());
      }

      let inc_n = (inc * n) as u64;
      if inc_n > rate.limit {
        break 'r Err(Retry::Never);
      }

      let tat = self.tat.max(t_arrived) + inc_n;
      let tat_threshold = t_arrived + rate.limit;
      if tat > tat_threshold {
        let after = Duration::from_nanos(tat - tat_threshold);
        break 'r Err(Retry::After(after));
      }

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
  fn used_and_free() {
    let rate = Rate::new(10.0, ns(10));

    let mut state = State::default();
    assert_eq!(state.update_n_at(rate, 1.0, 1).used(), 1.0);
    assert_eq!(state.update_n_at(rate, 1.0, 1).used(), 2.0);
    assert_eq!(state.update_n_at(rate, 1.0, 1).used(), 3.0);

    let mut state = State::default();
    assert_eq!(state.update_n_at(rate, 1.0, 1).free(), 9.0);
    assert_eq!(state.update_n_at(rate, 1.0, 1).free(), 8.0);
    assert_eq!(state.update_n_at(rate, 1.0, 1).free(), 7.0);
  }
}
