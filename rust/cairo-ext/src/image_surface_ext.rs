use cairo::*;

use crate::blur;

pub trait ImageSurfaceExt {
  fn gaussian_blur(&mut self, σ: f64) -> Result<()>;
  fn gaussian_blur_fast(&mut self, σ: f64, n: usize) -> Result<()>;
  fn resize(&self, filter: Filter, w: i32, h: i32) -> Result<Self>
  where
    Self: Sized;
}

impl ImageSurfaceExt for ImageSurface {
  fn gaussian_blur(&mut self, σ: f64) -> Result<()> {
    blur::accurate::gaussian_blur(self, σ)
  }

  fn gaussian_blur_fast(&mut self, σ: f64, n: usize) -> Result<()> {
    blur::fast::gaussian_blur(self, σ, n)
  }

  fn resize(&self, filter: Filter, w: i32, h: i32) -> Result<Self> {
    let dst = Self::create(self.format(), w, h)?;
    let ctx = Context::new(&dst)?;

    let pat = SurfacePattern::create(self);
    pat.set_filter(filter);

    let sx = dst.width() as f64 / self.width() as f64;
    let sy = dst.height() as f64 / self.height() as f64;
    ctx.scale(sx, sy);
    ctx.set_source(pat)?;
    ctx.paint()?;

    Ok(dst)
  }
}
