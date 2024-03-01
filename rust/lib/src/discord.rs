use std::borrow::Cow;
use std::sync::OnceLock;

use regex::Regex;

pub mod colors;
pub mod link;

// TODO: should use this function in each place of the codebase
// where applicable
pub fn escape(input: &str) -> Cow<'_, str> {
  // FIXME: this regex sucks, should rethink it entirely
  static RE: OnceLock<Regex> = OnceLock::new();
  RE.get_or_init(|| Regex::new(r"[\[\]()<>*_`]").unwrap())
    .replace_all(input, r"\$0")
}
