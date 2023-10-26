#![feature(array_chunks)]

use std::fs;
use std::fs::File;

use c::fontconfig;
use rayon::prelude::*;

mod color;
mod render;
mod sky;
mod api {
  pub mod openweather;
  pub mod pirateweather;
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
  fontconfig::add_dir("assets/fonts")?;

  fs::read_dir("_/openweather")?
    .par_bridge()
    .map(|entry| {
      let entry = entry?;
      let path = entry.path();
      let name = path
        .file_name()
        .and_then(|s| Some(s.to_str()?.rsplit_once('.')?.0))
        .unwrap();

      println!("{}", name);

      let json = fs::read(&path)?;
      let weather = serde_json::from_slice(&json)?;

      let img = render::render(name, weather)?;

      let path = format!("_/openweather_png/{name}.png");
      let mut file = File::create(path)?;
      img.write_to_png(&mut file)?;

      Result::Ok(())
    })
    .try_for_each(|res| res)?;

  Ok(())
}
