use std::fmt::Display;
use std::fmt::{self, Formatter};

pub struct Num(pub f64);

impl Display for Num {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let p = f.precision().unwrap_or(0);

    let p = if f.alternate() {
      let rounding_fix = match p {
        4 => f64::log10(9.9995),
        3 => f64::log10(9.995),
        2 => f64::log10(9.95),
        _ => 1.0,
      };
      let log10 = self.0.abs().log10() / rounding_fix;
      p.saturating_sub(1 + log10 as usize)
    } else {
      p
    };

    if self.0 < 0.0 {
      f.write_fmt(format_args!("\u{2212}{:.*}", p, self.0.abs()))
    } else {
      f.write_fmt(format_args!("{:.*}", p, self.0))
    }
  }
}

#[test]
fn tests() {
  assert_eq!(format!("{}", Num(-1.0)), "\u{2212}1");
  assert_eq!(format!("{}", Num(-0.6)), "\u{2212}1");
  // assert_eq!(format!("{}", Num(-0.5)), "0"); // FIXME
  // assert_eq!(format!("{}", Num(-0.4)), "0"); // FIXME
  // assert_eq!(format!("{}", Num(-0.0)), "0"); // FIXME
  assert_eq!(format!("{}", Num(0.0)), "0");
  assert_eq!(format!("{}", Num(1.0)), "1");

  assert_eq!(format!("{}", Num(1.1)), "1");
  assert_eq!(format!("{}", Num(11.1)), "11");
  assert_eq!(format!("{}", Num(111.1)), "111");

  assert_eq!(format!("{:.1}", Num(1.1)), "1.1");
  assert_eq!(format!("{:.1}", Num(11.1)), "11.1");
  assert_eq!(format!("{:.1}", Num(111.1)), "111.1");

  assert_eq!(format!("{:#.2}", Num(9.94)), "9.9");
  assert_eq!(format!("{:#.2}", Num(9.95)), "10");
  assert_eq!(format!("{:#.2}", Num(9.96)), "10");

  assert_eq!(format!("{:#.3}", Num(9.994)), "9.99");
  assert_eq!(format!("{:#.3}", Num(9.995)), "10.0");
  assert_eq!(format!("{:#.3}", Num(9.996)), "10.0");
}
