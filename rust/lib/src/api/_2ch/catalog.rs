use rand::prelude::*;
use serde::Deserialize;

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
}

impl Catalog {
  pub async fn get(board_id: &str) -> reqwest::Result<Self> {
    let url = format!("https://2ch.hk/{board_id}/catalog.json");
    let resp = reqwest::get(url).await?.error_for_status()?;
    Ok(resp.json().await?)
  }

  pub fn random(&self, f: impl Fn(&Thread) -> usize) -> (&Thread, usize) {
    let threads = || self.threads.iter();
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
