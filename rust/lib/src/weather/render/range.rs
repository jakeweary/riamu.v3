use std::ops::BitAnd;

#[derive(Debug, Clone, Copy)]
pub struct Range {
  pub min: f64,
  pub max: f64,
}

impl Range {
  pub fn new(min: f64, max: f64) -> Self {
    Self { min, max }
  }

  pub fn of<T>(items: &[T], map: impl Fn(&T) -> f64) -> Self {
    let (min, max) = (f64::INFINITY, f64::NEG_INFINITY);
    items.iter().fold(Self { min, max }, |acc, item| acc & map(item))
  }

  pub fn normalize(self, value: f64) -> f64 {
    (value - self.min) / (self.max - self.min)
  }
}

impl BitAnd for Range {
  type Output = Self;

  fn bitand(self, rhs: Self) -> Self::Output {
    let min = self.min.min(rhs.min);
    let max = self.max.max(rhs.max);
    Self { min, max }
  }
}

impl BitAnd<f64> for Range {
  type Output = Self;

  fn bitand(self, rhs: f64) -> Self::Output {
    let min = self.min.min(rhs);
    let max = self.max.max(rhs);
    Self { min, max }
  }
}
