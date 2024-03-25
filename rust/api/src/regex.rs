use std::result;

pub use regex::*;

pub type Result<T> = result::Result<T, Error>;

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
