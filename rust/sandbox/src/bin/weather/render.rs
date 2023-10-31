use cairo::*;
use lib::cairo::ext::ContextExt;

use crate::api::openweather as api;
use crate::Result;

mod components;
mod draw;
mod fmt;
mod icons;
mod range;
mod util;

const SCALE: i32 = 4;
const IMAGE_W: i32 = 550;
const IMAGE_H: i32 = 350;

pub fn render(city: &str, weather: api::Root) -> Result<ImageSurface> {
  let img = ImageSurface::create(Format::Rgb24, SCALE * IMAGE_W, SCALE * IMAGE_H)?;
  let ctx = Context::new(&img)?;
  ctx.scale(SCALE as f64, SCALE as f64);
  ctx.set_line_width(1.0);
  ctx.select_font_face("Roboto Flex", FontSlant::Normal, FontWeight::Normal);

  ctx.set_source_rgb_u32(0x313338);
  ctx.paint()?;

  components::current(&ctx, &weather, city)?;
  components::week(&ctx, &weather)?;
  components::hourly(&ctx, &weather)?;

  Ok(img)
}
