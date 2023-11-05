use std::fs::File;
use std::time::Instant;

use lib::cairo::blur::gaussian_blur;
use lib::cairo::ext::ContextExt;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
  let mut file = File::create("_/blur.png")?;
  render()?.write_to_png(&mut file)?;

  Ok(())
}

fn render() -> Result<cairo::ImageSurface> {
  let mut img = cairo::ImageSurface::create(cairo::Format::Rgb24, 512, 512)?;
  let cc = cairo::Context::new(&img)?;

  cc.translate(img.width() as f64 / 2.0, img.height() as f64 / 2.0);

  cc.set_source_rgb_u32(0x1e1f22);
  cc.paint()?;

  cc.set_source_rgb_u32(0xffffff);
  cc.rectangle(-32.0, -32.0, 64.0, 64.0);
  cc.fill()?;

  drop(cc);

  let t = Instant::now();
  gaussian_blur(&mut img, 10.0)?;
  let t = t.elapsed();
  println!("{:?}", t);

  Ok(img)
}
