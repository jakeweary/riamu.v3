use futures::StreamExt;
use reqwest::{header, redirect, IntoUrl};

pub async fn resolve_redirect(url: &str) -> reqwest::Result<Option<String>> {
  let rp = redirect::Policy::none();
  let client = reqwest::Client::builder().redirect(rp).build()?;
  let resp = client.head(url).send().await?.error_for_status()?;
  let url = resp.headers().get(header::LOCATION);
  Ok(url.and_then(|url| Some(url.to_str().ok()?.to_owned())))
}

pub async fn download(url: impl IntoUrl) -> reqwest::Result<Vec<u8>> {
  let resp = reqwest::get(url).await?.error_for_status()?;
  let mut stream = resp.bytes_stream();
  let mut bytes = Vec::new();
  while let Some(chunk) = stream.next().await {
    bytes.extend(chunk?);
  }
  Ok(bytes)
}
