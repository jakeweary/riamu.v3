use super::*;

pub trait Format
where
  Self: Sized,
{
  fn si(self) -> Formatted<Si>;
  fn iec(self) -> Formatted<Iec>;
  fn k(self) -> Formatted<K>;
}

macro_rules! impl_trait(($($T:ty)+) => {
  $(impl Format for $T {
    fn si(self) -> Formatted<Si> { si(self as f64) }
    fn iec(self) -> Formatted<Iec> { iec(self as f64) }
    fn k(self) -> Formatted<K> { k(self as f64) }
  })+
});

impl_trait!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize f32 f64);
