use std::fmt::Write;
use std::mem;

use rand::prelude::*;
use serenity::all::*;

use crate::client::{Context, Result};

#[macros::command(desc = "Random integer in [min, max] range, defaults to [1, 100]")]
pub async fn int(ctx: &Context<'_>, min: Option<i64>, max: Option<i64>) -> Result<()> {
  let (min, max) = (min.unwrap_or(1), max.unwrap_or(100));
  let n = thread_rng().gen_range(min..=max);
  let text = format!("{}", n);
  reply(ctx, |msg| msg.content(text)).await
}

#[macros::command(desc = "Random real number in [min, max) range, defaults to [0, 1)")]
pub async fn real(ctx: &Context<'_>, min: Option<f64>, max: Option<f64>) -> Result<()> {
  let (min, max) = (min.unwrap_or(0.0), max.unwrap_or(1.0));
  let n = thread_rng().gen_range(min..max);
  let text = format!("{}", n);
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
  pub static ANSWERS: [&str; 20] = [
    "It is certain",
    "It is decidedly so",
    "Without a doubt",
    "Yes — definitely",
    "You may rely on it",
    "As I see it, yes",
    "Most likely",
    "Outlook good",
    "Yes",
    "Signs point to yes",
    "Reply hazy, try again",
    "Ask again later",
    "Better not tell you now",
    "Cannot predict now",
    "Concentrate and ask again",
    "Don't count on it",
    "My reply is no",
    "My sources say no",
    "Outlook not so good",
    "Very doubtful",
  ];

  let answer = ANSWERS.choose(&mut thread_rng()).unwrap();
  let text = format!("❔ {question}\n🎱 {answer}.");
  reply(ctx, |msg| msg.content(text)).await
}

#[macros::command(desc = "Toss a coin")]
pub async fn coin(
  ctx: &Context<'_>,
  #[min = 1]
  #[max = 1000]
  #[desc = "How many coins to toss"]
  n: Option<i64>,
) -> Result<()> {
  const HEADS: &str = "\u{26ab}\u{fe0e}";
  const TAILS: &str = "\u{26aa}\u{fe0e}";

  let text = match n {
    Some(1) | None => {
      if random() {
        format!("{HEADS} heads")
      } else {
        format!("{TAILS} tails")
      }
    }
    Some(n) => {
      let mut acc = String::new();
      let mut count = [0; 2];
      let mut rng = thread_rng();
      for _ in 0..n {
        let is_heads = rng.gen();
        count[is_heads as usize] += 1;
        acc.push(if is_heads { '\u{25cf}' } else { '\u{25cb}' });
      }
      let [t, h] = count;
      format!("{h}{HEADS} {t}{TAILS} ({n} coins)\n{acc}")
    }
  };

  reply(ctx, |msg| msg.content(text)).await
}

#[macros::command(desc = "Roll a six-sided die")]
pub async fn die(
  ctx: &Context<'_>,
  #[min = 1]
  #[max = 1000]
  #[desc = "How many dice to roll"]
  n: Option<i64>,
) -> Result<()> {
  let side = |n| unsafe { mem::transmute(0x2680 + n) };
  let text = match n {
    Some(1) | None => {
      let n = thread_rng().gen_range(0..6);
      format!("{} {}", side(n), n + 1)
    }
    Some(n) => {
      let mut head = String::new();
      let mut body = String::new();
      let mut count = [0; 6];
      let mut sum = 0;
      let mut rng = thread_rng();
      for _ in 0..n {
        let n = rng.gen_range(0..6);
        body.push(side(n));
        sum += n + 1;
        count[n as usize] += 1;
      }
      for (i, n) in count.iter().enumerate() {
        write!(head, " + {}{}", n, side(i as u32))?;
      }
      write!(head, " = {sum} ({n} dice)")?;
      format!("{}\n{}", &head[3..], body)
    }
  };

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
