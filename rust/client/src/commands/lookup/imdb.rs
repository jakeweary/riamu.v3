use serenity::all::*;

use crate::client::{command, err, Context, Result};

#[command(desc = "Look up a movie on IMDB")]
pub async fn run(ctx: &Context<'_>, movie: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("fetching json…");
  let json = match api::query(movie).await {
    Ok(Some(html)) => api::extract_json(&html)?,
    _ => err::message!("could not find anything"),
  };

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new().embed(json.embed()?);
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

mod api {
  use std::borrow::Cow;
  use std::fmt::Write;

  use fmt::num::Format as _;
  use reqwest::header;
  use scraper::{Html, Selector};
  use serde::Deserialize;
  use serenity::all::*;
  use url::Url;
  use util::html;

  // ---

  #[derive(Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Response {
    pub url: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub director: Vec<Director>,
    #[serde(default)]
    pub creator: Vec<Creator>,
    #[serde(default)]
    pub actor: Vec<Actor>,
    pub genre: Vec<String>,
    pub image: Option<String>,
    pub duration: Option<String>,
    pub date_published: Option<String>,
    pub aggregate_rating: Option<AggregateRating>,
  }

  #[derive(Debug, Deserialize)]
  pub struct AggregateRating {
    #[serde(rename = "ratingCount")]
    pub count: i64,
    #[serde(rename = "ratingValue")]
    pub value: f64,
  }

  #[derive(Debug, Deserialize)]
  pub struct Actor {
    pub url: String,
    pub name: String,
  }

  #[derive(Debug, Deserialize)]
  pub struct Director {
    pub url: String,
    pub name: String,
  }

  #[derive(Debug, Deserialize)]
  pub struct Creator {
    pub url: String,
    pub name: Option<String>,
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

  pub async fn query(movie: &str) -> reqwest::Result<Option<String>> {
    let client = reqwest::Client::builder().build()?;

    let url = "https://v2.sg.media-imdb.com/suggestion/h";
    let mut url = Url::parse(url).unwrap();
    url.path_segments_mut().unwrap().push(&format!("{}.json", movie));
    let req = client.get(url);
    let res = req.send().await?.error_for_status()?;
    let json: Search = res.json().await?;

    let Some(movie) = json.list.iter().find(|sr| sr.id.starts_with("tt")) else {
      return Ok(None);
    };

    let url = format!("https://www.imdb.com/title/{}", movie.id);
    let req = client.get(url).header(header::ACCEPT_LANGUAGE, "en");
    let res = req.send().await?.error_for_status()?;
    let html = res.text().await?;

    Ok(Some(html))
  }

  pub fn extract_json(html: &str) -> serde_json::Result<Response> {
    let html = Html::parse_document(html);
    let select = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();
    let json = html.select(&select).flat_map(|e| e.text()).next().unwrap();
    serde_json::from_str(json)
  }

  impl Response {
    pub fn embed(&self) -> fmt::Result<CreateEmbed> {
      let title = html::strip(&self.title());
      let desc = html::strip(&self.description()?);
      let embed = CreateEmbed::new().url(&self.url).title(title).description(desc);

      match &self.image {
        Some(url) => Ok(embed.thumbnail(url)),
        None => Ok(embed),
      }
    }

    fn title(&self) -> Cow<'_, str> {
      match self.date_published.as_ref().and_then(|date| date.split_once('-')) {
        Some((year, _)) => Cow::Owned(format!("{} ({})", self.name, year)),
        None => Cow::Borrowed(&self.name),
      }
    }

    fn description(&self) -> fmt::Result<String> {
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
}
