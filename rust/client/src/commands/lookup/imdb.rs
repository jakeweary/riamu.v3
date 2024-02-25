use std::borrow::Cow;
use std::fmt::{self, Write};
use std::result::Result as StdResult;

use lib::fmt::num::Format;
use scraper::{Html, Selector};
use serde::Deserialize;
use serenity::all::*;
use url::Url;

use crate::client::{err, Context, Result};

#[macros::command(desc = "Look up a movie on IMDB")]
pub async fn run(ctx: &Context<'_>, movie: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("searching…");
  let json = Search::search(movie).await?;

  let Some(movie) = json.list.iter().find(|sr| sr.id.starts_with("tt")) else {
    err::message!("could not find anything");
  };

  tracing::debug!("fetching json…");
  let json = Json::get(&movie.id).await?;

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new().embed(json.embed());
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

// ---

#[derive(Debug, Deserialize)]
struct Search {
  #[serde(rename = "d")]
  list: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
  id: String,
}

// ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Json {
  url: String,
  name: String,
  description: String,
  #[serde(default)]
  director: Vec<Director>,
  #[serde(default)]
  creator: Vec<Creator>,
  #[serde(default)]
  actor: Vec<Actor>,
  genre: Vec<String>,
  image: Option<String>,
  duration: Option<String>,
  date_published: Option<String>,
  aggregate_rating: Option<AggregateRating>,
}

#[derive(Debug, Deserialize)]
struct AggregateRating {
  #[serde(rename = "ratingCount")]
  count: i64,
  #[serde(rename = "ratingValue")]
  value: f64,
}

#[derive(Debug, Deserialize)]
struct Actor {
  url: String,
  name: String,
}

#[derive(Debug, Deserialize)]
struct Director {
  url: String,
  name: String,
}

#[derive(Debug, Deserialize)]
struct Creator {
  url: String,
  name: Option<String>,
}

// ---

impl Search {
  async fn search(movie: &str) -> reqwest::Result<Self> {
    let url = "https://v2.sg.media-imdb.com/suggestion/h";
    let mut url = Url::parse(url).unwrap();
    url.path_segments_mut().unwrap().push(&format!("{}.json", movie));
    tracing::debug!(%url);

    let resp = reqwest::get(url).await?.error_for_status()?;
    let json = resp.json().await?;
    Ok(json)
  }
}

impl Json {
  async fn get(id: &str) -> Result<Self> {
    let url = format!("https://www.imdb.com/title/{}", id);
    tracing::debug!(%url);

    let resp = reqwest::get(url).await?.error_for_status()?;
    let html = resp.text().await?;

    let html = Html::parse_document(&html);
    let select = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();
    let json = html.select(&select).flat_map(|e| e.text()).next().unwrap();
    let json = serde_json::from_str(json)?;
    Ok(json)
  }

  fn embed(&self) -> CreateEmbed {
    let title = lib::html::strip(&self.title());
    let desc = lib::html::strip(&self.description().unwrap());
    let embed = CreateEmbed::new().url(&self.url).title(title).description(desc);

    match &self.image {
      Some(url) => embed.thumbnail(url),
      None => embed,
    }
  }

  fn title(&self) -> Cow<'_, str> {
    match self.date_published.as_ref().and_then(|date| date.split_once('-')) {
      Some((year, _)) => Cow::Owned(format!("{} ({})", self.name, year)),
      None => Cow::Borrowed(&self.name),
    }
  }

  fn description(&self) -> StdResult<String, fmt::Error> {
    let mut acc = String::new();

    let directors = self.director.iter();
    let writers = self.creator.iter().filter_map(|c| Some((c.name.as_ref()?, &c.url)));
    let stars = self.actor.iter();

    writeln!(acc, "{}", self.description)?;

    for (i, director) in directors.enumerate() {
      let sep = if i == 0 { "\nDirectors: " } else { ", " };
      write!(acc, "{}[{}]({})", sep, director.name, director.url)?;
    }

    for (i, (name, url)) in writers.enumerate() {
      let sep = if i == 0 { "\nWriters: " } else { ", " };
      write!(acc, "{}[{}]({})", sep, name, url)?;
    }

    for (i, star) in stars.enumerate() {
      let sep = if i == 0 { "\nStars: " } else { ", " };
      write!(acc, "{}[{}]({})", sep, star.name, star.url)?;
    }

    write!(acc, "\n\n")?;

    if let Some(rating) = &self.aggregate_rating {
      write!(acc, "{}/10 ({}) · ", rating.value, rating.count.k())?;
    }

    if let Some(dur) = &self.duration {
      let dur = dur.trim_start_matches("PT").to_ascii_lowercase();
      write!(acc, "{} · ", dur)?;
    }

    for (i, genre) in self.genre.iter().enumerate() {
      let sep = if i == 0 { "" } else { ", " };
      write!(acc, "{}{}", sep, genre)?;
    }

    Ok(acc)
  }
}
