use std::future::Future;
use std::pin::Pin;

use ::serenity::all as serenity;

use super::{Context, Result};

#[derive(Debug)]
pub struct Command {
  pub name: &'static str,
  pub description: Option<&'static str>,
  pub owner_only: bool,

  #[allow(clippy::type_complexity)]
  pub run: for<'a> fn(&'a Context<'_>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>,
  pub options: Vec<&'static CommandOption>,
}

#[derive(Debug)]
pub struct CommandOption {
  pub name: &'static str,
  pub description: Option<&'static str>,
  pub choices: Option<&'static [CommandOptionChoice]>,
  pub required: bool,
  pub ty: serenity::CommandOptionType,
}

#[derive(Debug)]
pub struct CommandOptionChoice {
  pub name: &'static str,
  pub value: &'static str,
}
