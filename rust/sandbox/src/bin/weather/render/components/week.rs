use cairo::*;
use lib::cairo::ext::ContextExt;

use super::fmt::Num;
use super::Result;
use super::*;

pub fn week(ctx: &Context, weather: &api::Root) -> Result<()> {
  let width = 8.0 * 28.0;
  let day_w = width / weather.daily.len() as f64;

  ctx.save()?;
  ctx.translate(285.0, 8.0);

  for day in &weather.daily {
    ctx.save()?;

    ctx.set_source_rgb_u32(0xffffff);
    let font_size = 10.0;
    util::cairo::set_font_variations(ctx, "opsz=10")?;
    ctx.set_font_size(font_size);
    let dt = util::datetime(weather.timezone_offset, day.dt);
    let text = dt.format("%a").to_string();
    let ext = ctx.text_extents(&text)?;
    ctx.move_to(-ext.x_advance() / 2.0, font_size);
    ctx.show_text(&text)?;
    ctx.translate(0.0, font_size + 4.0);

    ctx.save()?;
    icons::openweather(&day.weather[0].icon, &|svg, size| -> Result<_> {
      ctx.scale(28.0 / size.height, 28.0 / size.height);
      ctx.translate(-size.width / 2.0, 0.0);
      svg.render_cairo(ctx)?;
      Ok(())
    })?;
    ctx.restore()?;
    ctx.translate(0.0, 28.0 + 1.0);

    let font_size = 10.0;
    util::cairo::set_font_variations(ctx, "wdth=50,opsz=10")?;
    ctx.set_font_size(font_size);

    ctx.set_source_rgb_u32(0xffffff);
    let text = format!("{:.0}°", Num(day.temp.max));
    ctx.move_to(0.0, font_size);
    util::cairo::center_text_by_template(ctx, "°00°", &text)?; // two ° is a hack for proper alignment
    ctx.show_text(&text)?;
    ctx.translate(0.0, font_size);

    ctx.set_source_rgb_u32(0x80848e);
    let text = format!("{:.0}°", Num(day.temp.min));
    ctx.move_to(0.0, font_size);
    util::cairo::center_text_by_template(ctx, "°00°", &text)?; // two ° is a hack for proper alignment
    ctx.show_text(&text)?;
    ctx.translate(0.0, font_size);

    ctx.translate(0.0, 13.5);
    ctx.save()?;
    ctx.scale(6.0, 6.0);
    ctx.rotate((day.wind_deg as f64).to_radians());
    draw::arrow(ctx);
    ctx.set_source_rgb_u32(0xffffff);
    ctx.fill()?;
    ctx.restore()?;
    ctx.translate(0.0, 11.5);

    ctx.set_source_rgb_u32(0xffffff);
    let text = format!("{:.1}", Num(day.wind_speed));
    ctx.move_to(0.0, font_size);
    util::cairo::center_text_by_template(ctx, "0.0", &text)?;
    ctx.show_text(&text)?;
    ctx.translate(0.0, font_size);

    ctx.set_source_rgb_u32(0x80848e);
    let text = format!("{:.1}", Num(day.wind_gust));
    ctx.move_to(0.0, font_size);
    util::cairo::center_text_by_template(ctx, "0.0", &text)?;
    ctx.show_text(&text)?;
    ctx.translate(0.0, font_size);

    ctx.restore()?;
    ctx.translate(day_w, 0.0);
  }

  ctx.restore()?;

  Ok(())
}
