use serenity::all::*;
use util::task;
use weather::{api::Api, render::render};

use crate::client::{command, err, Context, Result};

#[command(desc = "Weather forecast")]
pub async fn run(
  ctx: &Context<'_>,
  #[desc = "The city name, with an optional country code (e.g.: London, GB; Москва; 東京)"] location: &str,
) -> Result<()> {
  ctx.event.defer(ctx).await?;

  let api = Api::new(&ctx.client.env.openweathermap_api_key);

  tracing::debug!("api: geo…");
  let loc = match api.geo(location).await {
    Ok(mut list) if !list.is_empty() => list.swap_remove(0),
    _ => err::message!("unknown location: {}", location),
  };

  tracing::debug!("api: onecall…");
  let weather = api.onecall(loc.lat, loc.lon).await?;

  tracing::debug!("rendering image…");
  let png = task::spawn_blocking(move || -> Result<_> {
    let mut png = Vec::new();
    render(&weather, &loc)?.write_to_png(&mut png)?;
    Ok(png)
  })
  .await??;

  let file = CreateAttachment::bytes(png, "weather.png");
  let edit = EditInteractionResponse::new().new_attachment(file);

  tracing::debug!("sending response…");
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}
