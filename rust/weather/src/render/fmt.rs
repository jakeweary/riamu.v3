use std::fmt::Display;
use std::fmt::{self, Formatter};

pub struct Num(pub f64);

impl Display for Num {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    // let num = self.0;
    // let digits = f.precision().unwrap_or(0);
    // let (num, prec) = if f.alternate() {
    //   let prec = digits.saturating_sub(1 + num.abs().log10() as usize);
    //   let num = round(num, prec as i32);
    //   let prec = digits.saturating_sub(1 + num.abs().log10() as usize);
    //   (num, prec)
    // } else {
    //   let num = round(num, digits as i32);
    //   (num, digits)
    // };
    // let sign = if num < 0.0 { "\u{2212}" } else { "" };
    // write!(f, "{}{:.*}", sign, prec, num.abs())

    let prec = f.precision().unwrap_or(0);
    let prec = if f.alternate() {
      let num = self.0.abs();
      let exp = prec as i32 - num.log10().max(0.0) as i32;
      let log10 = (num + 5.0 / f64::powi(10.0, exp)).log10();
      prec.saturating_sub(1 + log10 as usize)
    } else {
      prec
    };
    let num = round(self.0, prec as i32);
    let sign = if num < 0.0 { "\u{2212}" } else { "" };
    write!(f, "{}{:.*}", sign, prec, num.abs())
  }
}

fn round(num: f64, precision: i32) -> f64 {
  let exp = f64::powi(10.0, precision);
  (num * exp).round() / exp
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_fn() {
    assert_eq!(round(-5.5555, 3), -5.556);
    assert_eq!(round(-5.5555, 2), -5.56);
    assert_eq!(round(-5.5555, 1), -5.6);
    assert_eq!(round(-5.5555, 0), -6.0);
    assert_eq!(round(5.5555, 0), 6.0);
    assert_eq!(round(5.5555, 1), 5.6);
    assert_eq!(round(5.5555, 2), 5.56);
    assert_eq!(round(5.5555, 3), 5.556);
  }

  #[test]
  fn sanity_check() {
    assert_eq!(format!("{}", Num(-1.0)), "\u{2212}1");
    assert_eq!(format!("{}", Num(-0.0)), "0");
    assert_eq!(format!("{}", Num(0.0)), "0");
    assert_eq!(format!("{}", Num(1.0)), "1");

    assert_eq!(format!("{:#}", Num(-1.0)), "\u{2212}1");
    assert_eq!(format!("{:#}", Num(-0.0)), "0");
    assert_eq!(format!("{:#}", Num(0.0)), "0");
    assert_eq!(format!("{:#}", Num(1.0)), "1");
  }

  #[test]
  fn minus_sign() {
    assert_eq!(format!("{}", Num(-0.6)), "\u{2212}1");
    assert_eq!(format!("{}", Num(-0.5)), "\u{2212}1");
    assert_eq!(format!("{}", Num(-0.4)), "0");

    assert_eq!(format!("{:.1}", Num(-0.06)), "\u{2212}0.1");
    assert_eq!(format!("{:.1}", Num(-0.05)), "\u{2212}0.1");
    assert_eq!(format!("{:.1}", Num(-0.04)), "0.0");

    assert_eq!(format!("{:.2}", Num(-0.006)), "\u{2212}0.01");
    assert_eq!(format!("{:.2}", Num(-0.005)), "\u{2212}0.01");
    assert_eq!(format!("{:.2}", Num(-0.004)), "0.00");
  }

  #[test]
  fn alternate_rounding() {
    assert_eq!(format!("{:#.1}", Num(9.9)), "10");
    assert_eq!(format!("{:#.2}", Num(99.9)), "100");
    assert_eq!(format!("{:#.3}", Num(999.9)), "1000");
    assert_eq!(format!("{:#.4}", Num(9999.9)), "10000");
    assert_eq!(format!("{:#.5}", Num(99999.9)), "100000");

    assert_eq!(format!("{:#.1}", Num(9.9)), "10");
    assert_eq!(format!("{:#.2}", Num(9.99)), "10");
    assert_eq!(format!("{:#.3}", Num(9.999)), "10.0");
    assert_eq!(format!("{:#.4}", Num(9.9999)), "10.00");
    assert_eq!(format!("{:#.5}", Num(9.99999)), "10.000");

    assert_eq!(format!("{:#.3}", Num(9.994)), "9.99");
    assert_eq!(format!("{:#.3}", Num(9.995)), "10.0");
    assert_eq!(format!("{:#.3}", Num(9.996)), "10.0");

    assert_eq!(format!("{:#.4}", Num(99.994)), "99.99");
    assert_eq!(format!("{:#.4}", Num(99.995)), "100.0");
    assert_eq!(format!("{:#.4}", Num(99.996)), "100.0");

    assert_eq!(format!("{:#.5}", Num(999.994)), "999.99");
    assert_eq!(format!("{:#.5}", Num(999.995)), "1000.0");
    assert_eq!(format!("{:#.5}", Num(999.996)), "1000.0");
  }
}
