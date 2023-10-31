use reqwest::{header, multipart};
use serde::Deserialize;
use serenity::all::*;
use url::Url;

use crate::client::{err, Context, Result};

#[macros::command(description = "Upload a media file to Imgur")]
pub async fn file(ctx: &Context<'_>, file: &Attachment) -> Result<()> {
  upload(ctx, &file.url, &file.filename).await
}

#[macros::command(description = "Upload a media file to Imgur by URL")]
pub async fn url(ctx: &Context<'_>, url: &str) -> Result<()> {
  let parsed = Url::parse(url)?;
  let filename = parsed.path_segments().and_then(|s| s.last()).unwrap();
  upload(ctx, url, filename).await
}

async fn upload(ctx: &Context<'_>, url: &str, filename: &str) -> Result<()> {
  ctx.event.defer(ctx).await?;

  let resp = reqwest::get(url).await?.error_for_status()?;
  let body = reqwest::Body::wrap_stream(resp.bytes_stream());
  let image = multipart::Part::stream(body).file_name(filename.to_owned());
  let form = multipart::Form::new().part("image", image);

  let auth = format!("Client-ID {}", ctx.client.env.imgur_app_id);

  let post = reqwest::Client::builder()
    .build()?
    .post("https://api.imgur.com/3/upload")
    .header(header::AUTHORIZATION, auth)
    .multipart(form)
    .send();

  tracing::debug!("uploading…");
  let Ok(resp) = post.await?.error_for_status() else {
    err::message!("failed to upload, most likely the file is too big");
  };

  let json = resp.json::<Json>().await?;
  let url = format!("https://imgur.com/{}", json.data.id);
  tracing::debug!(%url, "uploaded");

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new().content(url);
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

#[derive(Debug, Deserialize)]
struct Json {
  data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
  id: String,
}
