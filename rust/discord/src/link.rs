use std::fmt::{self, Display, Formatter};
use std::sync::OnceLock;

use regex::Regex;
use regex_ext::RegexExt;

pub struct Link<'a>(pub &'a str, pub &'a str);
pub struct Embed<'a>(pub &'a str, pub &'a str);
pub struct Name<'a>(pub &'a str);
pub struct Url<'a>(pub &'a str);

impl<'a> Display for Link<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let Self(name, url) = *self;
    link_fmt(f, name, url, ["[", "](<", ">)"])
  }
}

impl<'a> Display for Embed<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let Self(name, url) = *self;
    link_fmt(f, name, url, ["[", "](", ")"])
  }
}

impl<'a> Display for Name<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(?i)https?://|\[|\]").unwrap());

    // TODO: there should be a better way than this
    // would be nice to keep actual brackets if they don't break formatting
    // but that requires to understand exact conditions when it breaks
    re.replace_all_fmt(f, self.0, |f, caps| match &caps[0] {
      "[" => f.write_str("\u{298b}"),
      "]" => f.write_str("\u{298c}"),
      _ => Ok(()),
    })
  }
}

impl<'a> Display for Url<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\(|\)").unwrap());

    re.replace_all_fmt(f, self.0, |f, caps| match &caps[0] {
      "(" => f.write_str("%28"),
      ")" => f.write_str("%29"),
      _ => Ok(()),
    })
  }
}

fn link_fmt(f: &mut Formatter<'_>, name: &str, url: &str, parts: [&str; 3]) -> fmt::Result {
  let [p0, p1, p2] = parts;
  f.write_str(p0)?;
  Name(name).fmt(f)?;
  f.write_str(p1)?;
  Url(url).fmt(f)?;
  f.write_str(p2)
}
