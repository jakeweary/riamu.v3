use std::f64::consts::TAU as τ;
use std::fmt::Arguments;

use cairo::*;
use chrono::prelude::*;

use crate::discord;

use super::fmt::Num;
use super::range::Range;
use super::Result;
use super::*;

pub fn hourly(ctx: &Context, weather: &api::Onecall) -> Result<()> {
  let height = 40.0;
  let width = 48.0 * 10.0;
  let w = width / weather.hourly.len() as f64;
  let gap = 22.0;

  ctx.save()?;
  ctx.translate(12.5, IMAGE_H as f64 - 10.0);

  let font_size = 10.0;
  util::cairo::set_font_variations(ctx, "opsz=10,wdth=50")?;
  ctx.set_font_size(font_size);

  let background = || -> cairo::Result<_> {
    ctx.save()?;

    ctx.new_path();
    for i in 0..weather.hourly.len() {
      let x = (0.5 + i as f64) * w;
      ctx.move_to(x, 0.0);
      ctx.line_to(x, -height);
    }
    ctx.set_source_rgb_u32(0x2b2d31);
    ctx.stroke()?;

    ctx.new_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(width, 0.0);
    ctx.move_to(0.0, -height);
    ctx.line_to(width, -height);
    ctx.set_line_cap(LineCap::Round);
    ctx.set_source_rgb_u32(0x232428);
    ctx.stroke()?;

    ctx.restore()?;
    Ok(())
  };

  // time
  {
    ctx.save()?;
    ctx.translate(0.5 * w, 0.0);

    for (i, hour) in weather.hourly.iter().enumerate() {
      let dt = util::datetime(weather.timezone_offset, hour.dt);
      if dt.hour() % 3 == 0 {
        let color = if dt.hour() == 0 { 0xffffff } else { 0x949ba4 };
        let text = dt.format("%H:%M").to_string();
        let ext = ctx.text_extents(&text)?;
        ctx.move_to(w * i as f64 - ext.x_advance() / 2.0, 0.0);
        ctx.set_source_rgb_u32(color);
        ctx.show_text(&text)?;
      }
    }

    ctx.restore()?;
    ctx.translate(0.0, -7.5 - font_size);
  }

  // temps
  {
    let temp = Range::of(&weather.hourly, |h| h.temp);
    let dew_point = Range::of(&weather.hourly, |h| h.dew_point);
    let range = temp & dew_point;
    let map = |value| -0.0 - (height - 0.0) * range.normalize(value);

    ctx.save()?;
    background()?;

    ctx.move_to(0.0, -height - 7.5);
    #[rustfmt::skip]
    legend(ctx, discord::colors::TABLE[8].light, &[
      (0x949ba4, format_args!(" Temperature ")),
      (0xffffff, format_args!("{:.0}", Num(temp.min))),
      (0x949ba4, format_args!(" to ")),
      (0xffffff, format_args!("{:.0}", Num(temp.max))),
      (0x949ba4, format_args!("°C")),
    ])?;
    #[rustfmt::skip]
    legend(ctx, discord::colors::TABLE[2].light, &[
      (0x949ba4, format_args!(" Dew point ")),
      (0xffffff, format_args!("{:.0}", Num(dew_point.min))),
      (0x949ba4, format_args!(" to ")),
      (0xffffff, format_args!("{:.0}", Num(dew_point.max))),
      (0x949ba4, format_args!("°C")),
    ])?;

    ctx.move_to(6.0 + width, 3.5 - height);
    ctx.set_source_rgb_u32(0xffffff);
    ctx.show_text(&format!("{:.0}", Num(range.max)))?;
    ctx.set_source_rgb_u32(0x949ba4);
    ctx.show_text("°C")?;
    ctx.move_to(6.0 + width, 3.5);
    ctx.set_source_rgb_u32(0xffffff);
    ctx.show_text(&format!("{:.0}", Num(range.min)))?;

    ctx.translate(0.5 * w, 0.0);
    ctx.set_line_cap(LineCap::Round);
    ctx.set_line_width(1.0);

    ctx.new_path();
    draw::spline(ctx, w, &weather.hourly, |h| map(h.dew_point));
    ctx.set_dash(&[1.0, 3.0], 0.0);
    ctx.set_source_rgb_u32(discord::colors::TABLE[2].light);
    ctx.stroke()?;

    ctx.new_path();
    draw::spline(ctx, w, &weather.hourly, |h| map(h.temp));
    ctx.set_dash(&[], 0.0);
    ctx.set_source_rgb_u32(discord::colors::TABLE[8].light);
    ctx.stroke()?;

    ctx.restore()?;
    ctx.translate(0.0, -height - gap);
  }

  // wind
  {
    let wind_speed = Range::of(&weather.hourly, |h| h.wind_speed);
    let wind_gust = Range::of(&weather.hourly, |h| h.wind_gust);
    let range = Range::new(0.0, 1.0) & wind_speed & wind_gust;
    let map = |value| -0.0 - (height - 0.0) * range.normalize(value);

    ctx.save()?;
    background()?;

    ctx.move_to(0.0, -height - 7.5);
    #[rustfmt::skip]
    legend(ctx, 0xffffff, &[
      (0x949ba4, format_args!(" Wind up to ")),
      (0xffffff, format_args!("{:#.2}", Num(wind_speed.max))),
      (0x949ba4, format_args!("m/s")),
    ])?;
    #[rustfmt::skip]
    legend(ctx, discord::colors::TABLE[8].light, &[
      (0x949ba4, format_args!(" Wind gusts up to ")),
      (0xffffff, format_args!("{:#.2}", Num(wind_gust.max))),
      (0x949ba4, format_args!("m/s")),
    ])?;

    ctx.move_to(6.0 + width, 3.5 - height);
    ctx.set_source_rgb_u32(0xffffff);
    ctx.show_text(&format!("{:#.2}", Num(range.max)))?;
    ctx.set_source_rgb_u32(0x949ba4);
    ctx.show_text("m/s")?;
    ctx.move_to(6.0 + width, 3.5);
    ctx.set_source_rgb_u32(0xffffff);
    ctx.show_text("0")?;

    ctx.translate(0.5 * w, 0.0);

    ctx.new_path();
    for (i, hour) in weather.hourly.iter().enumerate() {
      draw::circle(ctx, w * i as f64, map(hour.wind_gust), 1.5);
    }
    ctx.set_source_rgb_u32(discord::colors::TABLE[8].light);
    ctx.fill()?;

    ctx.new_path();
    for (i, hour) in weather.hourly.iter().enumerate() {
      ctx.save()?;
      ctx.translate(w * i as f64, map(hour.wind_speed));
      ctx.rotate((hour.wind_deg as f64).to_radians());
      ctx.scale1(5.0);
      draw::arrow(ctx);
      ctx.restore()?;
    }
    ctx.set_source_rgb_u32(0xffffff);
    ctx.fill()?;

    ctx.restore()?;
    ctx.translate(0.0, -height - gap);
  }

  // clouds and rain
  {
    let clouds = Range::of(&weather.hourly, |h| h.clouds as f64);
    let rain = Range::of(&weather.hourly, |h| h.rain.as_ref().map_or(0.0, |r| r.one_hour));
    let snow = Range::of(&weather.hourly, |h| h.snow.as_ref().map_or(0.0, |s| s.one_hour));
    let range = Range::new(0.0, 1.0) & rain & snow;

    ctx.save()?;
    background()?;

    ctx.move_to(0.0, -height - 7.5);
    if rain.max > 0.0 {
      #[rustfmt::skip]
      legend(ctx, discord::colors::TABLE[2].dark, &[
        (0x949ba4, format_args!(" Rain up to ")),
        (0xffffff, format_args!("{:#.2}", Num(rain.max))),
        (0x949ba4, format_args!("mm/h")),
      ])?;
    }
    if snow.max > 0.0 {
      #[rustfmt::skip]
      legend(ctx, discord::colors::TABLE[8].dark, &[
        (0x949ba4, format_args!(" Snow up to ")),
        (0xffffff, format_args!("{:#.2}", Num(snow.max))),
        (0x949ba4, format_args!("mm/h")),
      ])?;
    }
    #[rustfmt::skip]
    legend(ctx, 0x3f4248, &[
      (0x949ba4, format_args!(" Clouds up to ")),
      (0xffffff, format_args!("{}", clouds.max)),
      (0x949ba4, format_args!("%")),
    ])?;

    if rain.max > 0.0 || snow.max > 0.0 {
      ctx.move_to(6.0 + width, 3.5 - height);
      ctx.set_source_rgb_u32(0xffffff);
      ctx.show_text(&format!("{:#.2}", Num(range.max)))?;
      ctx.set_source_rgb_u32(0x949ba4);
      ctx.show_text("mm/h")?;
      ctx.move_to(6.0 + width, 3.5);
      ctx.set_source_rgb_u32(0xffffff);
      ctx.show_text("0")?;
    }

    ctx.new_path();
    for (i, h) in weather.hourly.iter().enumerate() {
      let clouds = h.clouds as f64 / 100.0;
      ctx.rectangle(0.5 + w * i as f64, -0.5, w - 1.0, -(height - 1.0) * clouds);
    }
    ctx.set_source_rgb_u32(0x3f4248);
    ctx.fill()?;

    for (i, h) in weather.hourly.iter().enumerate() {
      let rain = h.rain.as_ref().map_or(0.0, |r| r.one_hour);
      let rain = range.normalize(rain);
      ctx.new_path();
      ctx.rectangle(0.5 + w * i as f64, -0.5, w - 1.0, -(height - 1.0) * rain);
      ctx.set_source_rgb_u32_and_alpha(discord::colors::TABLE[2].dark, h.pop);
      ctx.fill()?;

      let snow = h.snow.as_ref().map_or(0.0, |r| r.one_hour);
      let snow = range.normalize(snow);
      ctx.new_path();
      ctx.rectangle(0.5 + w * i as f64, -0.5, w - 1.0, -(height - 1.0) * snow);
      ctx.set_source_rgb_u32_and_alpha(discord::colors::TABLE[8].dark, h.pop);
      ctx.fill()?;
    }

    ctx.restore()?;
    ctx.translate(0.0, -height - gap);
  }

  ctx.restore()?;

  Ok(())
}

fn legend(ctx: &Context, color: u32, pairs: &[(u32, Arguments<'_>)]) -> cairo::Result<()> {
  let (x, y) = ctx.current_point()?;

  ctx.set_source_rgb_u32(color);
  ctx.new_path();
  ctx.arc(x + 3.5, y - 3.5, 3.0, 0.0, τ);
  ctx.close_path();
  ctx.fill()?;

  ctx.move_to(x + 7.0, y);
  for &(color, arg) in pairs {
    ctx.set_source_rgb_u32(color);
    match arg.as_str() {
      Some(arg) => ctx.show_text(arg)?,
      None => ctx.show_text(&arg.to_string())?,
    }
  }
  ctx.rel_move_to(10.0, 0.0);

  Ok(())
}
