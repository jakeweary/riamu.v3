use std::f64::consts::{PI as π, TAU as τ};

use cairo::*;

pub trait ContextExt {
  fn scale1(&self, s: f64);
  fn set_source_rgb_u8(&self, r: u8, g: u8, b: u8);
  fn set_source_rgb_u8_and_alpha(&self, r: u8, g: u8, b: u8, a: f64);
  fn set_source_rgba_u8(&self, r: u8, g: u8, b: u8, a: u8);
  fn set_source_rgb_u32(&self, rgb: u32);
  fn set_source_rgb_u32_and_alpha(&self, rgb: u32, a: f64);
  fn set_source_rgba_u32(&self, argb: u32);
  fn circle(&self, x: f64, y: f64, r: f64);
  fn rounded_rect(&self, x: f64, y: f64, w: f64, h: f64, r: f64);
  fn rounded_rect_smooth(&self, x: f64, y: f64, w: f64, h: f64, r: f64, t: f64);
  fn rounded_rect_figma(&self, x: f64, y: f64, w: f64, h: f64, r: f64, t: f64);
  fn rounded_rect_ios7(&self, x: f64, y: f64, w: f64, h: f64, r: f64);
  fn text_width(&self, parts: &[&str]) -> Result<f64>;
  fn map_path<F: Fn(f64, f64) -> (f64, f64)>(&self, map: F) -> Result<()>;
}

impl ContextExt for Context {
  fn scale1(&self, s: f64) {
    self.scale(s, s)
  }

  fn set_source_rgb_u8(&self, r: u8, g: u8, b: u8) {
    self.set_source_rgb(float(r), float(g), float(b))
  }

  fn set_source_rgba_u8(&self, r: u8, g: u8, b: u8, a: u8) {
    self.set_source_rgba(float(r), float(g), float(b), float(a))
  }

  fn set_source_rgb_u8_and_alpha(&self, r: u8, g: u8, b: u8, a: f64) {
    self.set_source_rgba(float(r), float(g), float(b), a)
  }

  fn set_source_rgb_u32(&self, rgb: u32) {
    let [b, g, r, _] = rgb.to_le_bytes();
    self.set_source_rgb_u8(r, g, b)
  }

  fn set_source_rgb_u32_and_alpha(&self, rgb: u32, a: f64) {
    let [b, g, r, _] = rgb.to_le_bytes();
    self.set_source_rgb_u8_and_alpha(r, g, b, a)
  }

  fn set_source_rgba_u32(&self, argb: u32) {
    let [b, g, r, a] = argb.to_le_bytes();
    self.set_source_rgba_u8(r, g, b, a)
  }

  fn circle(&self, x: f64, y: f64, r: f64) {
    self.new_sub_path();
    self.arc(x, y, r, 0.0, τ);
    self.close_path();
  }

  fn rounded_rect(&self, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let (x0, y0, x1, y1) = (x + r, y + r, x - r + w, y - r + h);
    self.new_sub_path();
    self.arc(x1, y1, r, 0.0, 0.5 * π); // bottom right
    self.arc(x0, y1, r, 0.5 * π, π); // bottom left
    self.arc(x0, y0, r, π, 1.5 * π); // top left
    self.arc(x1, y0, r, 1.5 * π, 0.0); // top right
    self.close_path();
  }

  // https://nikolskayaolia.medium.com/an-easy-way-to-implement-smooth-shapes-a5ba4e1139ed
  // https://pavellaptev.medium.com/squircles-on-the-web-houdini-to-the-rescue-5ef11f646b72
  // https://bootcamp.uxdesign.cc/smooth-corner-rounding-in-adobe-illustrator-94003145a7bf
  fn rounded_rect_smooth(&self, x: f64, y: f64, w: f64, h: f64, r: f64, t: f64) {
    let (xʹ, yʹ, rʹ, rʺ) = (x + w, y + h, r * (1.0 + t), 0.5 * r * (1.0 - t));
    self.new_sub_path();
    self.line_to(x, y + rʹ); // top left
    self.curve_to(x, y + rʺ, x + rʺ, y, x + rʹ, y);
    self.line_to(xʹ - rʹ, y); // top right
    self.curve_to(xʹ - rʺ, y, xʹ, y + rʺ, xʹ, y + rʹ);
    self.line_to(xʹ, yʹ - rʹ); // bottom right
    self.curve_to(xʹ, yʹ - rʺ, xʹ - rʺ, yʹ, xʹ - rʹ, yʹ);
    self.line_to(x + rʹ, yʹ); // bottom left
    self.curve_to(x + rʺ, yʹ, x, yʹ - rʺ, x, yʹ - rʹ);
    self.close_path();
  }

  // https://figma.com/blog/desperately-seeking-squircles/
  // https://martinrgb.github.io/blog/index.html#/Figma_Round_Corner
  // https://github.com/MartinRGB/Figma_Squircles_Approximation/blob/master/js/rounded-corners.js
  // https://github.com/phamfoo/figma-squircle/blob/main/packages/figma-squircle/src/draw.ts
  fn rounded_rect_figma(&self, x: f64, y: f64, w: f64, h: f64, r: f64, t: f64) {
    let (xʹ, yʹ) = (x + w, y + h);
    let θ = 0.25 * π * t;
    let y4 = (1.0 - θ.cos()) * r;
    let x4 = (1.0 - θ.sin()) * r;
    let x3 = (1.0 - (0.5 * θ).tan()) * r;
    let x1 = (1.0 + t) * r;
    let x2 = (x1 + x3 * 2.0) / 3.0;
    self.new_sub_path();
    self.line_to(x, y + x1); // top left
    self.curve_to(x, y + x2, x, y + x3, x + y4, y + x4);
    self.arc(x + r, y + r, r, π + θ, 1.5 * π - θ);
    self.curve_to(x + x3, y, x + x2, y, x + x1, y);
    self.line_to(xʹ - x1, y); // top right
    self.curve_to(xʹ - x2, y, xʹ - x3, y, xʹ - x4, y + y4);
    self.arc(xʹ - r, y + r, r, 1.5 * π + θ, -θ);
    self.curve_to(xʹ, y + x3, xʹ, y + x2, xʹ, y + x1);
    self.line_to(xʹ, yʹ - x1); // bottom right
    self.curve_to(xʹ, yʹ - x2, xʹ, yʹ - x3, xʹ - y4, yʹ - x4);
    self.arc(xʹ - r, yʹ - r, r, θ, 0.5 * π - θ);
    self.curve_to(xʹ - x3, yʹ, xʹ - x2, yʹ, xʹ - x1, yʹ);
    self.line_to(x + x1, yʹ); // bottom left
    self.curve_to(x + x2, yʹ, x + x3, yʹ, x + x4, yʹ - y4);
    self.arc(x + r, yʹ - r, r, 0.5 * π + θ, π - θ);
    self.curve_to(x, yʹ - x3, x, yʹ - x2, x, yʹ - x1);
    self.close_path();
  }

  // https://paintcodeapp.com/blogpost/code-for-ios-7-rounded-rectangles
  // https://blog.mikeswanson.com/iosroundedrect
  // https://mani.de/backstage/?p=483
  // https://mani.de/download/iOS%207%20App%20Icon%20Radius27@120.svg
  fn rounded_rect_ios7(&self, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let (k0, k1, k2) = (r * 1.52866483, r * 1.08849299, r * 0.86840701);
    let (k3, k4, k5) = (r * 0.07491100, r * 0.63149399, r * 0.16906001);
    let (k6, xʹ, yʹ) = (r * 0.37282401, x + w, y + h);
    self.new_sub_path();
    self.line_to(x, y + k0); // top left
    self.curve_to(x, y + k1, x, y + k2, x + k3, y + k4);
    self.curve_to(x + k5, y + k6, x + k6, y + k5, x + k4, y + k3);
    self.curve_to(x + k2, y, x + k1, y, x + k0, y);
    self.line_to(xʹ - k0, y); // top right
    self.curve_to(xʹ - k1, y, xʹ - k2, y, xʹ - k4, y + k3);
    self.curve_to(xʹ - k6, y + k5, xʹ - k5, y + k6, xʹ - k3, y + k4);
    self.curve_to(xʹ, y + k2, xʹ, y + k1, xʹ, y + k0);
    self.line_to(xʹ, yʹ - k0); // bottom right
    self.curve_to(xʹ, yʹ - k1, xʹ, yʹ - k2, xʹ - k3, yʹ - k4);
    self.curve_to(xʹ - k5, yʹ - k6, xʹ - k6, yʹ - k5, xʹ - k4, yʹ - k3);
    self.curve_to(xʹ - k2, yʹ, xʹ - k1, yʹ, xʹ - k0, yʹ);
    self.line_to(x + k0, yʹ); // bottom left
    self.curve_to(x + k1, yʹ, x + k2, yʹ, x + k4, yʹ - k3);
    self.curve_to(x + k6, yʹ - k5, x + k5, yʹ - k6, x + k3, yʹ - k4);
    self.curve_to(x, yʹ - k2, x, yʹ - k1, x, yʹ - k0);
    self.close_path();
  }

  fn text_width(&self, parts: &[&str]) -> Result<f64> {
    parts.iter().try_fold(0.0, |acc, part| {
      let ext = self.text_extents(part)?;
      Ok(acc + ext.x_advance())
    })
  }

  fn map_path<F>(&self, map: F) -> Result<()>
  where
    F: Fn(f64, f64) -> (f64, f64),
  {
    let path = self.copy_path()?;
    self.new_path();

    for item in path.iter() {
      match item {
        PathSegment::MoveTo((x, y)) => {
          let (x, y) = map(x, y);
          self.move_to(x, y);
        }
        PathSegment::LineTo((x, y)) => {
          let (x, y) = map(x, y);
          self.line_to(x, y);
        }
        PathSegment::CurveTo((x1, y1), (x2, y2), (x3, y3)) => {
          let (x1, y1) = map(x1, y1);
          let (x2, y2) = map(x2, y2);
          let (x3, y3) = map(x3, y3);
          self.curve_to(x1, y1, x2, y2, x3, y3);
        }
        PathSegment::ClosePath => {
          self.close_path();
        }
      }
    }

    Ok(())
  }
}

fn float(byte: u8) -> f64 {
  byte as f64 / u8::MAX as f64
}
