use std::f64::consts::TAU as τ;
use std::fmt::Display;

use cairo::{Filter, FontSlant, FontWeight, Format, ImageSurface, SurfacePattern};
use chrono::{DateTime, Days, FixedOffset, Local, Offset, TimeZone, Timelike, Utc};
use lib::{cairo::ext::*, task};
use serenity::all::*;

use crate::client::{err, Context, Result};
use crate::db::{self, statuses};

#[macros::command(description = "Show one month of someone's status history")]
pub async fn run(
  ctx: &Context<'_>,
  #[description = "The user of interest"] user: &User,
  #[description = "The time zone (UTC offset in hours or in ±HHMM format, e.g.: -7, +3, +0530, +1245)"] tz: i64,
) -> Result<()> {
  let Some(now) = tz_offset(tz as i32).and_then(now) else {
    err::message!("invalid UTC offset");
  };

  ctx.event.defer(ctx).await?;

  tracing::debug!("querying database…");
  let statuses = db::statuses::query(&ctx.client.db, user.id, "-30 days").await?;

  tracing::debug!("rendering image…");
  let png = task::spawn_blocking(move || -> Result<_> {
    let mut png = Vec::new();
    status_history(now, &statuses)?.write_to_png(&mut png)?;
    Ok(png)
  })
  .await??;

  let file = CreateAttachment::bytes(png, "status_history.png");
  let edit = EditInteractionResponse::new().new_attachment(file);

  tracing::debug!("sending response…");
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

// ---

const SCALE: i32 = 8;
const IMAGE_W: i32 = 550;
const IMAGE_H: i32 = 350;
const CELL_W: i32 = 19;
const CELL_H: i32 = 10;

fn status_history<Tz>(dt: DateTime<Tz>, statuses: &[db::statuses::Row]) -> cairo::Result<ImageSurface>
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

  let data = status_history_data(dt, statuses)?;
  let pat = SurfacePattern::create(&data);
  pat.set_filter(Filter::Nearest);

  ctx.save()?;
  ctx.scale(1.0, CELL_H as f64);
  ctx.set_source(pat)?;
  ctx.paint()?;
  ctx.restore()?;

  // ---

  ctx.save()?;
  ctx.translate(8.0 + 24.0 * CELL_W as f64, 8.0);
  for day in 0..30 {
    let text = match 29 - day {
      0 => "Today".into(),
      1 => "Yesterday".into(),
      d => {
        let date = Local::now().checked_sub_days(Days::new(d)).unwrap();
        let day = date.format("%e").to_string().replace(' ', "\u{2007}");
        let mon = date.format("%b");
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

fn status_history_data<Tz>(dt: DateTime<Tz>, statuses: &[db::statuses::Row]) -> cairo::Result<ImageSurface>
where
  Tz: TimeZone,
{
  let now = dt.timestamp();
  let tomorrow = next_day(dt).timestamp();

  let px = 60 * 60 / (CELL_W - 1) as usize; // seconds in one pixel
  let w = 24 * CELL_W;
  let h = 30;

  let mut img = ImageSurface::create(Format::ARgb32, w, h)?;
  let mut data = img.data().unwrap();

  let mut draw = |packed: statuses::Packed, start, end| {
    let i0 = (tomorrow - end) as usize / px;
    let i1 = (tomorrow - start) as usize / px;

    if i0 != i1 {
      let color = match packed.status() {
        OnlineStatus::Offline => 0xff_80848e_u32.to_be_bytes(),
        OnlineStatus::Online => 0xff_23a55a_u32.to_be_bytes(),
        OnlineStatus::Idle => 0xff_f0b232_u32.to_be_bytes(),
        OnlineStatus::DoNotDisturb => 0xff_f23f43_u32.to_be_bytes(),
        _ => panic!(),
      };

      for mut i in i0..i1 {
        i += i / (CELL_W - 1) as usize; // 1px gap between each hour
        i *= 4;
        match data.get_mut(i..i + 4) {
          Some(c) => c.copy_from_slice(&color),
          None => break,
        }
      }
    }
  };

  for [start, end] in statuses.array_windows() {
    draw(start.status, start.time, end.time);
  }

  if let Some(last) = statuses.last() {
    draw(last.status, last.time, now);
  }

  data.reverse();
  drop(data);

  Ok(img)
}

// ---

fn now(tz_offset: i32) -> Option<DateTime<FixedOffset>> {
  let offset = FixedOffset::east_opt(tz_offset)?;
  Some(Utc::now().with_timezone(&offset))
}

fn next_day<Tz: TimeZone>(dt: DateTime<Tz>) -> DateTime<Tz> {
  dt.checked_add_days(Days::new(1))
    .and_then(|dt| dt.with_hour(0)?.with_minute(0)?.with_second(0)?.with_nanosecond(0))
    .unwrap()
}

fn tz_offset(input: i32) -> Option<i32> {
  let (h, m) = match (input / 100, input % 100) {
    (0, h) => (h, 0),
    (h, m) => (h, m),
  };
  match (h, m) {
    (-23..=23, -59..=59) => Some(60 * 60 * h + 60 * m),
    _ => None,
  }
}
