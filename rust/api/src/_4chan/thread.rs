use ego_tree::NodeRef;
use scraper::{CaseSensitivity::*, Html, Node};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Thread {
  pub posts: Vec<Post>,
}

#[derive(Debug, Deserialize)]
pub struct Post {
  #[serde(rename = "time")]
  pub timestamp: u32,
  #[serde(rename = "no")]
  pub id: u64,
  #[serde(rename = "sub")]
  pub subject: Option<String>,
  #[serde(rename = "com")]
  pub comment: Option<String>,
  #[serde(flatten)]
  pub file: Option<File>,
}

#[derive(Debug, Deserialize)]
pub struct File {
  #[serde(rename = "tim")]
  pub id: u64,
  pub ext: String,
}

impl Thread {
  pub async fn get(board: &str, thread: u64) -> reqwest::Result<Self> {
    let url = format!("https://a.4cdn.org/{board}/thread/{thread}.json");
    let resp = reqwest::get(url).await?.error_for_status()?;
    let json = resp.json().await?;
    Ok(json)
  }

  pub fn get_post_by_id(&self, id: u64) -> Option<(&Post, usize)> {
    let index = self.posts.binary_search_by_key(&id, |p| p.id).ok()?;
    Some((&self.posts[index], index))
  }

  pub fn find_replies_to(&self, id: u64) -> impl Iterator<Item = &Post> {
    self.posts.iter().filter({
      let pat = format!("\"#p{id}\"");
      move |&p| p.comment.as_deref().is_some_and(|c| c.contains(&pat))
    })
  }
}

impl Post {
  pub fn render(&self, base: &Url) -> String {
    let mut acc = String::new();
    self.render_to(&mut acc, base);
    acc
  }

  pub fn render_to(&self, acc: &mut String, base: &Url) {
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
    visit(acc, base, root);
  }
}
