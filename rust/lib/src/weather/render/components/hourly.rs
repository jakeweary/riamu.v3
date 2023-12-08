use std::f64::consts::TAU as τ;
use std::fmt::Arguments;

use cairo::*;
use chrono::prelude::*;

use crate::discord;
use crate::srgb::Srgb;
use crate::weather::render::draw;

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
  ctx.translate(12.5, IMAGE_H as f64 - 13.0);

  let font_size = 10.0;
  util::cairo::set_font_variations(ctx, "opsz=10,wdth=50")?;
  ctx.set_font_size(font_size);

  let lines = || -> cairo::Result<_> {
    ctx.save()?;
    ctx.set_line_cap(LineCap::Round);

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

  // time & uvi
  {
    let text0 = color_sub(0x949ba4, 0x2b2d31);
    let text1 = 0xffffff;

    let uvi_range = Range::of(&weather.hourly, |h| h.uvi);
    let uvi_grad = LinearGradient::new(0.0, 0.0, width, 0.0);
    for (i, hour) in weather.hourly.iter().enumerate() {
      let t = (0.5 + i as f64) / weather.hourly.len() as f64;
      let a = uvi_range.unlerp(hour.uvi);
      let [r, g, b] = Srgb::from(Srgb::<_, 3>::from(0xfbbf24)).into();
      uvi_grad.add_color_stop_rgba(t, r, g, b, a);
    }

    let r = 6.5;
    draw::rounded_rect(ctx, w / 2.0, -3.5, width - w, 0.0, r);

    ctx.save()?;
    ctx.set_line_width(2.0);
    ctx.set_source(&uvi_grad)?;
    ctx.stroke_preserve()?;
    ctx.clip();
    ctx.set_source_rgb_u32(0x2b2d31);
    ctx.paint()?;
    ctx.set_source(&uvi_grad)?;
    ctx.paint_with_alpha(0.2)?;
    ctx.restore()?;

    ctx.push_group();
    ctx.translate(0.5 * w, 0.0);
    let h0 = weather.hourly[0].dt;
    for i in -1..=weather.hourly.len() as i64 {
      let dt = util::datetime(weather.timezone_offset, h0 + i * 3600);
      if dt.hour() % 3 == 0 {
        let text = dt.format("%H:%M").to_string();
        let ext = ctx.text_extents(&text)?;
        ctx.set_source_rgb_u32(if dt.hour() == 0 { text1 } else { text0 });
        ctx.move_to(w * i as f64 - ext.x_advance() / 2.0, 0.0);
        ctx.show_text(&text)?;
      }
    }
    ctx.pop_group_to_source()?;

    let mask = {
      let rg = RadialGradient::new(0.0, 0.0, 0.0, 0.0, 0.0, r);
      rg.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 1.0);
      rg.add_color_stop_rgba(4.0 / r, 0.0, 0.0, 0.0, 1.0);
      rg.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 0.0);

      ctx.push_group_with_content(Content::Alpha);
      ctx.translate(w / 2.0, -3.5);
      ctx.set_source_rgba(0.0, 0.0, 0.0, 1.0);
      ctx.rectangle(0.0, -r, width - w, 2.0 * r);
      ctx.fill()?;
      ctx.set_source(&rg)?;
      ctx.paint()?;
      ctx.translate(width - w, 0.0);
      ctx.set_source(&rg)?;
      ctx.paint()?;
      ctx.pop_group()?
    };

    ctx.save()?;
    ctx.set_operator(Operator::Add);
    ctx.mask(mask)?;
    ctx.restore()?;

    ctx.translate(0.0, -10.5 - font_size);
  }

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

    draw::spline(ctx, w, &weather.hourly, |h| map(h.dew_point));
    ctx.set_dash(&[1.0, 3.0], 0.0);
    ctx.set_source_rgb_u32(discord::colors::TABLE[2].light);
    ctx.stroke()?;

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

    for (i, hour) in weather.hourly.iter().enumerate() {
      draw::circle(ctx, w * i as f64, map(hour.wind_gust), 1.5);
    }
    ctx.set_source_rgb_u32(discord::colors::TABLE[8].light);
    ctx.fill()?;

    for (i, hour) in weather.hourly.iter().enumerate() {
      let (x, y) = (w * i as f64, map(hour.wind_speed));
      let angle = (hour.wind_deg as f64).to_radians();
      let scale = 5.0;
      draw::arrow(ctx, x, y, scale, angle);
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

    for (i, h) in weather.hourly.iter().enumerate() {
      let clouds = h.clouds as f64 / 100.0;
      ctx.rectangle(0.5 + w * i as f64, -0.5, w - 1.0, -(height - 1.0) * clouds);
    }
    ctx.set_source_rgb_u32(0x3f4248);
    ctx.fill()?;

    for (i, h) in weather.hourly.iter().enumerate() {
      let rain = h.rain.as_ref().map_or(0.0, |r| r.one_hour);
      let rain = range.unlerp(rain);
      ctx.rectangle(0.5 + w * i as f64, -0.5, w - 1.0, -(height - 1.0) * rain);
      ctx.set_source_rgb_u32_and_alpha(discord::colors::TABLE[2].dark, h.pop);
      ctx.fill()?;

      let snow = h.snow.as_ref().map_or(0.0, |r| r.one_hour);
      let snow = range.unlerp(snow);
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

fn color_sub(a: u32, b: u32) -> u32 {
  let [b0, g0, r0, a0] = a.to_le_bytes();
  let [b1, g1, r1, a1] = b.to_le_bytes();
  u32::from_le_bytes([b0 - b1, g0 - g1, r0 - r1, a0 - a1])
}

fn legend(ctx: &Context, color: u32, pairs: &[(u32, Arguments<'_>)]) -> cairo::Result<()> {
  let (x, y) = ctx.current_point()?;

  ctx.set_source_rgb_u32(color);
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
