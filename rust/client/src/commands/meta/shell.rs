use std::process::Command;
use std::{fs, io};

use serenity::all::*;
use util::task;

use crate::client::{err, Context, Result};

#[macros::command(desc = "Run a shell command (owner only)", owner_only)]
pub async fn run(ctx: &Context<'_>, command: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  let tempdir = tempfile::tempdir()?;

  let mut cmd = Command::new("bash");
  cmd.current_dir(&tempdir).args(["-c", command]);

  tracing::debug!("running a command…");
  let output = task::spawn_blocking(move || cmd.output()).await??;

  let files = fs::read_dir(tempdir.path())?
    .map(|e| Ok(e?.path()))
    .collect::<io::Result<Vec<_>>>()?;

  let mut edit = EditInteractionResponse::new();

  for path in files {
    let file = CreateAttachment::path(path).await?;
    edit = edit.new_attachment(file);
  }

  let stdout = ("stdout.txt", &output.stdout[..]);
  let stderr = ("stderr.txt", &output.stderr[..]);
  let files = [stdout, stderr];

  for (name, bytes) in files.into_iter().filter(|(_, b)| !b.is_empty()) {
    let file = CreateAttachment::bytes(bytes, name);
    edit = edit.new_attachment(file);
  }

  tracing::debug!("sending response…");
  if ctx.event.edit_response(ctx, edit).await.is_err() {
    err::message!("failed to send response, most likely files are too big");
  }

  Ok(())
}
