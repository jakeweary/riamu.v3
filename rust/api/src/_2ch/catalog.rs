use serde::Deserialize;

use crate::{random, regex};

#[derive(Debug, Deserialize)]
pub struct Catalog {
  pub threads: Vec<Thread>,
}

#[derive(Debug, Deserialize)]
pub struct Thread {
  #[serde(rename = "num")]
  pub id: u64,
  pub posts_count: u32,
  pub files_count: u32,
  pub subject: String,
  pub comment: String,
  pub tags: String,
}

impl Catalog {
  pub async fn get(board_id: &str) -> reqwest::Result<Self> {
    let url = format!("https://2ch.hk/{board_id}/catalog.json");
    let resp = reqwest::get(url).await?.error_for_status()?;
    resp.json().await
  }

  pub fn random<F, W>(&self, filter: F, weight: W) -> Option<(&Thread, usize)>
  where
    F: Fn(&Thread) -> bool,
    W: Fn(&Thread) -> usize,
  {
    let r = random::weighted(|| &self.threads, filter, weight)?;
    Some((r.item, r.local_index))
  }
}

pub fn thread_filter(include: Option<&str>, exclude: Option<&str>) -> regex::Result<impl Fn(&Thread) -> bool> {
  let is_included = regex::matcher(include, true)?;
  let is_excluded = regex::matcher(exclude, false)?;

  Ok(move |t: &Thread| {
    let included = is_included(&t.subject);
    let excluded = is_excluded(&t.subject);
    included && !excluded
  })
}
