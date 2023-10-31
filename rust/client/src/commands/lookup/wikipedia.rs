use ego_tree::NodeRef;
use scraper::{Html, Node};
use serde::Deserialize;
use serenity::all::*;
use url::Url;

use crate::client::{err, Context, Result};

#[macros::command(description = "Look up a term on Wikipedia (english lang.)")]
pub async fn en(ctx: &Context<'_>, term: &str) -> Result<()> {
  lookup(ctx, term, "en").await
}

#[macros::command(description = "Look up a term on Wikipedia (russian lang.)")]
pub async fn ru(ctx: &Context<'_>, term: &str) -> Result<()> {
  lookup(ctx, term, "ru").await
}

async fn lookup(ctx: &Context<'_>, term: &str, lang: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("fetching json…");
  let Ok(json) = Json::get(term, lang).await else {
    err::message!("could not find anything");
  };

  let json = match json {
    Json::Standard(json) => json,
    Json::Disambiguation => {
      err::message!("too ambiguous, be more specific");
    }
  };

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new().embed(json.embed());
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

// ---

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum Json {
  #[serde(rename = "standard")]
  Standard(Standard),
  #[serde(rename = "disambiguation")]
  Disambiguation,
}

#[derive(Debug, Deserialize)]
struct Standard {
  title: String,
  originalimage: Option<Originalimage>,
  content_urls: ContentUrls,
  extract_html: String,
}

#[derive(Debug, Deserialize)]
struct Originalimage {
  source: String,
}

#[derive(Debug, Deserialize)]
struct ContentUrls {
  desktop: Desktop,
}

#[derive(Debug, Deserialize)]
struct Desktop {
  page: String,
}

// ---

impl Json {
  async fn get(term: &str, lang: &str) -> reqwest::Result<Self> {
    // let mut cyrillic = term.matches(|c| matches!(c, 'а'..='я' | 'А'..='Я'));
    // let lang = if cyrillic.next().is_some() { "ru" } else { "en" };

    let url = format!("https://{lang}.wikipedia.org/api/rest_v1/page/summary");
    let mut url = Url::parse(&url).unwrap();
    url.path_segments_mut().unwrap().push(term);
    tracing::debug!(%url);

    let resp = reqwest::get(url).await?.error_for_status()?;
    let json = resp.json().await?;
    Ok(json)
  }
}

impl Standard {
  fn embed(&self) -> CreateEmbed {
    let embed = CreateEmbed::new()
      .url(&self.content_urls.desktop.page)
      .title(&self.title)
      .description(self.render());

    match &self.originalimage {
      Some(img) => embed.thumbnail(&img.source),
      None => embed,
    }
  }

  fn render(&self) -> String {
    fn visit(acc: &mut String, node: NodeRef<'_, Node>) {
      for node in node.children() {
        match node.value() {
          Node::Text(text) => acc.push_str(text),
          Node::Element(el) => match el.name() {
            "i" | "em" => {
              acc.push('*');
              visit(acc, node);
              acc.push('*');
            }
            "b" | "strong" => {
              acc.push_str("**");
              visit(acc, node);
              acc.push_str("**");
            }
            "u" => {
              acc.push_str("__");
              visit(acc, node);
              acc.push_str("__");
            }
            "br" => acc.push('\n'),
            _ => visit(acc, node),
          },
          _ => {}
        }
      }
    }

    let html = Html::parse_fragment(&self.extract_html);
    let root = html.tree.root();

    let mut acc = String::new();
    visit(&mut acc, root);
    acc
  }
}
