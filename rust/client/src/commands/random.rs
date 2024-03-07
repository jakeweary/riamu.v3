use rand::prelude::*;
use serenity::all::*;

use crate::client::{Context, Result};

#[macros::command(desc = "Random integer in [min, max) interval, defaults to [0, 100)")]
pub async fn int(ctx: &Context<'_>, min: Option<i64>, max: Option<i64>) -> Result<()> {
  let (min, max) = (min.unwrap_or(0), max.unwrap_or(100));
  let n = thread_rng().gen_range(min..max);
  let text = format!("# {}", n);
  reply(ctx, |msg| msg.content(text)).await
}

#[macros::command(desc = "Random real number in [min, max) interval, defaults to [0, 1)")]
pub async fn real(ctx: &Context<'_>, min: Option<f64>, max: Option<f64>) -> Result<()> {
  let (min, max) = (min.unwrap_or(0.0), max.unwrap_or(1.0));
  let n = thread_rng().gen_range(min..max);
  let text = format!("# {}", n);
  reply(ctx, |msg| msg.content(text)).await
}

#[macros::command(desc = "Toss a coin")]
pub async fn coin(ctx: &Context<'_>) -> Result<()> {
  let coin = if random() { "heads" } else { "tails" };
  let text = format!("# \u{1fa99} {}", coin);
  reply(ctx, |msg| msg.content(text)).await
}

#[macros::command(desc = "Random color")]
pub async fn color(ctx: &Context<'_>) -> Result<()> {
  let color = random::<u32>() & 0xffffff;
  let [b, g, r, _] = color.to_le_bytes();
  let text = format!("#{:06x} · rgb({}, {}, {})", color, r, g, b);
  let embed = CreateEmbed::new().color(color).description(text);
  reply(ctx, |msg| msg.embed(embed)).await
}

#[macros::command(desc = "Ask the magic 8 ball")]
pub async fn eightball(ctx: &Context<'_>, question: &str) -> Result<()> {
  let answer = ANSWERS.choose(&mut thread_rng()).unwrap();
  let text = format!("❔ {question}\n🎱 {answer}");
  reply(ctx, |msg| msg.content(text)).await
}

// ---

async fn reply<F>(ctx: &Context<'_>, f: F) -> Result<()>
where
  F: FnOnce(CreateInteractionResponseMessage) -> CreateInteractionResponseMessage,
{
  tracing::debug!("sending response…");
  let msg = CreateInteractionResponseMessage::new();
  let msg = CreateInteractionResponse::Message(f(msg));
  ctx.event.create_response(ctx, msg).await?;
  Ok(())
}

pub static ANSWERS: [&str; 20] = [
  "It is certain.",
  "It is decidedly so.",
  "Without a doubt.",
  "Yes — definitely.",
  "You may rely on it.",
  "As I see it, yes.",
  "Most likely.",
  "Outlook good.",
  "Yes.",
  "Signs point to yes.",
  "Reply hazy, try again.",
  "Ask again later.",
  "Better not tell you now.",
  "Cannot predict now.",
  "Concentrate and ask again.",
  "Don't count on it.",
  "My reply is no.",
  "My sources say no.",
  "Outlook not so good.",
  "Very doubtful.",
];
