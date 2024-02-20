use cairo::*;
use chrono::Offset;

use crate::cairo::ext::ContextExt;
use crate::fmt::num::Format;
use crate::weather::render::iso3166::TABLE as ISO3166;

use super::fmt::Num;
use super::Result;
use super::{api, icons, util};

pub fn current(ctx: &Context, weather: &api::Onecall, loc: &api::geo::Location) -> Result<()> {
  ctx.save()?;
  ctx.translate(12.0, 8.0);

  let city = loc.local_names.en.as_ref().unwrap_or(&loc.name);
  let country = ISO3166.iter().find(|(code, _)| &loc.country == code);

  let dt = util::datetime(weather.timezone_offset, weather.current.dt);
  let tz_offset_min = dt.offset().fix().local_minus_utc() / 60;
  let tz_fmt = if tz_offset_min % 60 == 0 { "UTC%:::z" } else { "UTC%:z" };

  let font_size = 10.0;
  util::cairo::set_font_variations(ctx, "opsz=10,wdth=50")?;
  ctx.set_font_size(font_size);
  ctx.set_source_rgb_u32(0xffffff);
  ctx.move_to(0.0, font_size);
  ctx.show_text(&dt.format("%A, %B %-e, %H:%M ").to_string())?;
  ctx.set_source_rgb_u32(0x949ba4);
  ctx.show_text(&dt.format(tz_fmt).to_string().replace('-', "\u{2212}"))?;
  ctx.translate(0.0, font_size);

  let font_size = 18.0;
  util::cairo::set_font_variations(ctx, "opsz=18,wght=600")?;
  ctx.set_font_size(font_size);
  ctx.set_source_rgb_u32(0xffffff);
  ctx.translate(0.0, font_size + 8.0);
  ctx.move_to(0.0, 0.0);
  ctx.show_text(city)?;
  ctx.show_text(" ")?;
  util::cairo::set_font_variations(ctx, "opsz=18,wght=200")?;
  ctx.show_text(match country {
    Some(&(_, country)) if ctx.text_width(&[city, " ", country])? < 200.0 => country,
    _ => &loc.country,
  })?;

  let font_size = 10.0;
  util::cairo::set_font_variations(ctx, "opsz=10,wdth=50")?;
  ctx.set_font_size(font_size);
  ctx.set_source_rgb_u32(0xffffff);
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

  let (svg, size) = icons::openweather(&weather.current.weather[0].icon)?;
  ctx.translate(0.0, 8.0);
  ctx.save()?;
  ctx.scale(64.0 / size.width, 64.0 / size.height);
  svg.render_cairo(ctx)?;
  ctx.restore()?;
  ctx.translate(64.0 + 8.0, 0.0);

  let font_size = 18.0;
  util::cairo::set_font_variations(ctx, "opsz=18,wdth=50")?;
  ctx.set_font_size(font_size);
  ctx.set_source_rgb_u32(0xffffff);
  ctx.translate(0.0, font_size);
  ctx.move_to(0.0, 0.0);
  ctx.show_text(&format!("{:.0}°", Num(weather.current.temp)))?;

  let font_size = 10.0;
  util::cairo::set_font_variations(ctx, "opsz=10,wdth=50")?;
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
