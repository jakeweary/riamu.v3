use std::fmt::{self, Display, Formatter};
use std::sync::OnceLock;

use chrono::DateTime;
use regex::Regex;
use regex_ext::RegexExt;
use serde::Deserialize;
use serenity::all::*;
use url::Url;

use crate::client::{err, Context, Result};

#[macros::command(desc = "Look up a term on Urban Dictionary")]
pub async fn run(ctx: &Context<'_>, term: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("fetching json…");
  let json = Json::get(term, 1).await?;

  let Some(def) = json.list.iter().find(|d| d.thumbs_up >= d.thumbs_down) else {
    err::message!("could not find anything");
  };

  tracing::debug!("sending response…");
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

    let mut acc = Self::default();

    for page in 1..=pages {
      let mut url = url.clone();
      url.query_pairs_mut().append_pair("page", &page.to_string());
      tracing::trace!(%url);

      let resp = reqwest::get(url).await?.error_for_status()?;
      let json = resp.json::<Self>().await?;

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
      .description(TermLinks(&desc).to_string())
      .footer(CreateEmbedFooter::new(footer))
      .timestamp(time)
  }
}

fn sanitize(text: &str) -> String {
  text.trim().replace('*', r"\*").replace('_', r"\_")
}

fn term_link(term: &str) -> Url {
  let url = "https://urbandictionary.com/define.php";
  Url::parse_with_params(url, &[("term", term)]).unwrap()
}

struct TermLinks<'a>(&'a str);

impl Display for TermLinks<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\[(.+?)\]").unwrap());

    re.replace_all_fmt(f, self.0, |f, caps| {
      let (_, [term]) = caps.extract();
      write!(f, "[{}]({})", term, term_link(term))
    })
  }
}
