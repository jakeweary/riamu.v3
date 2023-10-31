use std::time::Duration;

use futures::StreamExt;
use lib::fmt::num::Format;
use serde::Deserialize;
use serde_json::json;
use serenity::all::*;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::cache::Name;
use crate::client::{err, Context, Result};

#[macros::command(description = "Download a video from TikTok")]
pub async fn run(ctx: &Context<'_>, url: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  let json = Json::get(url).await?;
  let Some(data) = json.data else {
    err::message!("something went wrong");
  };

  tracing::debug!("selecting quality…");
  let file_url = select_quality(ctx, &data).await?;

  tracing::debug!("downloading…");
  ctx.progress("downloading…").await?;
  let Ok(resp) = reqwest::get(file_url).await?.error_for_status() else {
    err::message!("failed to download");
  };

  let tempdir = tempfile::tempdir()?;
  let fname = format!("{}.mp4", data.id);
  let fpath = tempdir.path().join(fname);

  let mut bytes = resp.bytes_stream();
  let mut file = File::create(&fpath).await?;
  while let Some(bytes) = bytes.next().await {
    file.write_all(&bytes?).await?;
  }
  file.flush().await?;

  let fsize = file.metadata().await?.len();
  tracing::debug!("downloaded {}B", fsize.iec());

  let content = format! {
    "[TikTok](<https://tiktok.com/@{}/video/{}>) by [{}](<https://tiktok.com/@{}>)",
    data.author.id, data.id, data.author.nickname, data.author.unique_id
  };

  if fsize > ctx.filesize_limit().await? {
    tracing::debug!("caching…");
    let url = ctx.client.cache.store_file(&fpath, Name::Keep).await?.unwrap();

    let content = format!("{} \u{205D} [mp4]({}) {}B", content, url, fsize.iec());
    let edit = EditInteractionResponse::new()
      .components(Default::default()) // remove components
      .content(content);

    tracing::debug!("sending response…");
    ctx.event.edit_response(ctx, edit).await?;
  } else {
    let file = CreateAttachment::path(&fpath).await?;
    let edit = EditInteractionResponse::new()
      .components(Default::default()) // remove components
      .content(content)
      .new_attachment(file);

    tracing::debug!("uploading…");
    ctx.progress("uploading…").await?;
    if ctx.event.edit_response(ctx, edit).await.is_err() {
      err::message!("failed to upload, most likely the file is too big");
    }
  }

  Ok(())
}

async fn select_quality<'a>(ctx: &Context<'_>, data: &'a Data) -> Result<&'a str> {
  let play = format!("{}B", data.size.iec());
  let hdplay = format!("{}B", data.hd_size.iec());
  let wmplay = format!("{}B with watermark", data.wm_size.iec());

  let buttons = CreateActionRow::Buttons({
    let values = [("hdplay", hdplay), ("play", play), ("wmplay", wmplay)];
    let buttons = values.into_iter().enumerate().map(|(i, (id, label))| {
      let style = match i {
        2 => ButtonStyle::Secondary,
        _ => ButtonStyle::Primary,
      };
      CreateButton::new(id).label(label).style(style)
    });
    buttons.collect()
  });

  let components = vec![buttons];
  let edit = EditInteractionResponse::new().components(components);
  let msg = ctx.event.edit_response(ctx, edit).await?;

  tracing::debug!("waiting for user interaction…");
  let mut collector = msg
    .await_component_interaction(ctx)
    .author_id(ctx.event.user.id)
    .timeout(Duration::from_secs(60))
    .stream();

  let Some(mci) = collector.next().await else {
    err::timeout!();
  };

  mci.defer(ctx).await?;

  let url = match &*mci.data.custom_id {
    "play" => &data.play,
    "wmplay" => &data.wmplay,
    "hdplay" => &data.hdplay,
    _ => unreachable!(),
  };

  Ok(url)
}

// ---

impl Json {
  async fn get(url: &str) -> reqwest::Result<Self> {
    let form = json!({ "url": url, "hd": 1 });
    let client = reqwest::Client::builder().build()?;
    let post = client.post("https://tikwm.com/api/").form(&form);
    let resp = post.send().await?.error_for_status()?;
    resp.json().await
  }
}

// ---

#[derive(Debug, Deserialize)]
struct Json {
  data: Option<Data>,
}

#[derive(Debug, Deserialize)]
struct Data {
  author: Author,
  id: String,
  play: String,
  wmplay: String,
  hdplay: String,
  size: u64,
  wm_size: u64,
  hd_size: u64,
}

#[derive(Debug, Deserialize)]
struct Author {
  id: String,
  unique_id: String,
  nickname: String,
}

// ---

// async fn extract_id(url: &str) -> reqwest::Result<Option<u64>> {
//   match parse_id(url) {
//     Some(id) => Ok(Some(id)),
//     _ => {
//       let url = resolve_redirect(url).await?;
//       let id = url.and_then(|url| parse_id(&url));
//       Ok(id)
//     }
//   }
// }

// fn parse_id(url: &str) -> Option<u64> {
//   tracing::debug!(%url, "parsing…");
//   let parsed = Url::parse(url).ok()?;
//   let last = parsed.path_segments()?.last()?;
//   let id = last.parse().ok()?;
//   tracing::debug!(id, "parsed");
//   Some(id)
// }
