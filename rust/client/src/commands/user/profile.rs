use serenity::all::*;

use crate::client::{err, Context, Result};

#[macros::command(description = "Get someone's avatar")]
pub async fn avatar(ctx: &Context<'_>, user: &User) -> Result<()> {
  let Some(url) = user.avatar_url() else {
    err::message!("the user doesn't have a custom avatar");
  };

  reply(ctx, url).await
}

#[macros::command(description = "Get someone's banner")]
pub async fn banner(ctx: &Context<'_>, user: &User) -> Result<()> {
  let user = ctx.serenity.http.get_user(user.id).await?;
  let Some(url) = user.banner_url() else {
    err::message!("the user doesn't have a custom banner");
  };

  reply(ctx, url).await
}

async fn reply(ctx: &Context<'_>, content: String) -> Result<()> {
  tracing::debug!("sending response…");
  let msg = CreateInteractionResponseMessage::new().content(content);
  let msg = CreateInteractionResponse::Message(msg);
  ctx.event.create_response(ctx, msg).await?;

  Ok(())
}
