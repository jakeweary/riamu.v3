use serenity::all::ResolvedValue as R;
use serenity::all::*;

use super::CommandOptionChoice;

pub trait CommandOptionTrait<'a> {
  const TYPE: CommandOptionType;
  const CHOICES: Option<&'static [CommandOptionChoice]> = None;
  const REQUIRED: bool = true;

  fn extract(value: Option<&'a ResolvedOption<'_>>) -> Self;
}

impl<'a, T> CommandOptionTrait<'a> for Option<T>
where
  T: CommandOptionTrait<'a>,
{
  const TYPE: CommandOptionType = T::TYPE;
  const CHOICES: Option<&'static [CommandOptionChoice]> = T::CHOICES;
  const REQUIRED: bool = false;

  fn extract(value: Option<&'a ResolvedOption<'_>>) -> Self {
    value.map(|value| T::extract(Some(value)))
  }
}

macro_rules! impl_trait {
  ($lifetime:tt, $T:ty: $type:ident, $pat:pat => $expr:expr) => {
    impl<$lifetime> CommandOptionTrait<$lifetime> for $T {
      const TYPE: CommandOptionType = CommandOptionType::$type;

      fn extract(value: Option<&'a ResolvedOption<'_>>) -> Self {
        match value {
          Some(&ResolvedOption { value: $pat, .. }) => $expr,
          _ => unreachable!(),
        }
      }
    }
  };
}

impl_trait!('a, bool: Boolean, R::Boolean(value) => value);
impl_trait!('a, i64: Integer, R::Integer(value) => value);
impl_trait!('a, f64: Number, R::Number(value) => value);
impl_trait!('a, &'a str: String, R::String(value) => value);
impl_trait!('a, &'a Attachment: Attachment, R::Attachment(value) => value);
impl_trait!('a, &'a PartialChannel: Channel, R::Channel(value) => value);
impl_trait!('a, &'a Role: Role, R::Role(value) => value);
impl_trait!('a, &'a User: User, R::User(value, _) => value);
impl_trait!('a, &'a PartialMember: User, R::User(_, Some(value)) => value);
impl_trait!('a, &'a ResolvedValue<'a>: Mentionable, ref value => value);
