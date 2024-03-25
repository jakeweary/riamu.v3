use std::sync::OnceLock;

use regex::Regex;

pub mod catalog;
pub mod thread;

pub fn parse_url(url: &str) -> Option<(&str, &str, u64, Option<u64>)> {
  static RE: OnceLock<Regex> = OnceLock::new();

  let re = RE.get_or_init(|| {
    let re = r"(?i-u)https?://([\w.]+)/(\w+)/res/(\d+)\.html(?:#(\d+))?";
    Regex::new(re).unwrap()
  });

  let captures = re.captures(url)?;
  let domain = captures.get(1).map(|b| b.as_str())?;
  let board = captures.get(2).map(|b| b.as_str())?;
  let thread = captures.get(3).and_then(|t| t.as_str().parse().ok())?;
  let post = captures.get(4).and_then(|p| p.as_str().parse().ok());
  Some((domain, board, thread, post))
}
