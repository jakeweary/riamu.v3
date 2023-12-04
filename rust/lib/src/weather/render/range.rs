use std::ops::BitAnd;

#[derive(Debug, Clone, Copy)]
pub struct Range {
  pub min: f64,
  pub max: f64,
}

impl Range {
  pub fn of<T>(items: &[T], map: impl Fn(&T) -> f64) -> Self {
    let (min, max) = (f64::INFINITY, f64::NEG_INFINITY);
    items.iter().fold(Self { min, max }, |acc, item| acc & map(item))
  }

  pub fn round(self) -> Self {
    let min = (0.5 + self.min).floor();
    let max = (0.5 + self.max).floor();
    Self { min, max }
  }

  pub fn round_n_abs(self, n: f64) -> Self {
    let min = n * (self.min / n).floor();
    let max = n * (self.max / n).ceil();
    Self { min, max }
  }

  pub fn round_n_rel(self, n: f64) -> Self {
    let range = n * ((self.max - self.min) / n).ceil();
    let min = 0.5 * (self.min + self.max - range);
    let max = 0.5 * (self.min + self.max + range);
    Self { min, max }
  }

  pub fn lerp(self, t: f64) -> f64 {
    self.min + t * (self.max - self.min)
  }

  pub fn unlerp(self, x: f64) -> f64 {
    (x - self.min) / (self.max - self.min)
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
