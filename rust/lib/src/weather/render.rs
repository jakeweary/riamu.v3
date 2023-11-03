use std::io;

use c::rsvg;
use cairo::*;

use crate::cairo::ext::ContextExt;

use super::api;

mod components;
mod draw;
mod fmt;
mod icons;
mod range;
mod util;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
  Cairo(#[from] cairo::Error),
  RSvg(#[from] rsvg::Error),
  Io(#[from] io::Error),
}

// ---

const SCALE: i32 = 8;
const IMAGE_W: i32 = 550;
const IMAGE_H: i32 = 350;

pub fn render(weather: &api::Onecall, loc: &api::geo::Location) -> Result<ImageSurface> {
  let img = ImageSurface::create(Format::Rgb24, SCALE * IMAGE_W, SCALE * IMAGE_H)?;
  let ctx = Context::new(&img)?;
  ctx.scale1(SCALE as f64);
  ctx.set_line_width(1.0);
  ctx.select_font_face("Roboto Flex", FontSlant::Normal, FontWeight::Normal);

  ctx.set_source_rgb_u32(0x313338);
  ctx.paint()?;

  components::current(&ctx, weather, loc)?;
  components::daily(&ctx, weather)?;
  components::hourly(&ctx, weather)?;

  Ok(img)
}
