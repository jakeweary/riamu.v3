use ego_tree::NodeRef;
use scraper::{CaseSensitivity::*, Html, Node};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Threads {
  pub threads: Vec<Thread>,
}

#[derive(Debug, Deserialize)]
pub struct Thread {
  pub posts: Vec<Post>,
}

#[derive(Debug, Deserialize)]
pub struct Post {
  pub timestamp: u32,
  #[serde(rename = "num")]
  pub id: u64,
  pub subject: String,
  pub comment: String,
  pub files: Option<Vec<File>>,
}

#[derive(Debug, Deserialize)]
pub struct File {
  pub path: String,
}

impl Thread {
  pub async fn get(domain: &str, board: &str, thread: u64) -> reqwest::Result<Self> {
    let url = format!("https://{domain}/{board}/res/{thread}.json");
    let resp = reqwest::get(url).await?.error_for_status()?;
    let Threads { mut threads } = resp.json().await?;
    Ok(threads.swap_remove(0))
  }

  pub fn get_post_by_id(&self, id: u64) -> Option<(&Post, usize)> {
    let index = self.posts.binary_search_by_key(&id, |p| p.id).ok()?;
    Some((&self.posts[index], index))
  }

  pub fn find_replies_to(&self, id: u64) -> impl Iterator<Item = &Post> {
    self.posts.iter().filter({
      let pat = format!("#{id}\"");
      move |&p| p.comment.contains(&pat)
    })
  }
}

impl Post {
  pub fn render(&self, domain: &str) -> String {
    let mut acc = String::new();
    self.render_to(&mut acc, domain);
    acc
  }

  pub fn render_to(&self, acc: &mut String, domain: &str) {
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
    visit(acc, domain, root);
  }
}
