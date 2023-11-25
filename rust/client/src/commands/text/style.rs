use lib::text::style;
use serenity::all::*;

use crate::client::{Context, Result};

#[macros::command(description = "Text style: regional indicators")]
pub async fn regional_indicators(ctx: &Context<'_>, input: &str) -> Result<()> {
  let input = format!("`{}`", input);
  reply(ctx, &input, style::regional_indicators).await
}

#[macros::command(description = "Text style: full width")]
pub async fn full_width(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::full_width).await
}

#[macros::command(description = "Text style: monospace")]
pub async fn monospace(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::monospace).await
}

#[macros::command(description = "Text style: double struck")]
pub async fn double_struck(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::double_struck).await
}

#[macros::command(description = "Text style: fractur")]
pub async fn fractur(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::fractur::regular).await
}

#[macros::command(description = "Text style: fractur (bold)")]
pub async fn fractur_bold(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::fractur::bold).await
}

#[macros::command(description = "Text style: script")]
pub async fn script(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::script::regular).await
}

#[macros::command(description = "Text style: script (bold)")]
pub async fn script_bold(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::script::bold).await
}

#[macros::command(description = "Text style: serif (bold)")]
pub async fn serif_bold(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::serif::bold).await
}

#[macros::command(description = "Text style: serif (italic)")]
pub async fn serif_italic(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::serif::italic).await
}

#[macros::command(description = "Text style: serif (bold italic)")]
pub async fn serif_bold_italic(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::serif::bold_italic).await
}

#[macros::command(description = "Text style: sans serif")]
pub async fn sans_serif(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::sans_serif::regular).await
}

#[macros::command(description = "Text style: sans serif (bold)")]
pub async fn sans_serif_bold(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::sans_serif::bold).await
}

#[macros::command(description = "Text style: sans serif (italic)")]
pub async fn sans_serif_italic(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::sans_serif::italic).await
}

#[macros::command(description = "Text style: sans serif (bold italic)")]
pub async fn sans_serif_bold_italic(ctx: &Context<'_>, input: &str) -> Result<()> {
  reply(ctx, input, style::sans_serif::bold_italic).await
}

// ---

async fn reply(ctx: &Context<'_>, input: &str, f: fn(char) -> char) -> Result<()> {
  let output = input.chars().map(f).collect::<String>();

  tracing::debug!("sending response…");
  let msg = CreateInteractionResponseMessage::new().content(output);
  let msg = CreateInteractionResponse::Message(msg);
  ctx.event.create_response(ctx, msg).await?;

  Ok(())
}
