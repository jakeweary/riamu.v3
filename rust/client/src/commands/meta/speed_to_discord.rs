use std::fmt::Write;
use std::time::Instant;

use fmt::num::Format as _;
use futures::StreamExt;
use random::{Random, XorShift64};
use serenity::all::*;

use crate::client::{Context, Result};

#[macros::command(desc = "Measure my connection speed to Discord servers")]
pub async fn run(ctx: &Context<'_>) -> Result<()> {
  ctx.event.defer(ctx).await?;

  let n_bytes = ctx.filesize_limit().await? - 512;

  let mut buffer = vec![0; n_bytes as usize];
  let mut rng = XorShift64::from_time();
  rng.fill(&mut buffer);

  tracing::debug!("uploading…");
  let upload = Instant::now();
  let att = CreateAttachment::bytes(buffer, "nudes.rar");
  let edit = EditInteractionResponse::new().new_attachment(att);
  let msg = ctx.event.edit_response(ctx, edit).await?;
  let upload = upload.elapsed().as_secs_f64();

  tracing::debug!("downloading…");
  let download = Instant::now();
  let resp = reqwest::get(&msg.attachments[0].url).await?.error_for_status()?;
  let mut stream = resp.bytes_stream();
  while let Some(bytes) = stream.next().await {
    bytes?;
  }
  let download = download.elapsed().as_secs_f64();

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new()
    .clear_attachments()
    .content(content(n_bytes, upload, download)?);
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

fn content(bytes: u64, upload: f64, download: f64) -> fmt::Result<String> {
  #[rustfmt::skip]
  let line = |acc: &mut dyn Write, label: &str, bytes_per_second: f64| {
    let si = (8.0 * bytes_per_second).si();
    let iec = bytes_per_second.iec();
    writeln!(acc, "{}: `{}` {}bit/s (`{}` {}B/s)",
      label, si.norm, si.prefix, iec.norm, iec.prefix)
  };

  let mut acc = String::new();
  line(&mut acc, "Me → Discord", bytes as f64 / upload)?;
  line(&mut acc, "Me ← Discord", bytes as f64 / download)?;
  Ok(acc)
}
