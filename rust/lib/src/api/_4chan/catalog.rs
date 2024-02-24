use rand::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Catalog {
  pub pages: Vec<Page>,
}

#[derive(Debug, Deserialize)]
pub struct Page {
  pub page: u32,
  pub threads: Vec<Thread>,
}

#[derive(Debug, Deserialize)]
pub struct Thread {
  #[serde(rename = "no")]
  pub id: u64,
  pub replies: u32,
  pub images: u32,
}

impl Catalog {
  pub async fn get(board_id: &str) -> reqwest::Result<Self> {
    let url = format!("https://a.4cdn.org/{board_id}/catalog.json");
    let resp = reqwest::get(url).await?.error_for_status()?;
    let pages = resp.json().await?;
    Ok(Self { pages })
  }

  pub fn random(&self, f: impl Fn(&Thread) -> usize) -> (&Thread, usize) {
    let threads = || self.pages.iter().flat_map(|page| &page.threads);
    let board_total = threads().map(&f).sum();
    let board_index = thread_rng().gen_range(0..board_total);

    let (thread, board_subtotal) = threads()
      .scan(0, |acc, thread| {
        *acc += f(&thread);
        Some((thread, *acc))
      })
      .find(|&(_, acc)| board_index < acc)
      .unwrap();

    let thread_total = f(&thread);
    let thread_index = board_index + thread_total - board_subtotal;
    (thread, thread_index)
  }
}
