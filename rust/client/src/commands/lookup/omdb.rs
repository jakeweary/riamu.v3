use serenity::all::*;

use crate::client::{err, Context, Result};

#[macros::command(desc = "Look up a movie on OMDB")]
pub async fn run(ctx: &Context<'_>, movie: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("fetching json…");
  let api = omdb::Api::new(&ctx.client.env.omdb_api_key);
  let omdb::Response::Success(json) = api.query(movie).await? else {
    err::message!("could not find anything");
  };

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new().embed(json.embed()?);
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

mod omdb {
  use std::fmt::Write;

  use serde::Deserialize;
  use serenity::all::*;
  use url::Url;

  pub struct Api<'a> {
    key: &'a str,
  }

  impl<'a> Api<'a> {
    pub fn new(key: &'a str) -> Self {
      Self { key }
    }

    pub async fn query(&self, title: &str) -> reqwest::Result<Response> {
      let params = &[("apikey", &*self.key), ("t", title)];
      let url = "https://www.omdbapi.com";
      let url = Url::parse_with_params(url, params).unwrap();
      tracing::debug!(%url);

      let resp = reqwest::get(url).await?.error_for_status()?;
      let json = resp.json().await?;
      Ok(json)
    }
  }

  #[derive(Debug, Deserialize)]
  #[serde(rename_all = "PascalCase", untagged)]
  pub enum Response {
    Error(Error),
    Success(Success),
  }

  #[derive(Debug, Deserialize)]
  #[serde(rename_all = "PascalCase")]
  pub struct Error {
    pub error: String,
  }

  #[derive(Debug, Deserialize)]
  #[serde(rename_all = "PascalCase")]
  pub struct Success {
    pub title: String,
    pub year: String,
    pub rated: String,
    pub released: String,
    pub runtime: String,
    pub genre: String,
    pub director: String,
    pub writer: String,
    pub actors: String,
    pub plot: String,
    pub language: String,
    pub country: String,
    pub awards: String,
    pub poster: String,
    pub ratings: Vec<Rating>,
    pub metascore: String,
    pub r#type: String,
    #[serde(flatten)]
    pub imdb: Imdb,
    #[serde(flatten)]
    pub movie_extra: Option<MovieExtra>,
  }

  #[derive(Debug, Deserialize)]
  #[serde(rename_all = "PascalCase")]
  pub struct MovieExtra {
    #[serde(rename = "DVD")]
    pub dvd: String,
    pub box_office: String,
    pub production: String,
    pub website: String,
  }

  #[derive(Debug, Deserialize)]
  #[serde(rename_all = "PascalCase")]
  pub struct Rating {
    pub source: String,
    pub value: String,
  }

  #[derive(Debug, Deserialize)]
  pub struct Imdb {
    #[serde(rename = "imdbRating")]
    pub rating: String,
    #[serde(rename = "imdbVotes")]
    pub votes: String,
    #[serde(rename = "imdbID")]
    pub id: String,
  }

  impl Success {
    pub fn embed(&self) -> Result<CreateEmbed, std::fmt::Error> {
      let url = format!("https://www.imdb.com/title/{}", self.imdb.id);
      let title = format!("{} ({})", self.title, self.year);
      let footer = format!("{} · {}", self.runtime, self.genre);

      let embed = CreateEmbed::new()
        .url(url)
        .title(title)
        .description(&self.plot)
        .field("", self.people()?, false)
        .field("", self.details()?, true)
        .field("", self.ratings()?, true)
        .footer(CreateEmbedFooter::new(footer));

      match &*self.poster {
        "N/A" => Ok(embed),
        url => Ok(embed.thumbnail(url)),
      }
    }

    fn people(&self) -> Result<String, std::fmt::Error> {
      let mut acc = String::new();
      writeln!(acc, "**Directors:** {}", self.director)?;
      writeln!(acc, "**Writers:** {}", self.writer)?;
      writeln!(acc, "**Actors:** {}", self.actors)?;
      Ok(acc)
    }

    fn details(&self) -> Result<String, std::fmt::Error> {
      let mut acc = String::new();
      writeln!(acc, "**Countries:** {}", self.country)?;
      writeln!(acc, "**Languages:** {}", self.language)?;
      if let Some(extra) = &self.movie_extra {
        writeln!(acc, "**Box office:** {}", extra.box_office)?;
      }
      Ok(acc)
    }

    fn ratings(&self) -> Result<String, std::fmt::Error> {
      let mut acc = String::new();
      writeln!(acc, "**IMDB:** {} ({} votes)", self.imdb.rating, self.imdb.votes)?;
      for r in &self.ratings {
        if r.source != "Internet Movie Database" {
          writeln!(acc, "**{}:** {}", r.source, r.value)?;
        }
      }
      Ok(acc)
    }
  }
}
