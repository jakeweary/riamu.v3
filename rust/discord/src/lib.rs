use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;

pub mod colors;
pub mod link;

// TODO: should use this function in each place of the codebase
// where applicable
pub fn escape(input: &str) -> Cow<'_, str> {
  // FIXME: this regex sucks, should rethink it entirely
  static RE: LazyLock<Regex> = LazyLock::new(|| {
    let re = r"[\[\]()<>*_`]";
    Regex::new(re).unwrap()
  });

  RE.replace_all(input, r"\$0")
}
