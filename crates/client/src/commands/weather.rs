use lib::task;
use lib::weather::{api::Api, render::render};
use serenity::all::*;

use crate::client::{err, Context, Result};

#[macros::command(description = "Weather forecast")]
pub async fn run(
  ctx: &Context<'_>,
  #[description = "The city name, with an optional country code (e.g.: London, GB; Москва; 東京)"] location: &str,
) -> Result<()> {
  ctx.event.defer(ctx).await?;

  let api = Api::new(&ctx.client.env.openweathermap_api_key);

  tracing::info!("api: geo…");
  let loc = match api.geo(location).await {
    Ok(mut list) if !list.is_empty() => list.swap_remove(0),
    _ => err::message!("unknown location: {}", location),
  };

  tracing::info!("api: onecall…");
  let weather = api.onecall(loc.lat, loc.lon).await?;

  tracing::info!("rendering image…");
  let png = task::spawn_blocking(move || -> Result<_> {
    let mut png = Vec::new();
    render(&weather, &loc)?.write_to_png(&mut png)?;
    Ok(png)
  })
  .await??;

  let file = CreateAttachment::bytes(png, "weather.png");
  let edit = EditInteractionResponse::new().new_attachment(file);

  tracing::info!("sending response…");
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}
