use std::borrow::Cow;
use std::sync::OnceLock;

use chrono::DateTime;
use regex_lite::{Captures, Regex};
use serde::Deserialize;
use serenity::all::*;
use url::Url;

use crate::client::{err, Context, Result};

#[macros::command(description = "Look up a term on Urban Dictionary")]
pub async fn run(ctx: &Context<'_>, term: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::info!("fetching json…");
  let json = Json::get(term, 1).await?;

  let Some(def) = json.list.iter().find(|d| d.thumbs_up >= d.thumbs_down) else {
    err::message!("could not find anything");
  };

  tracing::info!("sending response…");
  let edit = EditInteractionResponse::new().embed(def.embed());
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

// ---

#[derive(Debug, Deserialize, Default)]
struct Json {
  list: Vec<Definition>,
}

#[derive(Debug, Deserialize)]
struct Definition {
  word: String,
  definition: String,
  example: String,
  written_on: String,
  thumbs_up: i64,
  thumbs_down: i64,
}

// ---

impl Json {
  async fn get(term: &str, pages: usize) -> reqwest::Result<Self> {
    let url = "https://api.urbandictionary.com/v0/define";
    let url = Url::parse_with_params(url, &[("term", &term)]).unwrap();

    let mut acc = Json::default();

    for page in 1..=pages {
      let mut url = url.clone();
      url.query_pairs_mut().append_pair("page", &page.to_string());
      tracing::trace!(%url);

      let resp = reqwest::get(url).await?.error_for_status()?;
      let json = resp.json::<Json>().await?;

      if json.list.is_empty() {
        break;
      }

      acc.list.extend(json.list);
    }

    Ok(acc)
  }
}

impl Definition {
  fn embed(&self) -> CreateEmbed {
    let definition = sanitize(&self.definition);
    let example = sanitize(&self.example);

    let desc = format!("{}\n\n*{}*", definition, example);
    let footer = format!("👍{} 👎{}", self.thumbs_up, self.thumbs_down);
    let time = DateTime::parse_from_rfc3339(&self.written_on).unwrap();

    CreateEmbed::new()
      .url(term_link(&self.word))
      .title(&self.word)
      .description(insert_term_links(&desc))
      .footer(CreateEmbedFooter::new(footer))
      .timestamp(time)
  }
}

fn insert_term_links(text: &str) -> Cow<'_, str> {
  static RE: OnceLock<Regex> = OnceLock::new();
  let re = RE.get_or_init(|| Regex::new(r"\[(.+?)\]").unwrap());

  re.replace_all(text, |captures: &Captures<'_>| {
    let term = captures.get(1).unwrap().as_str();
    format!("[{}]({})", term, term_link(term))
  })
}

fn term_link(term: &str) -> Url {
  let url = "https://urbandictionary.com/define.php";
  Url::parse_with_params(url, &[("term", term)]).unwrap()
}

fn sanitize(text: &str) -> String {
  text.trim().replace('*', r"\*").replace('_', r"\_")
}
