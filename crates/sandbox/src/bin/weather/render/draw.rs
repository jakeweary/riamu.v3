#![allow(dead_code)]

use std::f64::consts::{PI as π, TAU as τ};
use std::fmt::Arguments;
use std::mem;

use cairo::*;
use lib::cairo::ext::ContextExt;
// use pango::prelude::*;

// pub fn markup(ctx: &Context, markup: &str) {
//   let map = pangocairo::FontMap::default();
//   let pc = map.create_context();
//   let layout = pango::Layout::new(&pc);
//   layout.set_markup(&markup);
//   pangocairo::update_layout(ctx, &layout);
//   pangocairo::show_layout(ctx, &layout);
// }

pub fn colored_text(ctx: &Context, pairs: &[(u32, Arguments<'_>)]) -> cairo::Result<()> {
  for &(color, arg) in pairs {
    ctx.set_source_rgb_u32(color);
    match arg.as_str() {
      Some(arg) => ctx.show_text(arg)?,
      None => ctx.show_text(&arg.to_string())?,
    }
  }
  Ok(())
}

pub fn circle(ctx: &Context, x: f64, y: f64, r: f64) {
  ctx.new_sub_path();
  ctx.arc(x, y, r, 0.0, τ);
  ctx.close_path();
}

pub fn arrow(ctx: &Context) {
  let (sin, cos) = (τ / 12.0).sin_cos();
  ctx.new_sub_path();
  ctx.line_to(0.0, 0.25 - cos);
  ctx.line_to(sin, -cos);
  ctx.line_to(0.0, 1.0);
  ctx.line_to(-sin, -cos);
  ctx.close_path();
}

pub fn rounded_rect(ctx: &Context, w: f64, h: f64, r: f64) {
  ctx.new_sub_path();
  ctx.arc(w, h, r, 0.0, 0.5 * π);
  ctx.arc(0.0, h, r, 0.5 * π, π);
  ctx.arc(0.0, 0.0, r, π, 1.5 * π);
  ctx.arc(w, 0.0, r, 1.5 * π, 0.0);
  ctx.close_path();
}

// http://scaledinnovation.com/analytics/splines/aboutSplines.html
pub fn spline<T>(ctx: &Context, step: f64, items: &[T], map: impl Fn(&T) -> f64) {
  let m = ctx.matrix();
  ctx.scale(step, 1.0);
  ctx.move_to(0.0, map(&items[0]));

  let t = 1.0 / 3.0; // tension
  let mut dy0 = None;

  for i in 0..items.len() - 1 {
    let x0 = i as f64;
    let x1 = x0 + 1.0;

    let y0 = map(&items[i]);
    let y1 = map(&items[i + 1]);
    let y2 = items.get(i + 2).map(&map);

    let dy1 = y2.map(|y2| 0.5 * t * (y2 - y0));
    let dy0 = mem::replace(&mut dy0, dy1);

    let (dx0, dy0) = dy0.map_or((0.0, 0.0), |dy0| (t, dy0));
    let (dx1, dy1) = dy1.map_or((0.0, 0.0), |dy1| (t, dy1));

    ctx.curve_to(x0 + dx0, y0 + dy0, x1 - dx1, y1 - dy1, x1, y1);
  }

  ctx.set_matrix(m);
}
