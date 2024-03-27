use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub enum CommandError {
  Message(String),
  Timeout,
}

impl Error for CommandError {}
impl Display for CommandError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    Debug::fmt(self, f)
  }
}

pub mod err {
  macro_rules! message(($($arg:tt)+) => {
    return ::std::result::Result::Err({
      let msg = format!($($arg)+);
      $crate::client::CommandError::Message(msg).into()
    })
  });

  macro_rules! timeout(() => {
    return ::std::result::Result::Err({
      $crate::client::CommandError::Timeout.into()
    })
  });

  pub(crate) use message;
  pub(crate) use timeout;
}
