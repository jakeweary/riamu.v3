#[derive(Debug)]
pub enum CommandError {
  Message(String),
  Timeout,
}

impl std::error::Error for CommandError {}
impl std::fmt::Display for CommandError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Debug::fmt(self, f)
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
