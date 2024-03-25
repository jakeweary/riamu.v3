use std::fmt::{self, Display, Formatter};

use ::serenity::all as serenity;
use util::hash::splitmix64;

use super::{commands, Client, Command};

#[derive(Debug)]
pub struct Context<'a> {
  pub id: Id,
  pub client: &'a Client,
  pub event: &'a serenity::CommandInteraction,
  pub serenity: &'a serenity::Context,
  pub command: &'a Command,
  pub options: Vec<serenity::ResolvedOption<'a>>,
}

impl<'a> Context<'a> {
  #[rustfmt::skip]
  pub fn new(client: &'a Client, ctx: &'a serenity::Context, event: &'a serenity::CommandInteraction) -> Self {
    let id = Id(event.id.get());
    let (command, options) = commands::resolve(&client.commands, event.data.options(), &event.data.name);
    Context { id, client, serenity: ctx, event, command, options }
  }

  pub async fn filesize_limit(&self) -> serenity::Result<u64> {
    let tier = match self.event.guild_id {
      Some(id) => id.to_partial_guild(self).await?.premium_tier.into(),
      None => 0,
    };
    Ok([25, 25, 50, 100][tier as usize] << 20)
  }

  pub async fn progress(&self, message: impl Into<String>) -> serenity::Result<serenity::Message> {
    let button = serenity::CreateButton::new("…")
      .style(serenity::ButtonStyle::Secondary)
      .disabled(true)
      .label(message);
    let buttons = serenity::CreateActionRow::Buttons(vec![button]);
    let edit = serenity::EditInteractionResponse::new().components(vec![buttons]);
    let edit = self.event.edit_response(self, edit);
    edit.await
  }
}

impl serenity::CacheHttp for Context<'_> {
  fn http(&self) -> &serenity::Http {
    self.serenity.http()
  }
}

impl AsRef<serenity::Http> for Context<'_> {
  fn as_ref(&self) -> &serenity::Http {
    self.serenity.as_ref()
  }
}

impl AsRef<serenity::Cache> for Context<'_> {
  fn as_ref(&self) -> &serenity::Cache {
    self.serenity.as_ref()
  }
}

impl AsRef<serenity::ShardMessenger> for Context<'_> {
  fn as_ref(&self) -> &serenity::ShardMessenger {
    self.serenity.as_ref()
  }
}

// ---

#[derive(Debug)]
pub struct Id(pub u64);

impl Display for Id {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{:016X}", splitmix64::mix(self.0))
  }
}
