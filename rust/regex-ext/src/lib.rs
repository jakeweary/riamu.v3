use std::fmt::{self, Formatter};
use std::result;

pub use regex::*;

pub type Result<T> = result::Result<T, Error>;

pub trait RegexExt {
  fn replace_all_fmt<F>(&self, fmt: &mut Formatter<'_>, input: &str, f: F) -> fmt::Result
  where
    F: FnMut(&mut Formatter<'_>, Captures<'_>) -> fmt::Result;
}

impl RegexExt for Regex {
  fn replace_all_fmt<F>(&self, fmt: &mut Formatter<'_>, input: &str, mut f: F) -> fmt::Result
  where
    F: FnMut(&mut Formatter<'_>, Captures<'_>) -> fmt::Result,
  {
    let mut last_match = 0;
    for c in self.captures_iter(input) {
      let m = c.get(0).unwrap();
      fmt.write_str(&input[last_match..m.start()])?;
      f(fmt, c)?;
      last_match = m.end();
    }
    fmt.write_str(&input[last_match..])?;
    Ok(())
  }
}
