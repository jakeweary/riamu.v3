use std::fmt::{self, Formatter};
use std::result;

pub use regex::*;

pub type Result<T> = result::Result<T, Error>;

pub trait RegexExt {
  fn replace_fmt<F>(&self, fmt: &mut Formatter<'_>, input: &str, f: F) -> fmt::Result
  where
    F: FnMut(&mut Formatter<'_>, Match<'_>) -> fmt::Result;
}

impl RegexExt for Regex {
  fn replace_fmt<F>(&self, fmt: &mut Formatter<'_>, input: &str, mut f: F) -> fmt::Result
  where
    F: FnMut(&mut Formatter<'_>, Match<'_>) -> fmt::Result,
  {
    let mut last_match = 0;
    for c in self.captures_iter(input) {
      let m = unsafe { c.get(0).unwrap_unchecked() };
      fmt.write_str(unsafe { input.get_unchecked(last_match..m.start()) })?;
      f(fmt, m)?;
      last_match = m.end();
    }
    fmt.write_str(unsafe { input.get_unchecked(last_match..) })?;
    Ok(())
  }
}

pub fn matcher(pattern: Option<&str>, default: bool) -> Result<impl Fn(&str) -> bool> {
  let re = match pattern {
    Some(pat) => Some(RegexBuilder::new(pat).case_insensitive(true).build()?),
    None => None,
  };

  Ok(move |input: &str| match &re {
    Some(re) => re.is_match(input),
    None => default,
  })
}
