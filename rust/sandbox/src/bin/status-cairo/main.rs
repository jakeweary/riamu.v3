#![feature(array_windows)]

use std::f64::consts::TAU as τ;
use std::fs::File;

use cairo::{Context, Filter, Format, ImageSurface, SurfacePattern};
use cairo::{FontSlant, FontWeight};
use chrono::{DateTime, Days, Local, TimeZone, Timelike};
use rusqlite::Connection;

use self::cairo_ext::ContextExt;

mod cairo_ext;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const SCALE: i32 = 1;
const IMAGE_W: i32 = 550;
const IMAGE_H: i32 = 350;
const CELL_W: i32 = 19;
const CELL_H: i32 = 10;

fn next_day<Tz: TimeZone>(dt: DateTime<Tz>) -> DateTime<Tz> {
  dt.checked_add_days(Days::new(1))
    .and_then(|dt| dt.with_hour(0)?.with_minute(0)?.with_second(0)?.with_nanosecond(0))
    .unwrap()
}

fn status_history_data(mut rows: Vec<(i64, String)>) -> Result<ImageSurface> {
  let dt = Local::now();
  let now = dt.timestamp();
  let tomorrow = next_day(dt).timestamp();

  rows.push((now, "".into()));

  let px = 60 * 60 / (CELL_W - 1) as usize; // seconds in one pixel
  let w = 24 * CELL_W;
  let h = 30;

  let mut img = ImageSurface::create(Format::ARgb32, w, h)?;
  let Ok(mut data) = img.data() else {
    unreachable!();
  };

  for [prev, curr] in rows.array_windows() {
    let (prev_time, prev_status) = prev;
    let (curr_time, _) = curr;

    let i0 = (tomorrow - curr_time) as usize / px;
    let i1 = (tomorrow - prev_time) as usize / px;

    if i0 != i1 {
      let color = match &**prev_status {
        "offline" => 0xff_80848e_u32.to_be_bytes(),
        "online" => 0xff_23a55a_u32.to_be_bytes(),
        "idle" => 0xff_f0b232_u32.to_be_bytes(),
        "dnd" => 0xff_f23f43_u32.to_be_bytes(),
        _ => panic!(),
      };

      for mut i in i0..i1 {
        i += i / (CELL_W - 1) as usize; // 1px gap between each hour
        i *= 4;
        data[i..i + 4].copy_from_slice(&color);
      }
    }
  }

  data.reverse();
  drop(data);

  Ok(img)
}

fn status_history(rows: Vec<(i64, String)>) -> Result<ImageSurface> {
  let img = ImageSurface::create(Format::Rgb24, SCALE * IMAGE_W, SCALE * IMAGE_H)?;
  let ctx = Context::new(&img)?;

  ctx.scale(SCALE as f64, SCALE as f64);
  ctx.translate(24.0, 12.0);

  ctx.set_source_rgb_u32(0x313338);
  ctx.paint()?;

  // ---

  let data = status_history_data(rows)?;
  let pat = SurfacePattern::create(&data);
  pat.set_filter(Filter::Nearest);

  ctx.save()?;
  ctx.scale(1.0, CELL_H as f64);
  ctx.set_source(pat)?;
  ctx.paint()?;
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

  // ---

  ctx.select_font_face("sans-serif", FontSlant::Normal, FontWeight::Normal);
  ctx.set_font_size(9.0);
  ctx.set_source_rgb(1.0, 1.0, 1.0);

  ctx.save()?;
  ctx.translate(8.0 + 24.0 * CELL_W as f64, 8.0);
  for day in 0..30 {
    let text = match 29 - day {
      0 => "Today".into(),
      1 => "Yesterday".into(),
      d => {
        let now = Local::now();
        let date = now.checked_sub_days(Days::new(d)).unwrap();
        format!("{}", date.format("%b %-d"))
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

  // ctx.save()?;
  // ctx.translate(0.0, 30.0 * CELL_H as f64);
  // for hour in 0..=24 {
  //   ctx.save()?;
  //   ctx.translate(hour as f64 * CELL_W as f64, 16.0);

  //   let text = format!("{}", 1 + (hour + 11) % 12);
  //   ctx.set_font_size(9.0);
  //   let ext = ctx.text_extents(&text)?;
  //   ctx.move_to(-ext.width() / 2.0, 0.0);
  //   ctx.show_text(&text)?;

  //   if hour % 12 == 0 {
  //     let text = if hour % 24 < 12 { "AM" } else { "PM" };
  //     ctx.set_font_size(7.5);
  //     let ext = ctx.text_extents(&text)?;
  //     ctx.move_to(-ext.width() / 2.0, 9.0);
  //     ctx.show_text(&text)?;
  //   }

  //   ctx.restore()?;
  // }
  // ctx.restore()?;

  Ok(img)
}

fn main() -> Result<()> {
  let db = Connection::open("db.sqlite")?;

  let mut stmt = db.prepare("SELECT id, name FROM user WHERE id IN (?);")?;
  let rows = stmt
    .query_map([(100355854062059520_u64)], |row| {
      let id: i64 = row.get(0)?;
      let name: Option<String> = row.get(1)?;
      Ok((id, name))
    })?
    .collect::<std::result::Result<Vec<_>, _>>()?;

  for (id, name) in rows {
    let mut stmt = db.prepare("SELECT time, status FROM status WHERE user = ?;")?;
    let rows = stmt
      .query_map([id], |row| {
        let time: i64 = row.get(0)?;
        let status: String = row.get(1)?;
        Ok((time, status))
      })?
      .collect::<std::result::Result<Vec<_>, _>>()?;

    let path = format!("_/status_{}_{}.png", id, name.unwrap_or_default());
    let mut file = File::create(&path)?;
    println!("{path:?}");

    let img = status_history(rows)?;
    img.write_to_png(&mut file)?;
  }

  Ok(())
}
