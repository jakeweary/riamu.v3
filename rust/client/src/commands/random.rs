use rand::prelude::*;
use serenity::all::*;

use crate::client::{Context, Result};

#[macros::command(description = "Random integer in [min, max) interval, defaults to [0, 100)")]
pub async fn int(ctx: &Context<'_>, min: Option<i64>, max: Option<i64>) -> Result<()> {
  let (min, max) = (min.unwrap_or(0), max.unwrap_or(100));
  let n = thread_rng().gen_range(min..max);
  let text = format!("# {}", n);
  reply(ctx, text).await
}

#[macros::command(description = "Random real number in [min, max) interval, defaults to [0, 1)")]
pub async fn real(ctx: &Context<'_>, min: Option<f64>, max: Option<f64>) -> Result<()> {
  let (min, max) = (min.unwrap_or(0.0), max.unwrap_or(1.0));
  let n = thread_rng().gen_range(min..max);
  let text = format!("# {}", n);
  reply(ctx, text).await
}

#[macros::command(description = "Flip a coin")]
pub async fn coin(ctx: &Context<'_>) -> Result<()> {
  let coin = if random() { "heads" } else { "tails" };
  let text = format!("# \u{1fa99} {}", coin);
  reply(ctx, text).await
}

// ---

async fn reply(ctx: &Context<'_>, content: String) -> Result<()> {
  tracing::debug!("sending response…");
  let msg = CreateInteractionResponseMessage::new().content(content);
  let msg = CreateInteractionResponse::Message(msg);
  ctx.event.create_response(ctx, msg).await?;

  Ok(())
}
