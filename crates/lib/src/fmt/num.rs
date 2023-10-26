pub use structs::*;
pub use traits::*;

pub mod structs;
pub mod tables;
pub mod traits;

pub fn si(num: f64) -> Formatted<Si> {
  let power = if num != 0.0 { (num.log10() / 3.0).floor() } else { 0.0 };
  let norm = num / 1000f64.powf(power);
  let precision = Some(2 - norm.log10() as usize);
  let prefix = match power as isize {
    0 => None,
    p => Some(tables::SI[(p + 11) as usize * 10 / 11]),
  };
  Formatted::new(norm, precision, prefix)
}

pub fn iec(num: f64) -> Formatted<Iec> {
  let power = (1.024 * num).log2() / 10.0;
  match power as usize {
    0 => Formatted::new(num, None, None),
    p => {
      let norm = num / 1024f64.powf(power.floor());
      let precision = 2 - norm.log10() as usize;
      let prefix = tables::IEC[p - 1];
      Formatted::new(norm, Some(precision), Some(prefix))
    }
  }
}

pub fn k(num: f64) -> Formatted<K> {
  let power = num.log10() / 3.0;
  match power as usize {
    0 => Formatted::new(num, None, None),
    p => {
      let norm = num / 1000f64.powf(power.floor());
      let precision = 2 - norm.log10() as usize;
      Formatted::new(norm, Some(precision), Some(K(p)))
    }
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;

  #[test]
  fn si() {
    assert_eq!(format!("{}g", 0.0.si()), "0.00g");
    assert_eq!(format!("{}g", 1.0.si()), "1.00g");
    assert_eq!(format!("{}g", 1e-3.si()), "1.00mg");
    assert_eq!(format!("{}g", 1e-2.si()), "10.0mg");
    assert_eq!(format!("{}g", 1e-1.si()), "100mg");
    assert_eq!(format!("{}g", 1e+1.si()), "10.0g");
    assert_eq!(format!("{}g", 1e+2.si()), "100g");
    assert_eq!(format!("{}g", 1e+3.si()), "1.00kg");
    assert_eq!(format!("{}g", 1e+4.si()), "10.0kg");
    assert_eq!(format!("{}g", 1e+5.si()), "100kg");
    assert_eq!(format!("{}g", 1e+6.si()), "1.00Mg");
    assert_eq!(format!("{}g", 1e+7.si()), "10.0Mg");
  }

  #[test]
  fn iec() {
    assert_eq!(format!("{}B", 0.0.iec()), "0B");
    assert_eq!(format!("{}B", 1.0.iec()), "1B");
    assert_eq!(format!("{}B", 1e-3.iec()), "0B");
    assert_eq!(format!("{}B", 1e-2.iec()), "0B");
    assert_eq!(format!("{}B", 1e-1.iec()), "0B");
    assert_eq!(format!("{}B", 1e+1.iec()), "10B");
    assert_eq!(format!("{}B", 1e+2.iec()), "100B");
    assert_eq!(format!("{}B", 1e+3.iec()), "0.98KiB");
    assert_eq!(format!("{}B", 1e+4.iec()), "9.77KiB");
    assert_eq!(format!("{}B", 1e+5.iec()), "97.7KiB");
    assert_eq!(format!("{}B", 1e+6.iec()), "977KiB");
    assert_eq!(format!("{}B", 1e+7.iec()), "9.54MiB");
  }

  #[test]
  fn k() {
    assert_eq!(format!("{}", 0.0.k()), "0");
    assert_eq!(format!("{}", 1.0.k()), "1");
    assert_eq!(format!("{}", 1e-3.k()), "0");
    assert_eq!(format!("{}", 1e-2.k()), "0");
    assert_eq!(format!("{}", 1e-1.k()), "0");
    assert_eq!(format!("{}", 1e+1.k()), "10");
    assert_eq!(format!("{}", 1e+2.k()), "100");
    assert_eq!(format!("{}", 1e+3.k()), "1.00k");
    assert_eq!(format!("{}", 1e+4.k()), "10.0k");
    assert_eq!(format!("{}", 1e+5.k()), "100k");
    assert_eq!(format!("{}", 1e+6.k()), "1.00kk");
    assert_eq!(format!("{}", 1e+7.k()), "10.0kk");
  }

  #[test]
  fn advanced_usage() {
    let f = 1e10.iec().compact(false);

    assert_eq!(format!("{:.6}B", f), "9.313226 GiB");
    assert_eq!(format!("{:#.6}bytes", f), "9.313226 gibibytes");
    assert_eq!(format!("{} {:#}bytes", f.norm, f.prefix), "9.31 gibibytes");

    assert_eq!(format!("{}B", f.precision(0)), "9 GiB");
    assert_eq!(format!("{}B", f.precision(1)), "9 GiB");
    assert_eq!(format!("{}B", f.precision(2)), "9.3 GiB");
    assert_eq!(format!("{}B", f.precision(3)), "9.31 GiB");
    assert_eq!(format!("{}B", f.precision(4)), "9.313 GiB");
    assert_eq!(format!("{}B", f.precision(5)), "9.3132 GiB");
  }
}
