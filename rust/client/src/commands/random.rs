use std::fmt::Write;
use std::mem;

use rand::prelude::*;
use random::{Random, XorShift64};
use serenity::all::*;

use crate::client::{command, Context, Result};

#[command(desc = "Random integer in [min, max] range, defaults to [1, 100]")]
pub async fn int(ctx: &Context<'_>, min: Option<i64>, max: Option<i64>) -> Result<()> {
  let (min, max) = (min.unwrap_or(1), max.unwrap_or(100));
  let n = thread_rng().gen_range(min..=max);
  let text = format!("{}", n);
  reply(ctx, |msg| msg.content(text)).await
}

#[command(desc = "Random real number in [min, max) range, defaults to [0, 1)")]
pub async fn real(ctx: &Context<'_>, min: Option<f64>, max: Option<f64>) -> Result<()> {
  let (min, max) = (min.unwrap_or(0.0), max.unwrap_or(1.0));
  let n = thread_rng().gen_range(min..max);
  let text = format!("{}", n);
  reply(ctx, |msg| msg.content(text)).await
}

#[command(desc = "Random color")]
pub async fn color(ctx: &Context<'_>) -> Result<()> {
  let color = random::<u32>() & 0xffffff;
  let [b, g, r, _] = color.to_le_bytes();
  let text = format!("#{:06x} · rgb({}, {}, {})", color, r, g, b);
  let embed = CreateEmbed::new().color(color).description(text);
  reply(ctx, |msg| msg.embed(embed)).await
}

#[command(desc = "Take a card from a shuffled standard 52-card deck")]
pub async fn card(
  ctx: &Context<'_>,
  #[min = 1]
  #[max = 52]
  #[desc = "How many cards to take"]
  n: Option<i64>,
) -> Result<()> {
  let mut cards = [0; 52];
  for (i, c) in cards.iter_mut().enumerate() {
    *c = i as u8;
  }

  let mut rng = XorShift64::from_time();
  let (shuffled, _) = cards.partial_shuffle(&mut rng, n.unwrap_or(1) as usize);

  let mut acc = String::new();
  for &mut c in shuffled {
    let (rank, suit) = (c / 4, c % 4);
    let rank = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"][rank as usize];
    let suit = ['\u{2660}', '\u{2663}', '\u{2665}', '\u{2666}'][suit as usize];
    write!(acc, "{}{}\u{fe0e}\u{2002}", rank, suit)?;
  }

  reply(ctx, |msg| msg.content(acc)).await
}

#[command(desc = "Toss a coin")]
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
      let mut tossed = 0;
      let mut rng = XorShift64::from_time();
      'outer: loop {
        let bits = rng.next_u64();
        for i in 0..64 {
          let bit = bits >> i & 1;
          acc.push(if bit == 0 { '\u{25cb}' } else { '\u{25cf}' });
          count[bit as usize] += 1;
          tossed += 1;
          if tossed == n {
            break 'outer;
          }
        }
      }
      let [t, h] = count;
      format!("{h}{HEADS} {t}{TAILS} ({n} coins)\n{acc}")
    }
  };

  reply(ctx, |msg| msg.content(text)).await
}

#[command(desc = "Roll a six-sided die")]
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
      let mut rng = XorShift64::from_time();
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

#[command(desc = "Ask the magic 8 ball")]
pub async fn eightball(ctx: &Context<'_>, #[desc = "A yes–no question"] question: &str) -> Result<()> {
  pub static ANSWERS: [&str; 20] = [
    "it is certain",
    "it is decidedly so",
    "without a doubt",
    "yes — definitely",
    "you may rely on it",
    "as I see it, yes",
    "most likely",
    "outlook good",
    "yes",
    "signs point to yes",
    "reply hazy, try again",
    "ask again later",
    "better not tell you now",
    "cannot predict now",
    "concentrate and ask again",
    "don't count on it",
    "my reply is no",
    "my sources say no",
    "outlook not so good",
    "very doubtful",
  ];

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
