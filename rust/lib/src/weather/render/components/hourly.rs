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

  let lines = || -> cairo::Result<_> {
    ctx.save()?;
    ctx.set_line_cap(LineCap::Round);

    ctx.new_path();
    for i in 0..weather.hourly.len() {
      let x = (0.5 + i as f64) * w;
      ctx.move_to(x, 0.0);
      ctx.line_to(x, -height);
    }
    for i in 0..5 {
      let t = i as f64 / 4.0;
      ctx.move_to(0.0, -t * height);
      ctx.line_to(width, -t * height);
    }
    ctx.set_source_rgb_u32(0x2b2d31);
    ctx.stroke()?;

    ctx.new_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(width, 0.0);
    ctx.move_to(0.0, -height);
    ctx.line_to(width, -height);
    ctx.set_source_rgb_u32(0x232428);
    ctx.stroke()?;

    ctx.restore()
  };

  let numbers = |range: Range, precision, units| -> cairo::Result<_> {
    ctx.save()?;
    ctx.translate(6.0 + width, 3.5);
    ctx.set_source_rgb_u32(0xffffff);
    for i in 0..5 {
      let t = i as f64 / 4.0;
      ctx.move_to(0.0, -t * height);
      ctx.show_text(&format!("{:.*}", precision, range.lerp(t)))?;
    }
    ctx.set_source_rgb_u32(0x949ba4);
    ctx.show_text(units)?;
    ctx.restore()
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

  // TODO: not finished
  //
  // // sun
  // {
  //   let px = w / 3600.0; // seconds to pixels
  //   let zero = weather.hourly[0].dt;
  //
  //   ctx.save()?;
  //   ctx.translate(0.5 * w, 0.0);
  //   ctx.new_path();
  //
  //   for day in &weather.daily {
  //     ctx.move_to(px * (day.sunrise - zero) as f64, 0.0);
  //     ctx.line_to(px * (day.sunset - zero) as f64, 0.0);
  //   }
  //
  //   ctx.set_line_width(1.0);
  //   ctx.set_line_cap(LineCap::Round);
  //   ctx.set_source_rgb_u32(discord::colors::TABLE[5].light);
  //   ctx.stroke()?;
  //   ctx.restore()?;
  //
  //   ctx.translate(0.0, -4.0);
  // }

  // temps
  {
    let temp = Range::of(&weather.hourly, |h| h.temp);
    let dew_point = Range::of(&weather.hourly, |h| h.dew_point);
    let range = (temp & dew_point).round_n_rel(4.0).round();
    let map = |value| -0.0 - (height - 0.0) * range.unlerp(value);

    ctx.save()?;
    lines()?;
    numbers(range, 0, "°C")?;

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
    let range = (wind_speed & wind_gust & 0.0).round_n_abs(4.0);
    let map = |value| -0.0 - (height - 0.0) * range.unlerp(value);

    ctx.save()?;
    lines()?;
    numbers(range, 0, "m/s")?;

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
    let range = (rain & snow & 0.0).round_n_abs(0.4);

    ctx.save()?;
    lines()?;

    if range.max > 0.0 {
      numbers(range, 1, "mm/h")?;
    }

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

    ctx.new_path();
    for (i, h) in weather.hourly.iter().enumerate() {
      let clouds = h.clouds as f64 / 100.0;
      ctx.rectangle(0.5 + w * i as f64, -0.5, w - 1.0, -(height - 1.0) * clouds);
    }
    ctx.set_source_rgb_u32(0x3f4248);
    ctx.fill()?;

    for (i, h) in weather.hourly.iter().enumerate() {
      let rain = h.rain.as_ref().map_or(0.0, |r| r.one_hour);
      let rain = range.unlerp(rain);
      ctx.new_path();
      ctx.rectangle(0.5 + w * i as f64, -0.5, w - 1.0, -(height - 1.0) * rain);
      ctx.set_source_rgb_u32_and_alpha(discord::colors::TABLE[2].dark, h.pop);
      ctx.fill()?;

      let snow = h.snow.as_ref().map_or(0.0, |r| r.one_hour);
      let snow = range.unlerp(snow);
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
