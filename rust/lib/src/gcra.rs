// Slightly improved version of Generic Cell Rate Algorithm
// from Traffic Management Specification Version 4.0
//
// https://broadband-forum.org/download/af-tm-0056.000.pdf
// https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm
//
// https://brandur.org/rate-limiting
// https://blog.ian.stapletoncordas.co/2018/12/understanding-generic-cell-rate-limiting
// https://smarketshq.com/implementing-gcra-in-python-5df1f11aaa96

use std::time::{self, Duration, SystemTime};

#[derive(Debug)]
pub struct Info {
  pub result: Result<(), Retry>,
  pub reset: u64,
  pub used: f64,
  pub free: f64,
}

#[derive(Debug)]
pub enum Retry {
  After(u64),
  Never,
}

// ---

#[derive(Debug, Clone, Copy)]
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

  fn increment(&self) -> f64 {
    self.limit as f64 / self.quota
  }
}

// ---

#[derive(Debug, Default)]
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
    let inc = rate.increment();
    let inc_n = (inc * n) as u64;

    let tat = self.tat.max(t_arrived) + inc_n;
    let tat_threshold = t_arrived + rate.limit;

    let result = if inc_n > rate.limit {
      Err(Retry::Never)
    } else if tat > tat_threshold {
      Err(Retry::After(tat - tat_threshold))
    } else {
      Ok(self.tat = tat)
    };

    let tat = self.tat.max(t_arrived);
    let used = tat - t_arrived;
    let free = tat_threshold - tat;

    Info {
      result,
      reset: used,
      used: used as f64 / inc,
      free: free as f64 / inc,
    }
  }
}

fn unix_epoch_ns() -> u64 {
  let now = SystemTime::now();
  let epoch = now.duration_since(time::UNIX_EPOCH).unwrap();
  epoch.as_nanos() as u64
}
