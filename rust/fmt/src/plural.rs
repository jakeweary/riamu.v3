use std::fmt::{self, Write};

pub struct Display<'a, T> {
  amount: T,
  noun: &'a str,
}

pub trait Plural
where
  Self: Sized,
{
  fn plural<'a>(self, singular: &'a str, plural: &'a str) -> Display<'a, Self>;
}

macro_rules! impl_traits(($($T:ty)+) => {
  $(impl fmt::Display for Display<'_, $T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match (self.amount, f.alternate()) {
        (0, true) => "no".fmt(f)?,
        (1, true) => "one".fmt(f)?,
        (n, _) => n.fmt(f)?,
      }
      f.write_char(' ')?;
      f.write_str(self.noun)
    }
  })+

  $(impl Plural for $T {
    fn plural<'a>(self, singular: &'a str, plural: &'a str) -> Display<'a, Self> {
      Display {
        amount: self,
        noun: match self { 1 => singular, _ => plural },
      }
    }
  })+
});

impl_traits!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize);

#[test]
fn tests() {
  assert_eq!(format!("{}", 0.plural("thing", "things")), "0 things");
  assert_eq!(format!("{}", 1.plural("thing", "things")), "1 thing");
  assert_eq!(format!("{}", 2.plural("thing", "things")), "2 things");

  assert_eq!(format!("{:#}", 0.plural("thing", "things")), "no things");
  assert_eq!(format!("{:#}", 1.plural("thing", "things")), "one thing");
  assert_eq!(format!("{:#}", 2.plural("thing", "things")), "2 things");
}
