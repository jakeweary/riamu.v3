use std::fmt::{self, Display, Formatter, Write};

// ---

#[derive(Debug, Clone, Copy)]
pub struct Si {
  pub(super) short: char,
  pub(super) long: &'static str,
}

impl Display for Si {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if f.alternate() {
      f.write_str(self.long)
    } else {
      f.write_char(self.short)
    }
  }
}

// ---

#[derive(Debug, Clone, Copy)]
pub struct Iec {
  pub(super) short: char,
  pub(super) long: &'static str,
}

impl Display for Iec {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if f.alternate() {
      f.write_str(self.long)
    } else {
      f.write_char(self.short)?;
      f.write_char('i')
    }
  }
}

// ---

#[derive(Debug, Clone, Copy)]
pub struct K(pub(super) usize);

impl Display for K {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    (0..self.0).try_fold((), |_, _| f.write_char('k'))
  }
}

// ---

#[derive(Debug, Clone, Copy)]
pub struct Normalized {
  number: f64,
  precision: Option<usize>,
}

impl Display for Normalized {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let p = f.precision().or(self.precision).unwrap_or(0);
    write!(f, "{:.*}", p, self.number)
  }
}

// ---

#[derive(Debug, Clone, Copy)]
pub struct Prefix<P>(Option<P>);

impl<P> Display for Prefix<P>
where
  P: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(prefix) = &self.0 {
      prefix.fmt(f)?;
    }
    Ok(())
  }
}

// ---

#[derive(Debug, Clone, Copy)]
pub struct Formatted<P> {
  pub norm: Normalized,
  pub prefix: Prefix<P>,
  pub compact: bool,
}

impl<P> Formatted<P> {
  pub fn new(number: f64, precision: Option<usize>, prefix: Option<P>) -> Self {
    Self {
      norm: Normalized { number, precision },
      prefix: Prefix(prefix),
      compact: true,
    }
  }

  pub fn compact(mut self, compact: bool) -> Self {
    self.compact = compact;
    self
  }

  pub fn precision(mut self, digits: usize) -> Self {
    let p = digits.saturating_sub(1 + self.norm.number.log10() as usize);
    self.norm.precision = Some(p);
    self
  }
}

impl<P> Display for Formatted<P>
where
  P: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.norm.fmt(f)?;
    if !self.compact {
      f.write_char(' ')?;
    }
    self.prefix.fmt(f)
  }
}
