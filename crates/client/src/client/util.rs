use std::any::Any;
use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result};

use serenity::all::*;

pub struct SlashCommandDisplay<'a>(pub &'a CommandData);
pub struct SlashCommandOptionsDisplay<'a>(pub &'a [CommandDataOption]);

impl Display for SlashCommandDisplay<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    let name = &self.0.name;
    let options = SlashCommandOptionsDisplay(&self.0.options);
    f.write_fmt(format_args!("/{name}{options}"))?;

    Ok(())
  }
}

impl Display for SlashCommandOptionsDisplay<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    use CommandDataOptionValue::*;

    for option in self.0 {
      f.write_fmt(format_args!(" {}", option.name))?;

      match &option.value {
        SubCommand(options) | SubCommandGroup(options) => {
          Self(options).fmt(f)?;
        }
        value => {
          let value: &dyn Display = match value {
            Boolean(v) => v,
            Integer(v) => v,
            Number(v) => v,
            String(v) => v,
            Attachment(v) => v,
            Channel(v) => v,
            Mentionable(v) => v,
            Role(v) => v,
            User(v) => v,
            _ => unimplemented!(),
          };

          f.write_fmt(format_args!(":{}", value))?;
        }
      }
    }

    Ok(())
  }
}

pub fn panic_message(panic: Box<dyn Any + Send + '_>) -> Cow<'_, str> {
  if let Some(s) = panic.downcast_ref() {
    return Cow::Borrowed(*s);
  }
  if let Ok(s) = panic.downcast() {
    return Cow::Owned(*s);
  }
  unreachable!()
}
