use serenity::all::*;
use util::task;

use crate::client::{err, Context, Result};
use crate::db::statuses;

#[macros::command(desc = "Show one month of someone's status history")]
pub async fn run(
  ctx: &Context<'_>,
  #[desc = "The user of interest"] user: &User,
  #[desc = "The time zone (UTC offset in hours or in ±HHMM format, e.g.: -7, +3, +0530, +1245)"] tz: i64,
) -> Result<()> {
  let Some(now) = time::tz_offset(tz as i32).and_then(time::now) else {
    err::message!("invalid UTC offset");
  };

  ctx.event.defer(ctx).await?;

  tracing::debug!("querying database…");
  let statuses = statuses::query(&ctx.client.db, user.id, "-30 days").await?;

  tracing::debug!("rendering image…");
  let png = task::spawn_blocking(move || -> Result<_> {
    let mut png = Vec::new();
    status_history::render(now, &statuses)?.write_to_png(&mut png)?;
    Ok(png)
  })
  .await??;

  let file = CreateAttachment::bytes(png, "status history.png");
  let edit = EditInteractionResponse::new().new_attachment(file);

  tracing::debug!("sending response…");
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

// ---

mod time {
  use chrono::{prelude::*, Days};

  pub fn now(tz_offset: i32) -> Option<DateTime<FixedOffset>> {
    let offset = FixedOffset::east_opt(tz_offset)?;
    Some(Utc::now().with_timezone(&offset))
  }

  pub fn next_day<Tz: TimeZone>(dt: DateTime<Tz>) -> DateTime<Tz> {
    dt.checked_add_days(Days::new(1))
      .and_then(|dt| dt.with_hour(0)?.with_minute(0)?.with_second(0)?.with_nanosecond(0))
      .unwrap()
  }

  pub fn tz_offset(input: i32) -> Option<i32> {
    let (h, m) = match (input / 100, input % 100) {
      (0, h) => (h, 0),
      (h, m) => (h, m),
    };

    match (h, m) {
      (-23..=23, -59..=59) => Some(60 * 60 * h + 60 * m),
      _ => None,
    }
  }
}

mod status_history {
  use std::f64::consts::TAU as τ;
  use std::fmt::Display;
  use std::slice;

  use cairo::{Result, *};
  use cairo_ext::{ContextExt, ImageSurfaceExt};
  use chrono::{prelude::*, Days};
  use serenity::all::*;

  use crate::db::statuses;

  use super::time;

  const SCALE: i32 = 8;
  const IMAGE_W: i32 = 550;
  const IMAGE_H: i32 = 350;
  const CELL_W: i32 = 19;
  const CELL_H: i32 = 10;

  pub fn render<Tz>(dt: DateTime<Tz>, statuses: &[statuses::Row]) -> Result<ImageSurface>
  where
    Tz: TimeZone,
    Tz::Offset: Display,
  {
    let img = ImageSurface::create(Format::Rgb24, SCALE * IMAGE_W, SCALE * IMAGE_H)?;
    let ctx = cairo::Context::new(&img)?;

    ctx.scale1(SCALE as f64);
    ctx.select_font_face("sans", FontSlant::Normal, FontWeight::Normal);
    ctx.set_font_size(9.0);

    ctx.set_source_rgb_u32(0x313338);
    ctx.paint()?;

    let offset_min = dt.offset().fix().local_minus_utc() / 60;
    let fmt = if offset_min % 60 == 0 { "UTC%:::z" } else { "%:z" };
    let tz = dt.format(fmt).to_string().replace('-', "\u{2212}");
    let ext = ctx.text_extents(&tz)?;
    ctx.move_to(IMAGE_W as f64 - ext.x_advance() - 12.0, IMAGE_H as f64 - 12.0);
    ctx.set_source_rgb_u32(0x949ba4);
    ctx.show_text(&tz)?;

    ctx.translate(24.0, 12.0);
    ctx.set_source_rgb_u32(0xffffff);

    // ---

    {
      let img = status_history_data(dt.clone(), statuses)?;
      let img = img.resize(Filter::Good, SCALE * CELL_W * 24, 30)?;
      let pat = SurfacePattern::create(&img);
      pat.set_filter(Filter::Nearest);

      ctx.save()?;
      ctx.scale(1.0 / SCALE as f64, CELL_H as f64);
      ctx.set_source(pat)?;
      ctx.paint()?;
      ctx.restore()?;
    }

    // ---

    ctx.save()?;
    ctx.translate(8.0 + 24.0 * CELL_W as f64, 8.0);
    for day in 0..30 {
      let text = match 29 - day {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        d => {
          let dt = dt.clone().checked_sub_days(Days::new(d)).unwrap();
          let day = dt.format("%e").to_string().replace(' ', "\u{2007}");
          let mon = dt.format("%b");
          format!("{day} {mon}")
        }
      };

      ctx.move_to(0.0, day as f64 * CELL_H as f64);
      ctx.show_text(&text)?;
    }
    ctx.restore()?;

    ctx.save()?;
    ctx.translate(0.0, 30.0 * CELL_H as f64);
    for hour in 0..=24 {
      let text = format!("{:02}:00", hour);
      let ext = ctx.text_extents(&text)?;
      ctx.save()?;
      ctx.translate(hour as f64 * CELL_W as f64, 16.0);
      ctx.rotate(τ * -1.0 / 12.0);
      ctx.move_to(-ext.width() / 2.0, ext.height() / 2.0);
      ctx.show_text(&text)?;
      ctx.restore()?;
    }
    ctx.restore()?;

    // ---

    let x0 = 0.0;
    let x1 = 1.0 + 24.0 * CELL_W as f64;
    for day in 0..=30 {
      let y = 0.5 + day as f64 * CELL_H as f64;
      ctx.move_to(x0, y);
      ctx.line_to(x1, y);
    }

    let y0 = 0.0;
    let y1 = 1.0 + 30.0 * CELL_H as f64;
    for hour in 0..=24 {
      let x = 0.5 + hour as f64 * CELL_W as f64;
      ctx.move_to(x, y0);
      ctx.line_to(x, y1);
    }

    ctx.set_source_rgb_u32(0x1e1f22);
    ctx.set_line_width(1.0);
    ctx.stroke()?;

    Ok(img)
  }

  fn status_history_data<Tz>(dt: DateTime<Tz>, statuses: &[statuses::Row]) -> Result<ImageSurface>
  where
    Tz: TimeZone,
  {
    let now = dt.timestamp();
    let tomorrow = time::next_day(dt).timestamp();

    let px_sec = 5; // seconds per pixel
    let cell_px = 60 * 60 / px_sec; // pixels per cell
    let gap_px = 60 * 60 / px_sec / (CELL_W - 1) as usize; // pixels per gap

    let w = 24 * (cell_px + gap_px) as i32;
    let h = 30;

    let mut img = ImageSurface::create(Format::ARgb32, w, h)?;
    let mut data = img.data().unwrap();

    let data_u32 = {
      let ptr = data.as_mut_ptr() as *mut u32;
      let len = data.len() / 4;
      unsafe { slice::from_raw_parts_mut(ptr, len) }
    };

    let mut draw = |status: statuses::Packed, start, end| {
      let i0 = (tomorrow - end) as usize / px_sec;
      let i1 = (tomorrow - start) as usize / px_sec;

      let i0 = i0 + i0 / cell_px * gap_px; // gaps between each hour
      let i1 = i1 + i1 / cell_px * gap_px;

      let i0 = i0.min(data_u32.len());
      let i1 = i1.min(data_u32.len());

      let rgb = match status.status() {
        OnlineStatus::Offline => 0x3f4248,
        OnlineStatus::Online => 0x23a55a,
        OnlineStatus::Idle => 0xf0b232,
        OnlineStatus::DoNotDisturb => 0xf23f43,
        _ => panic!(),
      };

      data_u32[i0..i1].fill(0xff000000 | rgb);
    };

    for [start, end] in statuses.array_windows() {
      draw(start.status, start.time, end.time);
    }

    if let Some(last) = statuses.last() {
      draw(last.status, last.time, now);
    }

    data_u32.reverse();
    drop(data);

    Ok(img)
  }
}
