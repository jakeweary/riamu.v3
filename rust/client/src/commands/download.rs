use std::borrow::Cow;
use std::time::Duration;
use std::{fs, mem};

use futures::StreamExt;
use lib::{fmt, fmt::num::Format as F, task};
use python::lib::dl::{self, *};
use serenity::all::*;
use url::Url;

use crate::cache::Name;
use crate::client::{err, Context, Result};

#[macros::command(description = "Download a media file from YouTube, Twitch, Twitter, etc.")]
pub async fn run(
  ctx: &Context<'_>,
  #[description = "A YouTube search query or a link to something"] query: &str,
) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("converting query to url…");
  let url = query_to_url(ctx, query).await?;

  tracing::debug!("initializing donwloader…");
  let tempdir = tempfile::tempdir()?;
  let dl = Downloader::new(url, &tempdir);

  tracing::debug!("extracting formats…");
  ctx.progress("extracting formats…").await?;
  let Ok(dl_ctx) = dl.context.await else {
    err::message!("failed to extract formats");
  };
  tracing::debug!("extracted {} formats", dl_ctx.formats.len());

  let format_ids = select_format_ids(ctx, &dl_ctx.formats).await?;
  tracing::debug!("selected {} formats", format_ids.len());

  tracing::debug!("downloading…");
  ctx.progress("downloading…").await?;
  dl.selected.send(format_ids).unwrap();
  let info = dl.finish.await??;

  let Some(Ok(file)) = fs::read_dir(&tempdir)?.next() else {
    err::message!("failed to download");
  };
  let fpath = file.path();
  let fname = fpath.file_name().unwrap().to_string_lossy();
  let fsize = fpath.metadata()?.len();
  tracing::debug!(file = %fname, "downloaded {}B", fsize.iec());

  let content = format!("[{}](<{}>)", info.title, info.webpage_url);

  if fsize > ctx.filesize_limit().await? {
    let ext = fpath.extension().and_then(|e| e.to_str()).unwrap();
    let name = format!("{}.{}", info.title, ext);

    tracing::debug!("caching…");
    let url = ctx.client.cache.store_file(&fpath, Name::Set(&name)).await?.unwrap();

    let content = format!("{} \u{205D} [{}]({}) {}B", content, ext, url, fsize.iec());
    let edit = EditInteractionResponse::new()
      .components(Default::default()) // remove components
      .content(content);

    tracing::debug!("sending response…");
    ctx.event.edit_response(ctx, edit).await?;
  } else {
    let attachment = CreateAttachment::path(&fpath).await?;
    let edit = EditInteractionResponse::new()
      .components(Default::default()) // remove components
      .content(content)
      .new_attachment(attachment);

    tracing::debug!("uploading…");
    ctx.progress("uploading…").await?;
    if ctx.event.edit_response(ctx, edit).await.is_err() {
      err::message!("failed to upload, most likely the file is too big ({fsize}B)");
    }
  }

  Ok(())
}

async fn query_to_url<'a>(ctx: &Context<'_>, query: &'a str) -> Result<Cow<'a, str>> {
  match Url::parse(query) {
    Ok(_) => Ok(Cow::Borrowed(query)),
    Err(_) => {
      tracing::debug!("searching…");
      let query = query.to_owned();
      let videos = task::spawn_blocking(move || dl::search(&query)).await??;
      tracing::debug!("found {} videos", videos.len());

      tracing::debug!("selecting video url…");
      let url = select_video_url(ctx, &videos).await?;
      tracing::debug!(%url, "selected");
      Ok(Cow::Owned(url))
    }
  }
}

async fn select_video_url(ctx: &Context<'_>, videos: &[search::Result]) -> Result<String> {
  if videos.len() == 1 {
    return Ok(videos[0].url.clone());
  }

  let selector = CreateActionRow::SelectMenu({
    let options = videos.iter().take(25).map(|video| {
      let label = fmt::ellipsis(&video.title, 100);
      let mut option = CreateSelectMenuOption::new(label, &video.url);
      if let Some(channel) = &video.channel {
        option = option.description(fmt::ellipsis(channel, 100));
      }
      option
    });

    let options = options.collect();
    let menu = CreateSelectMenuKind::String { options };
    let menu = CreateSelectMenu::new("url", menu);
    menu.placeholder("Select video")
  });

  let components = vec![selector];
  let edit = EditInteractionResponse::new().components(components);
  let msg = ctx.event.edit_response(ctx, edit).await?;

  tracing::debug!("waiting for user interaction…");
  let mut collector = msg
    .await_component_interaction(ctx)
    .author_id(ctx.event.user.id)
    .timeout(Duration::from_secs(60))
    .stream();

  let Some(mut mci) = collector.next().await else {
    err::timeout!();
  };

  mci.defer(ctx).await?;

  let url = match &mut mci.data.kind {
    ComponentInteractionDataKind::StringSelect { values } => values.swap_remove(0),
    _ => unreachable!(),
  };

  Ok(url)
}

async fn select_format_ids(ctx: &Context<'_>, formats: &[download::Format]) -> Result<Vec<String>> {
  let formats = formats.iter().rev().collect::<Vec<_>>();
  let videos = formats.iter().filter(|f| f.is_video()).collect::<Vec<_>>();
  let audios = formats.iter().filter(|f| f.is_audio()).collect::<Vec<_>>();

  if videos.is_empty() || audios.is_empty() {
    return Ok(vec![formats[0].format_id.clone()]);
  }

  let video_selector = CreateActionRow::SelectMenu({
    let options = videos.into_iter().take(25).map(|&fmt| {
      let size = fmt.size().unwrap_or(0).iec();
      let vcodec = fmt.vcodec.as_deref().unwrap_or("unknown");
      let desc = format!("{}B · {} · {}", size, fmt.ext, vcodec);

      let label = fmt::ellipsis(&fmt.format, 100);
      let desc = fmt::ellipsis(&desc, 100);
      CreateSelectMenuOption::new(label, &fmt.format_id).description(desc)
    });

    let options = options.collect();
    let menu = CreateSelectMenuKind::String { options };
    let menu = CreateSelectMenu::new("video", menu);
    menu.placeholder("Select video format")
  });

  let audio_selector = CreateActionRow::SelectMenu({
    let options = audios.into_iter().take(25).map(|&fmt| {
      let size = fmt.size().unwrap_or(0).iec();
      let acodec = fmt.acodec.as_deref().unwrap_or("unknown");
      let desc = format!("{}B · {} · {}", size, fmt.ext, acodec);

      let label = fmt::ellipsis(&fmt.format, 100);
      let desc = fmt::ellipsis(&desc, 100);
      CreateSelectMenuOption::new(label, &fmt.format_id).description(desc)
    });

    let options = options.collect();
    let menu = CreateSelectMenuKind::String { options };
    let menu = CreateSelectMenu::new("audio", menu);
    menu.placeholder("Select audio format")
  });

  let components = vec![video_selector, audio_selector];
  let edit = EditInteractionResponse::new().components(components);
  let msg = ctx.event.edit_response(ctx, edit).await?;

  let mut video_id = None;
  let mut audio_id = None;

  tracing::debug!("waiting for user interaction…");
  let mut collector = msg
    .await_component_interaction(ctx)
    .author_id(ctx.event.user.id)
    .timeout(Duration::from_secs(60))
    .stream();

  loop {
    let Some(mut mci) = collector.next().await else {
      err::timeout!();
    };

    mci.defer(ctx).await?;

    let key = &mci.data.custom_id;
    let value = match &mut mci.data.kind {
      ComponentInteractionDataKind::StringSelect { values } => values.swap_remove(0),
      _ => unreachable!(),
    };
    tracing::debug!(id = %value, "selected {}", key);

    match key.as_ref() {
      "video" => video_id = Some(value),
      "audio" => audio_id = Some(value),
      _ => unreachable!(),
    }

    if let (Some(video), Some(audio)) = (&mut video_id, &mut audio_id) {
      let video = mem::take(video);
      let audio = mem::take(audio);
      tracing::debug!(%video, %audio, "selected formats");

      return Ok(vec![video, audio]);
    }
  }
}
