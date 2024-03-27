use std::borrow::Cow;
use std::{fmt, result};

pub mod num;
pub mod plural;

pub type Result<T> = result::Result<T, fmt::Error>;

pub fn ellipsis(text: &str, max_len: usize) -> Cow<'_, str> {
  let mut indices = text.char_indices().skip(max_len - 1);
  match (indices.next(), indices.next()) {
    (Some((i, _)), Some(_)) => Cow::Owned(format!("{}…", &text[..i])),
    _ => Cow::Borrowed(text),
  }
}

pub fn line_ellipsis(text: &str, max_lines: usize) -> Cow<'_, str> {
  let newlines = text.bytes().enumerate().filter(|&(_, b)| b == b'\n');
  let mut indices = newlines.map(|(i, _)| i).skip(max_lines - 2);
  match (indices.next(), indices.next()) {
    (Some(i), Some(_)) => Cow::Owned(format!("{}\n…", &text[..i])),
    _ => Cow::Borrowed(text),
  }
}

pub fn duration(s: u64) -> String {
  match split_dhms(s) {
    [0, 0, m, s] => format! {               "{m}:{s:02}" },
    [0, h, m, s] => format! {        "{h}:{m:02}:{s:02}" },
    [d, h, m, s] => format! { "{d}:{h:02}:{m:02}:{s:02}" },
  }
}

pub fn dhms(s: u64) -> String {
  match split_dhms(s) {
    [0, 0, 0, s] => format! {     "{s}s" },
    [0, 0, m, s] => format! { "{m}m{s}s" },
    [0, h, m, _] => format! { "{h}h{m}m" },
    [d, h, _, _] => format! { "{d}d{h}h" },
  }
}

fn split_dhms(s: u64) -> [u64; 4] {
  let (m, s) = (s / 60, s % 60);
  let (h, m) = (m / 60, m % 60);
  let (d, h) = (h / 24, h % 24);
  [d, h, m, s]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_duration_dhms() {
    assert_eq!(dhms(1), "1s");
    assert_eq!(dhms(60), "1m0s");
    assert_eq!(dhms(60 * 60), "1h0m");
    assert_eq!(dhms(60 * 60 * 24), "1d0h");
  }

  #[test]
  fn test_duration() {
    assert_eq!(duration(1), "0:01");
    assert_eq!(duration(60), "1:00");
    assert_eq!(duration(60 * 60), "1:00:00");
    assert_eq!(duration(60 * 60 * 24), "1:00:00:00");
  }

  #[test]
  fn test_ellipsis() {
    assert_eq!(ellipsis("", 1), "");
    assert_eq!(ellipsis("1", 1), "1");
    assert_eq!(ellipsis("12", 1), "…");

    assert_eq!(ellipsis("1", 2), "1");
    assert_eq!(ellipsis("12", 2), "12");
    assert_eq!(ellipsis("123", 2), "1…");

    assert_eq!(ellipsis("12345", 1), "…");
    assert_eq!(ellipsis("12345", 2), "1…");
    assert_eq!(ellipsis("12345", 3), "12…");
    assert_eq!(ellipsis("12345", 4), "123…");
    assert_eq!(ellipsis("12345", 5), "12345");
    assert_eq!(ellipsis("12345", 6), "12345");
  }
}
