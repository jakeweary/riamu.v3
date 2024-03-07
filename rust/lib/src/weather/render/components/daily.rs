use cairo::*;

use crate::color::srgb::sRGB;

use super::fmt::Num;
use super::range::Range;
use super::Result;
use super::*;

pub fn daily(ctx: &Context, weather: &api::Onecall) -> Result<()> {
  let day_width = 28.0;

  let temp_min = Range::of(&weather.daily, |d| d.temp.min);
  let temp_max = Range::of(&weather.daily, |d| d.temp.max);
  let temp = temp_min & temp_max;

  let wind_speed = Range::of(&weather.daily, |d| d.wind_speed);
  let wind_gust = Range::of(&weather.daily, |d| d.wind_gust);
  let wind = wind_speed & wind_gust & 0.0;

  ctx.save()?;
  ctx.translate(285.0, 8.0);

  let color0 = 0xffffff;
  let color1 = 0x949ba4;

  let font_size = 10.0;
  util::cairo::set_font_variations(ctx, "opsz=10,wdth=50")?;
  ctx.set_font_size(font_size);

  let indicator_h = 3.0 - 2.0 * font_size;
  let indicator_g = {
    let lg = LinearGradient::new(0.0, 0.0, 0.0, indicator_h);
    let [b, g, r] = sRGB::from(sRGB::<_, 3>::from(color1)).into();
    lg.add_color_stop_rgb(0.0, r, g, b);
    let [b, g, r] = sRGB::from(sRGB::<_, 3>::from(color0)).into();
    lg.add_color_stop_rgb(1.0, r, g, b);
    lg
  };

  let show_indicator = |r: Range, min, max| {
    ctx.save()?;
    ctx.translate(8.0, 0.0);
    ctx.set_line_cap(LineCap::Round);

    ctx.move_to(0.0, 0.0);
    ctx.line_to(0.0, indicator_h);
    ctx.set_source_rgb_u32(0x2b2d31);
    ctx.set_line_width(4.0);
    ctx.stroke()?;

    ctx.move_to(0.0, indicator_h * r.unlerp(min));
    ctx.line_to(0.0, indicator_h * r.unlerp(max));
    ctx.set_source(&indicator_g)?;
    ctx.set_line_width(2.0);
    ctx.stroke()?;

    ctx.restore()
  };

  let show_number = |color, n: String| {
    ctx.translate(0.0, font_size);
    ctx.move_to(0.0, 0.0);
    util::cairo::center_text_by_template(ctx, "0", &n)?;
    ctx.set_source_rgb_u32(color);
    ctx.show_text(&n)
  };

  for day in &weather.daily {
    ctx.save()?;

    let dt = util::datetime(weather.timezone_offset, day.dt);
    let text = dt.format("%a").to_string();
    let ext = ctx.text_extents(&text)?;
    ctx.move_to(-ext.x_advance() / 2.0, font_size);
    ctx.set_source_rgb_u32(0xffffff);
    ctx.show_text(&text)?;
    ctx.translate(0.0, font_size + 4.0);

    let (svg, size) = icons::openweather(&day.weather[0].icon)?;
    ctx.save()?;
    ctx.scale1(day_width / size.width);
    ctx.translate(-size.width / 2.0, 0.0);
    svg.render_cairo(ctx)?;
    ctx.restore()?;
    ctx.translate(0.0, day_width + 1.0);

    show_number(color0, format!("{:.0}", Num(day.temp.max)))?;
    show_number(color1, format!("{:.0}", Num(day.temp.min)))?;
    show_indicator(temp, day.temp.min, day.temp.max)?;

    draw::arrow(ctx, 0.0, 13.5, 6.0, (day.wind_deg as f64).to_radians());
    ctx.set_source_rgb_u32(0xffffff);
    ctx.fill()?;
    ctx.translate(0.0, 25.0);

    show_number(color0, format!("{:.0}", Num(day.wind_gust)))?;
    show_number(color1, format!("{:.0}", Num(day.wind_speed)))?;
    show_indicator(wind, day.wind_speed, day.wind_gust)?;

    ctx.restore()?;
    ctx.translate(day_width, 0.0);
  }

  ctx.set_source_rgb_u32(color1);
  ctx.translate(-10.5, 5.0 + 2.5 * font_size + day_width);
  ctx.move_to(0.0, 0.0);
  ctx.show_text("°C")?;
  ctx.translate(0.0, 24.0 + 2.0 * font_size);
  ctx.move_to(0.0, 0.0);
  ctx.show_text("m/s")?;

  ctx.restore()?;

  Ok(())
}
