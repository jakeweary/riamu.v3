use std::borrow::Cow;

pub mod num;
pub mod plural;

pub fn ellipsis(text: &str, len: usize) -> Cow<'_, str> {
  if len == 0 {
    return Cow::Borrowed(&text[..0]);
  }
  let mut indices = text.char_indices().skip(len - 1);
  if let (Some((i, _)), Some(_)) = (indices.next(), indices.next()) {
    return Cow::Owned(format!("{}…", &text[..i]));
  }
  Cow::Borrowed(text)
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
    assert_eq!(ellipsis("", 0), "");
    assert_eq!(ellipsis("1", 0), "");

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
