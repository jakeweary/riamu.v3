use std::process::Command;

use lib::task;
use serenity::all::*;

use crate::client::{Context, Result};

#[macros::command(description = "Measure my connection speed", owner_only)]
pub async fn run(ctx: &Context<'_>) -> Result<()> {
  ctx.event.defer(ctx).await?;

  tracing::debug!("measuring…");
  let measure = task::spawn_blocking(|| {
    let mut cmd = Command::new("deps/speedtest");
    cmd.args(&["--accept-license", "--accept-gdpr", "-f", "json"]);
    cmd.output()
  });
  let output = measure.await??;
  let json = serde_json::from_slice::<json::Root>(&output.stdout)?;
  let url = format!("{}.png", json.result.url);

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new().content(url);
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

mod json {
  #[derive(serde::Deserialize)]
  pub struct Root {
    pub result: Result,
  }

  #[derive(serde::Deserialize)]
  pub struct Result {
    pub url: String,
  }
}
