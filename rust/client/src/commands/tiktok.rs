use std::time::Duration;

use futures::StreamExt;
use lib::discord::link::{LinkName, LinkUrl};
use lib::fmt::num::Format;
use serde::Deserialize;
use serde_json::json;
use serenity::all::*;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::cache::Name;
use crate::client::{err, Context, Result};

#[macros::command(desc = "Download a video from TikTok")]
pub async fn run(ctx: &Context<'_>, url: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("fetching json…");
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
    data.author.id, data.id, LinkName(&data.author.nickname), data.author.unique_id
  };

  if fsize > ctx.filesize_limit().await? {
    tracing::debug!("caching…");
    let url = ctx.client.cache.store_file(fpath, Name::Keep).await?.unwrap();
    let url = LinkUrl(url.as_str());

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
  let buttons = CreateActionRow::Buttons({
    let play = format!("{}B ", data.size.iec());
    let hdplay = format!("{}B source quality", data.hd_size.iec());
    // let wmplay = format!("{}B with watermark", data.wm_size.iec());

    let values = [
      ("play", play, ButtonStyle::Primary),
      ("hdplay", hdplay, ButtonStyle::Secondary),
      // ("wmplay", wmplay, ButtonStyle::Secondary),
    ];

    values
      .into_iter()
      .map(|(id, label, style)| CreateButton::new(id).label(label).style(style))
      .collect()
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
    "hdplay" => &data.hdplay,
    // "wmplay" => &data.wmplay,
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
  size: u64,
  hdplay: String,
  hd_size: u64,
  // wmplay: String,
  // wm_size: u64,
}

#[derive(Debug, Deserialize)]
struct Author {
  id: String,
  unique_id: String,
  nickname: String,
}

// ---

// fn normalize_url(url: &str) -> Option<String> {
//   static RE: OnceLock<Regex> = OnceLock::new();

//   let re = RE.get_or_init(|| {
//     let pattern = r"(?x)
//       tiktok\.com/@(?<name>\w+)/video/(?<id>\d+) |
//       tiktok\.com/(?:t/)?(?<short>\w+) |
//       ^(?<just_id>\d+)$
//     ";
//     Regex::new(pattern).unwrap()
//   });

//   let caps = re.captures(url)?;

//   if let Some(short) = caps.name("short") {
//     Some(format!("https://vm.tiktok.com/{}", short.as_str()))
//   } else if let (Some(name), Some(id)) = (caps.name("name"), caps.name("id")) {
//     Some(format!("https://tiktok.com/@{}/video/{}", name.as_str(), id.as_str()))
//   } else if let Some(id) = caps.name("just_id") {
//     Some(id.as_str().into())
//   } else {
//     None
//   }
// }

// async fn extract_id(url: &str) -> reqwest::Result<Option<u64>> {
//   let parse = |url: &str| {
//     let parsed = Url::parse(url).ok()?;
//     let last = parsed.path_segments()?.last()?;
//     Some(last.parse().ok()?)
//   };

//   match parse(url) {
//     Some(id) => Ok(Some(id)),
//     _ => {
//       let url = lib::network::resolve_redirect(url).await?;
//       let id = url.and_then(|url| parse(&url));
//       Ok(id)
//     }
//   }
// }
