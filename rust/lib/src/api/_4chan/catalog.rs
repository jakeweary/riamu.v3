use rand::prelude::*;
use regex::RegexBuilder;
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
  #[serde(rename = "sub")]
  pub subject: Option<String>,
  #[serde(rename = "com")]
  pub comment: Option<String>,
}

impl Catalog {
  pub async fn get(board_id: &str) -> reqwest::Result<Self> {
    let url = format!("https://a.4cdn.org/{board_id}/catalog.json");
    let resp = reqwest::get(url).await?.error_for_status()?;
    let pages = resp.json().await?;
    Ok(Self { pages })
  }

  pub fn random<F, W>(&self, filter: F, weight: W) -> Option<(&Thread, usize)>
  where
    F: Fn(&Thread) -> bool,
    W: Fn(&Thread) -> usize,
  {
    let threads = || self.pages.iter().flat_map(|p| &p.threads).filter(|&t| filter(t));

    let board_total = threads().map(&weight).sum();
    let board_index = match board_total {
      0 => return None,
      n => thread_rng().gen_range(0..n),
    };

    let (thread, board_subtotal) = threads()
      .scan(0, |acc, thread| {
        *acc += weight(&thread);
        Some((thread, *acc))
      })
      .find(|&(_, acc)| board_index < acc)
      .unwrap();

    let thread_total = weight(&thread);
    let thread_index = board_index + thread_total - board_subtotal;

    tracing::debug! {
      "random pick: #{}/{} (#{}/{})",
      1 + board_index, board_total,
      1 + thread_index, thread_total,
    }

    Some((thread, thread_index))
  }
}

// ---

type RegexResult<T> = Result<T, regex::Error>;

pub fn thread_filter(include: Option<&str>, exclude: Option<&str>) -> RegexResult<impl Fn(&Thread) -> bool> {
  let is_included = regex_matcher(include, true)?;
  let is_excluded = regex_matcher(exclude, false)?;

  Ok(move |t: &Thread| {
    let subject = t.subject.as_deref().unwrap_or_default();
    let included = is_included(subject);
    let excluded = is_excluded(subject);
    included && !excluded
  })
}

fn regex_matcher(pattern: Option<&str>, default: bool) -> RegexResult<impl Fn(&str) -> bool> {
  let re = match pattern {
    Some(pat) => Some(RegexBuilder::new(pat).case_insensitive(true).build()?),
    None => None,
  };

  Ok(move |input: &str| match &re {
    Some(re) => re.is_match(input),
    None => default,
  })
}
