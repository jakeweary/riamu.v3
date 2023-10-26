use std::borrow::Cow;
use std::sync::OnceLock;

use ego_tree::NodeRef;
use itertools::Itertools;
use regex_lite::Regex;
use scraper::{CaseSensitivity::*, Html, Node};
use serde::Deserialize;
use serenity::all::*;
use url::Url;

use crate::client::{err, Context, Result};
use lib::fmt::plural::Plural;

#[macros::command(description = "Repost something from 4chan")]
pub async fn run(ctx: &Context<'_>, url: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::info!("parsing url…");
  let Some((domain, board_id, thread_id, post_id)) = parse_url(url) else {
    err::message!("failed to parse url");
  };

  tracing::info!("fetching json…");
  let thread = Thread::get(board_id, thread_id).await?;
  let op = &thread.posts[0];
  let (index, post) = match post_id {
    Some(id) => thread.posts.iter().find_position(|p| p.id == id).unwrap(),
    None => (0, op),
  };
  let subject = match op.subject.as_deref() {
    Some(subject) => Cow::Borrowed(subject),
    None => Cow::Owned(format!("/{}/{}", board_id, thread_id)),
  };

  #[rustfmt::skip]
  let url = format!("https://{}/{}/thread/{}#p{}",
    domain, board_id, thread_id, post_id.unwrap_or(op.id));

  let replies = thread.find_replies_to(post.id).count().plural("reply", "replies");
  let footer = format!("#{}/{} · {:#}", index + 1, thread.posts.len(), replies);

  let mut edit = EditInteractionResponse::new();
  let embed = CreateEmbed::new()
    .description(post.render(&url.parse()?))
    .author(CreateEmbedAuthor::new(subject).url(&url))
    .footer(CreateEmbedFooter::new(footer))
    .timestamp(Timestamp::from_unix_timestamp(post.time)?);

  if let Some(file) = &post.file {
    let url = format!("https://i.4cdn.org/{}/{}{}", board_id, file.id, file.ext);
    let att = CreateAttachment::url(ctx, &url).await?;
    edit = edit.new_attachment(att);
  }

  tracing::info!("sending response…");
  ctx.event.edit_response(ctx, edit.embed(embed)).await?;

  Ok(())
}

// ---

#[derive(Debug, Deserialize)]
struct Thread {
  posts: Vec<Post>,
}

#[derive(Debug, Deserialize)]
struct Post {
  time: i64,
  #[serde(rename = "no")]
  id: i64,
  #[serde(rename = "sub")]
  subject: Option<String>,
  #[serde(rename = "com")]
  comment: Option<String>,
  #[serde(flatten)]
  file: Option<File>,
}

#[derive(Debug, Deserialize)]
struct File {
  #[serde(rename = "tim")]
  id: i64,
  ext: String,
}

// ---

fn parse_url(url: &str) -> Option<(&str, &str, i64, Option<i64>)> {
  static RE: OnceLock<Regex> = OnceLock::new();

  let re = RE.get_or_init(|| {
    let re = r"(?i-u)https?://([\w.]+)/(\w+)/thread/(\d+)(?:/[\w-]+)?(?:#p(\d+))?";
    Regex::new(re).unwrap()
  });

  let captures = re.captures(url)?;
  let domain = captures.get(1).map(|b| b.as_str())?;
  let board = captures.get(2).map(|b| b.as_str())?;
  let thread = captures.get(3).and_then(|t| t.as_str().parse().ok())?;
  let post = captures.get(4).and_then(|p| p.as_str().parse().ok());
  Some((domain, board, thread, post))
}

impl Thread {
  async fn get(board: &str, thread: i64) -> reqwest::Result<Self> {
    let url = format!("https://a.4cdn.org/{board}/thread/{thread}.json");
    tracing::debug!(%url);

    let resp = reqwest::get(url).await?.error_for_status()?;
    let json = resp.json().await?;
    Ok(json)
  }

  fn find_replies_to(&self, id: i64) -> impl Iterator<Item = &Post> {
    self.posts.iter().filter({
      let pat = format!("#p{id}\"");
      move |&p| p.comment.as_deref().is_some_and(|c| c.contains(&pat))
    })
  }
}

impl Post {
  fn render(&self, base: &Url) -> String {
    fn visit(acc: &mut String, base: &Url, node: NodeRef<'_, Node>) {
      for node in node.children() {
        match node.value() {
          Node::Text(text) => acc.push_str(text),
          Node::Element(el) => 'el: {
            let name = el.name();

            if name == "a" {
              let href = el.attr("href").unwrap();
              let url = Url::options().base_url(Some(base)).parse(href).unwrap();

              acc.push('[');
              visit(acc, base, node);
              acc.push_str("](");
              acc.push_str(url.as_str());
              acc.push(')');
              break 'el;
            }

            if name == "span" && el.has_class("quote", CaseSensitive) {
              let mut tmp = String::new();
              visit(&mut tmp, base, node);
              acc.push_str("> ");
              acc.push_str(tmp.trim_start_matches(['>', ' ']));
              break 'el;
            }

            if name == "span" && el.has_class("deadlink", CaseSensitive) {
              acc.push_str("~~");
              visit(acc, base, node);
              acc.push_str("~~");
              break 'el;
            }

            if name == "pre" {
              acc.push_str("```");
              visit(acc, base, node);
              acc.push_str("```");
              break 'el;
            }

            if name == "s" {
              acc.push_str("||");
              visit(acc, base, node);
              acc.push_str("||");
              break 'el;
            }

            if name == "br" {
              acc.push('\n');
              break 'el;
            }

            visit(acc, base, node);
          }
          _ => {}
        }
      }
    }

    let html = self.comment.as_deref().unwrap_or_default();
    let html = Html::parse_fragment(html);
    let root = html.tree.root();

    let mut acc = String::new();
    visit(&mut acc, base, root);
    acc
  }
}
