use std::borrow::Cow;
use std::sync::OnceLock;

use regex_lite::Regex;

pub mod colors;

// TODO: should use this function in each place of the codebase
// where applicable
//
// FIXME: this approach turned to be not so great
// there seems to be no escaping that works [in here](http://…)
// need to think what to do about it
//
pub fn escape(input: &str) -> Cow<'_, str> {
  static REGEX: OnceLock<Regex> = OnceLock::new();
  REGEX
    .get_or_init(|| Regex::new(r"[\[\]()<>*_`]").unwrap())
    .replace_all(input, r"\$0")
}

#[test]
fn tests() {
  assert_eq!(escape("()[]<>*_`"), r"\(\)\[\]\<\>\*\_\`");
}
