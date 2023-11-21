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
pub struct Info {
  pub result: Result,
  pub reset: Duration,
  pub used: u64,
  pub free: u64,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Retry {
  After(Duration),
  Never,
}

// ---

#[derive(Clone, Copy)]
pub struct Rate {
  pub quota: u64,
  pub limit: u64,
}

impl Rate {
  pub const fn new(quota: u64, period: Duration) -> Self {
    let limit = period.as_nanos() as u64;
    Self { quota, limit }
  }

  pub const fn per_second(quota: u64) -> Self {
    Self::new(quota, Duration::from_secs(1))
  }

  pub const fn per_minute(quota: u64) -> Self {
    Self::new(quota, Duration::from_secs(60))
  }

  pub const fn per_hour(quota: u64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60))
  }

  pub const fn per_day(quota: u64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60 * 24))
  }

  pub const fn per_week(quota: u64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60 * 24 * 7))
  }

  pub const fn per_month(quota: u64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60 * 24 * 30))
  }

  pub const fn per_year(quota: u64) -> Self {
    Self::new(quota, Duration::from_secs(60 * 60 * 24 * 365))
  }
}

// ---

#[derive(Default)]
pub struct State {
  pub tat: u64,
}

impl State {
  pub fn update(&mut self, rate: Rate) -> Info {
    self.update_n(rate, 1)
  }

  pub fn update_n(&mut self, rate: Rate, n: i64) -> Info {
    self.update_n_at(rate, n, unix_epoch_ns())
  }

  pub fn update_n_at(&mut self, rate: Rate, n: i64, t_arrived: u64) -> Info {
    let inc = div_floor(n.unsigned_abs() * rate.limit, rate.quota);

    let result = 'r: {
      if n < 0 {
        self.tat = self.tat.saturating_sub(inc);
        break 'r Ok(());
      }

      if inc > rate.limit {
        break 'r Err(Retry::Never);
      }

      let tat = self.tat.max(t_arrived) + inc;
      let tat_threshold = t_arrived + rate.limit;

      if tat > tat_threshold {
        let after = Duration::from_nanos(tat - tat_threshold);
        break 'r Err(Retry::After(after));
      }

      self.tat = tat;
      Ok(())
    };

    let delta = self.tat.saturating_sub(t_arrived);
    let reset = Duration::from_nanos(delta);
    let used = div_ceil(delta * rate.quota, rate.limit);
    let free = rate.quota - used;

    Info {
      result,
      reset,
      used,
      free,
    }
  }
}

// https://stackoverflow.com/a/14878734/8802501
fn div_ceil(num: u64, den: u64) -> u64 {
  match den {
    0 => unsafe { std::hint::unreachable_unchecked() },
    _ => num / den + (num % den != 0) as u64,
  }
}

fn div_floor(num: u64, den: u64) -> u64 {
  match den {
    0 => unsafe { std::hint::unreachable_unchecked() },
    _ => num / den,
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
    let rate = Rate::per_second(2);
    let mut state = State::default();

    assert!(state.update(rate).result.is_ok());
    assert!(state.update(rate).result.is_ok());
    assert!(state.update(rate).result.is_err());
  }

  #[test]
  fn time_is_freezed() {
    let rate = Rate::new(2, ns(2));
    let mut state = State::default();

    assert_eq! {
      state.update_n_at(rate, 1, 1),
      Info { result: Ok(()), reset: ns(1), used: 1, free: 1 },
    }

    assert_eq! {
      state.update_n_at(rate, 1, 1),
      Info { result: Ok(()), reset: ns(2), used: 2, free: 0 },
    }

    assert_eq! {
      state.update_n_at(rate, 1, 1),
      Info { result: Err(Retry::After(ns(1))), reset: ns(2), used: 2, free: 0 },
    }
  }

  #[test]
  fn time_is_moving_forward() {
    let rate = Rate::new(1, ns(4));
    let mut state = State::default();

    assert_eq! {
      state.update_n_at(rate, 1, 0),
      Info { result: Ok(()), reset: ns(4), used: 1, free: 0 },
    }

    assert_eq! {
      state.update_n_at(rate, 1, 1),
      Info { result: Err(Retry::After(ns(3))), reset: ns(3), used: 1, free: 0 },
    }

    assert_eq! {
      state.update_n_at(rate, 1, 4),
      Info { result: Ok(()), reset: ns(4), used: 1, free: 0 },
    }
  }

  #[test]
  fn negative_amounts() {
    let rate = Rate::new(4, ns(4));
    let mut state = State::default();

    assert_eq! {
      state.update_n_at(rate, 3, 1),
      Info { result: Ok(()), reset: ns(3), used: 3, free: 1 },
    }

    assert_eq! {
      state.update_n_at(rate, -2, 1),
      Info { result: Ok(()), reset: ns(1), used: 1, free: 3 },
    }

    assert_eq! {
      state.update_n_at(rate, 2, 1),
      Info { result: Ok(()), reset: ns(3), used: 3, free: 1 },
    }

    assert_eq! {
      state.update_n_at(rate, -10, 1),
      Info { result: Ok(()), reset: ns(0), used: 0, free: 4 },
    }

    assert_eq! {
      state.update_n_at(rate, 1, 1),
      Info { result: Ok(()), reset: ns(1), used: 1, free: 3 },
    }
  }

  #[test]
  fn retry_never() {
    let rate = Rate::new(1, ns(1));
    let mut state = State::default();

    assert_eq! {
      state.update_n_at(rate, 2, 1),
      Info { result: Err(Retry::Never), reset: ns(0), used: 0, free: 1 },
    }
  }
}
