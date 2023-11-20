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

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Info {
  pub result: Result<(), Retry>,
  pub reset: Duration,
  pub used: f64,
  pub free: f64,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Retry {
  After(Duration),
  Never,
}

// ---

#[derive(Clone, Copy)]
pub struct Rate {
  pub quota: f64,
  pub limit: u64,
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
    let inc = rate.as_increment();
    let inc_n = (inc * n) as u64;

    let tat = self.tat.max(t_arrived) + inc_n;
    let tat_threshold = t_arrived + rate.limit;

    let result = if inc_n > rate.limit {
      Err(Retry::Never)
    } else if tat > tat_threshold {
      Err(Retry::After(Duration::from_nanos(tat - tat_threshold)))
    } else {
      self.tat = tat;
      Ok(())
    };

    let tat = self.tat.max(t_arrived);
    let used = tat - t_arrived;
    let free = tat_threshold - tat;

    Info {
      result,
      reset: Duration::from_nanos(used),
      used: used as f64 / inc,
      free: free as f64 / inc,
    }
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
  fn time_is_freezed() {
    let rate = Rate::new(2.0, ns(2));
    let mut state = State::default();

    assert_eq! {
      state.update_n_at(rate, 1.0, 0),
      Info { result: Ok(()), reset: ns(1), used: 1.0, free: 1.0 },
    }

    assert_eq! {
      state.update_n_at(rate, 1.0, 0),
      Info { result: Ok(()), reset: ns(2), used: 2.0, free: 0.0 },
    }

    assert_eq! {
      state.update_n_at(rate, 1.0, 0),
      Info { result: Err(Retry::After(ns(1))), reset: ns(2), used: 2.0, free: 0.0 },
    }
  }

  #[test]
  fn time_is_moving_forward() {
    let rate = Rate::new(1.0, ns(4));
    let mut state = State::default();

    assert_eq! {
      state.update_n_at(rate, 1.0, 0),
      Info { result: Ok(()), reset: ns(4), used: 1.0, free: 0.0 },
    }

    assert_eq! {
      state.update_n_at(rate, 1.0, 1),
      Info { result: Err(Retry::After(ns(3))), reset: ns(3), used: 0.75, free: 0.25 },
    }

    assert_eq! {
      state.update_n_at(rate, 1.0, 4),
      Info { result: Ok(()), reset: ns(4), used: 1.0, free: 0.0 },
    }
  }

  #[test]
  fn retry_never() {
    let rate = Rate::new(1.0, ns(1));
    let mut state = State::default();

    assert_eq! {
      state.update_n_at(rate, 2.0, 0),
      Info { result: Err(Retry::Never), reset: ns(0), used: 0.0, free: 1.0 },
    }
  }

  #[test]
  fn rounding_and_precision() {
    let rate = Rate::new(1.0, ns(1));
    let mut state = State::default();

    assert_eq! {
      state.update_n_at(rate, 0.9, 0), // 0.9 rounded down to 0
      Info { result: Ok(()), reset: ns(0), used: 0.0, free: 1.0 },
    }

    assert_eq! {
      state.update_n_at(rate, 1.9, 0), // 1.9 rounded down to 1
      Info { result: Ok(()), reset: ns(1), used: 1.0, free: 0.0 },
    }
  }
}
