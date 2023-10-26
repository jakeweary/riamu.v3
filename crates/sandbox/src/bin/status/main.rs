#![feature(array_windows)]

use chrono::Utc;
use image::imageops::{self as ops, FilterType};
use image::{ImageBuffer, Rgb};
use rusqlite::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let now = Utc::now().timestamp();

  let db = Connection::open("db.sqlite")?;

  let mut stmt = db.prepare("SELECT id, name FROM user;")?;
  let rows = stmt
    .query_map([], |row| {
      let id: i64 = row.get(0)?;
      let name: Option<String> = row.get(1)?;
      Ok((id, name))
    })?
    .collect::<Result<Vec<_>, _>>()?;

  for (id, name) in rows {
    let mut stmt = db.prepare("SELECT time, status FROM status WHERE user = ?;")?;
    let mut rows = stmt
      .query_map([id], |row| {
        let time: i64 = row.get(0)?;
        let status: String = row.get(1)?;
        Ok((time, status))
      })?
      .collect::<Result<Vec<_>, _>>()?;

    rows.push((now, "".into()));

    let px = 40; // seconds in one pixel
    let w = 24 * 60 * 60 / px;
    let h = 30;
    let mut buf = ImageBuffer::from_pixel(w, h, Rgb([0x2b, 0x2d, 0x31]));

    for [prev, curr] in rows.array_windows() {
      let (prev_time, prev_status) = prev;
      let (curr_time, _) = curr;

      let color: Rgb<u8> = match &**prev_status {
        "offline" => Rgb([0x80, 0x84, 0x8e]),
        "online" => Rgb([0x23, 0xa5, 0x5a]),
        "idle" => Rgb([0xf0, 0xb2, 0x32]),
        "dnd" => Rgb([0xf2, 0x3f, 0x43]),
        _ => panic!(),
      };

      let a = (now - prev_time) as u32 / px;
      let b = (now - curr_time) as u32 / px;
      for i in b..=a {
        if let Some(px) = buf.get_pixel_mut_checked(i % w, i / w) {
          *px = color;
        }
      }
    }

    ops::rotate180_in_place(&mut buf);
    let buf = ops::resize(&buf, w, h * 32, FilterType::Nearest);

    let w = buf.width() + 40;
    let h = buf.height() + 40;
    let mut img = ImageBuffer::from_pixel(w, h, Rgb([0x2b, 0x2d, 0x31]));
    ops::overlay(&mut img, &buf, 20, 20);

    let path = format!("_/status{}_{}.png", id, name.unwrap_or_default());
    println!("{path:?}");
    img.save(path)?;
  }

  Ok(())
}
