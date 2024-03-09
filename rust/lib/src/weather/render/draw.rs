#![allow(dead_code)]

use std::f64::consts::{PI as π, TAU as τ};

use cairo::*;
use itertools::Itertools;

pub fn circle(ctx: &Context, x: f64, y: f64, r: f64) {
  ctx.new_sub_path();
  ctx.arc(x, y, r, 0.0, τ);
  ctx.close_path();
}

pub fn rounded_rect(ctx: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
  let m = ctx.matrix();
  ctx.translate(x, y);
  ctx.new_sub_path();
  ctx.arc(w, h, r, 0.0, 0.5 * π);
  ctx.arc(0.0, h, r, 0.5 * π, π);
  ctx.arc(0.0, 0.0, r, π, 1.5 * π);
  ctx.arc(w, 0.0, r, 1.5 * π, 0.0);
  ctx.close_path();
  ctx.set_matrix(m);
}

pub fn arrow(ctx: &Context, x: f64, y: f64, scale: f64, angle: f64) {
  let (sin, cos) = (τ / 12.0).sin_cos();
  let m = ctx.matrix();
  ctx.translate(x, y);
  ctx.scale(scale, scale);
  ctx.rotate(angle);
  ctx.new_sub_path();
  ctx.line_to(0.0, 0.25 - cos);
  ctx.line_to(sin, -cos);
  ctx.line_to(0.0, 1.0);
  ctx.line_to(-sin, -cos);
  ctx.close_path();
  ctx.set_matrix(m);
}

// TODO: probably should implement some nicer spline
// potentially better options: constrained cubic, monotonic, natural, etc.
// current impl is cardinal spline (i think)
//
// useful links:
// http://scaledinnovation.com/analytics/splines/aboutSplines.html
// http://using-d3js.com/05_04_curves.html
// https://ibiblio.org/e-notes/Splines/cardinal.html
// https://codeplea.com/introduction-to-splines
// https://particleincell.com/2012/bezier-splines/
// https://alglib.net/interpolation/spline3.php
// https://deriscope.com/docs/Kruger_CubicSpline.pdf
// https://en.wikipedia.org/wiki/Spline_interpolation#Algorithm_to_find_the_interpolating_cubic_spline
// https://en.wikipedia.org/wiki/Spline_(mathematics)#Algorithm_for_computing_natural_cubic_splines
// https://en.wikipedia.org/wiki/Monotone_cubic_interpolation#Example_implementation
//
pub fn spline<T>(
  ctx: &Context,
  t: f64,  // curve tension
  e: f64,  // extend curve beyond terminal points
  x: f64,  // position x
  y: f64,  // position y
  sx: f64, // scale x
  sy: f64, // scale y
  items: &[T],
  map: impl Fn(&T) -> f64,
) {
  let n = items.len();
  let m = ctx.matrix();
  ctx.translate(x, y);
  ctx.scale(sx, sy);

  let tangent = |(y0, y1, y2)| (0.5 * (y2 - y0), y0, y1, y2);
  let tangents = items.iter().map(&map).tuple_windows().map(tangent);

  let (x0, x0ʹ, x1, x1ʹ) = (0.0, 1.0, n as f64 - 1.0, n as f64 - 2.0);
  let ([y0, y0ʹ, y0ʺ], [y1, y1ʹ, y1ʺ]) = (&items[..3], &items[n - 3..]) else {
    unreachable!()
  };
  let (dy0, y0, y0ʹ, _) = tangent((map(y0), map(y0ʹ), map(y0ʺ)));
  let (dy1, _, y1ʹ, y1) = tangent((map(y1), map(y1ʹ), map(y1ʺ)));
  let (dy0ʹ, dy1ʹ) = (y0ʹ - y0, y1 - y1ʹ);

  ctx.new_sub_path();
  ctx.move_to(x0 - e, y0 - e * dy0ʹ);
  ctx.line_to(x0, y0);
  ctx.curve_to(x0 + t, y0 + t * dy0ʹ, x0ʹ - t, y0ʹ - t * dy0, x0ʹ, y0ʹ);
  for (i, pair) in tangents.tuple_windows().enumerate() {
    let (x0, x1) = (1.0 + i as f64, 2.0 + i as f64);
    let ((dy0, _, y0, _), (dy1, _, y1, _)) = pair;
    ctx.curve_to(x0 + t, y0 + t * dy0, x1 - t, y1 - t * dy1, x1, y1);
  }
  ctx.curve_to(x1ʹ + t, y1ʹ + t * dy1, x1 - t, y1 - t * dy1ʹ, x1, y1);
  ctx.line_to(x1 + e, y1 + e * dy1ʹ);

  ctx.set_matrix(m);
}

// previous version, keeping it just in case
//
// pub fn spline<T>(ctx: &Context, step: f64, items: &[T], map: impl Fn(&T) -> f64) {
//   let m = ctx.matrix();
//   ctx.scale(step, 1.0);
//   ctx.move_to(0.0, map(&items[0]));
//
//   let t = 1.0 / 3.0; // tension
//   let mut dy0 = None;
//
//   for i in 0..items.len() - 1 {
//     let x0 = i as f64;
//     let x1 = x0 + 1.0;
//
//     let y0 = map(&items[i]);
//     let y1 = map(&items[i + 1]);
//     let y2 = items.get(i + 2).map(&map);
//
//     let dy1 = y2.map(|y2| 0.5 * t * (y2 - y0));
//     let dy0 = mem::replace(&mut dy0, dy1);
//
//     let (dx0, dy0) = dy0.map_or((0.0, 0.0), |dy0| (t, dy0));
//     let (dx1, dy1) = dy1.map_or((0.0, 0.0), |dy1| (t, dy1));
//
//     ctx.curve_to(x0 + dx0, y0 + dy0, x1 - dx1, y1 - dy1, x1, y1);
//   }
//
//   ctx.set_matrix(m);
// }
