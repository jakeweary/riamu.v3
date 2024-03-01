use std::fmt::{self, Display, Formatter, Write};
use std::sync::OnceLock;

use regex::Regex;

use crate::regex::RegexExt;

pub struct Link<'a>(pub &'a str, pub &'a str);
pub struct LinkEmbed<'a>(pub &'a str, pub &'a str);
pub struct LinkName<'a>(pub &'a str);
pub struct LinkUrl<'a>(pub &'a str);

impl<'a> Display for Link<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let Self(name, url) = *self;
    f.write_char('[')?;
    LinkName(name).fmt(f)?;
    f.write_str("](<")?;
    LinkUrl(url).fmt(f)?;
    f.write_str(">)")
  }
}

impl<'a> Display for LinkEmbed<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let Self(name, url) = *self;
    f.write_char('[')?;
    LinkName(name).fmt(f)?;
    f.write_str("](")?;
    LinkUrl(url).fmt(f)?;
    f.write_str(")")
  }
}

impl<'a> Display for LinkName<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(?i)https?://|\[|\]").unwrap());

    // TODO: there should be a better way than simply removing [ and ]
    re.replace_fmt(f, self.0, |_, _| Ok(()))
  }
}

impl<'a> Display for LinkUrl<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\(|\)").unwrap());

    re.replace_fmt(f, self.0, |f, m| match m.as_str() {
      "(" => f.write_str("%28"),
      ")" => f.write_str("%29"),
      _ => Ok(()),
    })
  }
}
