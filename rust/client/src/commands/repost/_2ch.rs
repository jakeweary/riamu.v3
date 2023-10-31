use std::sync::OnceLock;

use ego_tree::NodeRef;
use regex_lite::Regex;
use scraper::{CaseSensitivity::*, Html, Node};
use serde::Deserialize;
use serenity::all::*;

use crate::client::{err, Context, Result};
use lib::fmt::plural::Plural;

#[macros::command(description = "Repost something from 2ch")]
pub async fn run(ctx: &Context<'_>, url: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("parsing url…");
  let Some((domain, board_id, thread_id, post_id)) = parse_url(url) else {
    err::message!("failed to parse url");
  };

  tracing::debug!("fetching json…");
  let json = Json::get(domain, board_id, thread_id).await?;
  let thread = &json.threads[0];
  let op = &thread.posts[0];
  let post = match post_id {
    Some(id) => thread.posts.iter().find(|p| p.id == id).unwrap(),
    None => op,
  };

  #[rustfmt::skip]
  let url = format!("https://{}/{}/res/{}.html#{}",
    domain, board_id, thread_id, post_id.unwrap_or(op.id));

  let replies = thread.find_replies_to(post.id).count().plural("reply", "replies");
  let footer = format!("#{}/{} · {:#}", post.index, thread.posts.len(), replies);

  let mut edit = EditInteractionResponse::new();
  let embed = CreateEmbed::new()
    .description(&post.render(domain))
    .author(CreateEmbedAuthor::new(&op.subject).url(&url))
    .footer(CreateEmbedFooter::new(footer))
    .timestamp(Timestamp::from_unix_timestamp(post.timestamp)?);

  tracing::debug!("attaching files…");
  for file in post.files.iter().flatten() {
    let url = format!("https://{}{}", domain, file.path);
    let att = CreateAttachment::url(ctx, &url).await?;
    edit = edit.new_attachment(att);
  }

  tracing::debug!("sending response…");
  if ctx.event.edit_response(ctx, edit.embed(embed)).await.is_err() {
    err::message!("failed to send response, most likely files are too big");
  }

  Ok(())
}

// ---

#[derive(Debug, Deserialize)]
struct Json {
  threads: Vec<Thread>,
}

#[derive(Debug, Deserialize)]
struct Thread {
  posts: Vec<Post>,
}

#[derive(Debug, Deserialize)]
struct Post {
  #[serde(rename = "num")]
  id: i64,
  #[serde(rename = "number")]
  index: i64,
  timestamp: i64,
  subject: String,
  comment: String,
  files: Option<Vec<File>>,
}

#[derive(Debug, Deserialize)]
struct File {
  path: String,
}

// ---

fn parse_url(url: &str) -> Option<(&str, &str, i64, Option<i64>)> {
  static RE: OnceLock<Regex> = OnceLock::new();

  let re = RE.get_or_init(|| {
    let re = r"(?i-u)https?://([\w.]+)/(\w+)/res/(\d+)\.html(?:#(\d+))?";
    Regex::new(re).unwrap()
  });

  let captures = re.captures(url)?;
  let domain = captures.get(1).map(|b| b.as_str())?;
  let board = captures.get(2).map(|b| b.as_str())?;
  let thread = captures.get(3).and_then(|t| t.as_str().parse().ok())?;
  let post = captures.get(4).and_then(|p| p.as_str().parse().ok());
  Some((domain, board, thread, post))
}

impl Json {
  async fn get(domain: &str, board: &str, thread: i64) -> reqwest::Result<Self> {
    let url = format!("https://{domain}/{board}/res/{thread}.json");
    tracing::debug!(%url);

    let resp = reqwest::get(url).await?.error_for_status()?;
    let json = resp.json().await?;
    Ok(json)
  }
}

impl Thread {
  fn find_replies_to(&self, id: i64) -> impl Iterator<Item = &Post> {
    self.posts.iter().filter({
      let pat = format!("#{id}\"");
      move |&p| p.comment.contains(&pat)
    })
  }
}

impl Post {
  fn render(&self, domain: &str) -> String {
    fn visit(acc: &mut String, domain: &str, node: NodeRef<'_, Node>) {
      for node in node.children() {
        match node.value() {
          Node::Text(text) => acc.push_str(text),
          Node::Element(el) => 'el: {
            let name = el.name();

            if name == "a" {
              match el.attr("href") {
                Some(href) if href.starts_with('/') => {
                  acc.push('[');
                  visit(acc, domain, node);
                  acc.push_str("](https://");
                  acc.push_str(domain);
                  acc.push_str(href);
                  acc.push(')');
                }
                Some(href) => {
                  acc.push_str(href);
                }
                _ => {}
              }
              break 'el;
            }

            if name == "span" && el.has_class("unkfunc", CaseSensitive) {
              let mut tmp = String::new();
              visit(&mut tmp, domain, node);
              acc.push_str("> ");
              acc.push_str(tmp.trim_start_matches(['>', ' ']));
              break 'el;
            }

            if name == "span" && el.has_class("spoiler", CaseSensitive) {
              acc.push_str("||");
              visit(acc, domain, node);
              acc.push_str("||");
              break 'el;
            }

            if name == "s" || name == "span" && el.has_class("s", CaseSensitive) {
              acc.push_str("~~");
              visit(acc, domain, node);
              acc.push_str("~~");
              break 'el;
            }

            if name == "u" || name == "span" && el.has_class("u", CaseSensitive) {
              acc.push_str("__");
              visit(acc, domain, node);
              acc.push_str("__");
              break 'el;
            }

            if name == "b" || name == "strong" || name == "span" && el.attr("style").is_some() {
              acc.push_str("**");
              visit(acc, domain, node);
              acc.push_str("**");
              break 'el;
            }

            if name == "i" || name == "em" {
              acc.push('*');
              visit(acc, domain, node);
              acc.push('*');
              break 'el;
            }

            if name == "br" {
              acc.push('\n');
              break 'el;
            }

            visit(acc, domain, node);
          }
          _ => {}
        }
      }
    }

    let html = Html::parse_fragment(&self.comment);
    let root = html.tree.root();

    let mut acc = String::new();
    visit(&mut acc, domain, root);
    acc
  }
}
