use cairo::*;
use lib::cairo::ext::ContextExt;
use lib::fmt::num::Format;

use super::api;
use super::fmt::Num;
use super::Result;
use super::{icons, util};

pub fn current(ctx: &Context, weather: &api::Root, city: &str) -> Result<()> {
  ctx.save()?;
  ctx.translate(12.0, 8.0);

  let dt = util::datetime(weather.timezone_offset, weather.current.dt);
  let font_size = 10.0;
  util::cairo::set_font_variations(ctx, "opsz=10")?;
  ctx.set_font_size(font_size);
  ctx.set_source_rgb_u32(0xffffff);
  ctx.move_to(0.0, font_size);
  ctx.show_text(&dt.format("%A, %B %-e, %H:%M ").to_string())?;
  ctx.set_source_rgb_u32(0x80848e);
  ctx.show_text(&dt.format("%:z").to_string().replace('-', "\u{2212}"))?;
  ctx.translate(0.0, font_size);

  let font_size = 18.0;
  util::cairo::set_font_variations(ctx, "opsz=18")?;
  ctx.set_font_size(font_size);
  ctx.set_source_rgb_u32(0xffffff);
  ctx.translate(0.0, font_size + 8.0);
  ctx.move_to(0.0, 0.0);
  ctx.show_text(city)?;

  let font_size = 10.0;
  util::cairo::set_font_variations(ctx, "opsz=10")?;
  ctx.set_font_size(font_size);
  ctx.translate(0.0, font_size + 4.0);
  ctx.move_to(0.0, 0.0);
  ctx.show_text(&{
    let mut acc = String::new();
    for (i, w) in weather.current.weather.iter().enumerate() {
      match i {
        0 => acc.push_str(&util::capitalize(&w.description)),
        _ => acc.push_str(&w.description),
      }
      acc.push_str(", ");
    }
    acc.push_str(util::beaufort_scale(weather.current.wind_speed));
    acc
  })?;

  ctx.translate(0.0, 8.0);
  ctx.save()?;
  icons::openweather(&weather.current.weather[0].icon, &|svg, size| -> Result<_> {
    ctx.scale(64.0 / size.width, 64.0 / size.height);
    svg.render_cairo(ctx)?;
    Ok(())
  })?;
  ctx.restore()?;
  ctx.translate(64.0 + 8.0, 0.0);

  let font_size = 18.0;
  util::cairo::set_font_variations(ctx, "opsz=18")?;
  ctx.set_font_size(font_size);
  ctx.set_source_rgb_u32(0xffffff);
  ctx.translate(0.0, font_size);
  ctx.move_to(0.0, 0.0);
  ctx.show_text(&format!("{:.0}°", Num(weather.current.temp)))?;

  let font_size = 10.0;
  util::cairo::set_font_variations(ctx, "opsz=10")?;
  ctx.set_font_size(font_size);
  ctx.show_text(&format!(" feels like {:.0}°C", Num(weather.current.feels_like)))?;

  let show_icon = |name| {
    ctx.save()?;
    ctx.set_font_size(15.0);
    ctx.select_font_face("Material Symbols Outlined", FontSlant::Normal, FontWeight::Normal);
    util::cairo::set_font_variations(ctx, "FILL=1,wght=300")?;
    ctx.rel_move_to(0.0, 4.0);
    ctx.show_text(name)?;
    ctx.rel_move_to(0.0, -4.0);
    ctx.restore()
  };

  ctx.translate(0.0, font_size + 6.0);
  ctx.move_to(0.0, 0.0);

  show_icon("\u{f879}")?;
  ctx.show_text(&format!(" {:.0}° ", Num(weather.current.dew_point)))?;

  show_icon("\u{f87e}")?;
  ctx.show_text(&format!(" {}% ", weather.current.humidity))?;

  show_icon("\u{efd8}")?;
  ctx.show_text(&{
    let wind = weather.current.wind_speed;
    match weather.current.wind_gust {
      Some(gust) if gust != 0.0 => format!(" {:#.2}\u{2013}{:#.2}m/s ", Num(wind), Num(gust)),
      _ => format!(" {:#.2}m/s ", Num(wind)),
    }
  })?;

  ctx.translate(0.0, font_size + 6.0);
  ctx.move_to(0.0, 0.0);

  show_icon("\u{e9e4}")?;
  ctx.show_text(&format!(" {}hPa ", weather.current.pressure))?;

  show_icon("\u{e8f4}")?;
  ctx.show_text(&format!(" {}m ", weather.current.visibility.si().precision(2)))?;

  show_icon("\u{e81a}")?;
  ctx.show_text(&format!(" {:#.2} ", Num(weather.current.uvi)))?;

  ctx.restore()?;

  Ok(())
}
