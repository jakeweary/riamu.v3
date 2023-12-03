use cairo::*;

pub fn text_width(ctx: &Context, parts: &[&str]) -> Result<f64> {
  parts.iter().try_fold(0.0, |acc, part| {
    let ext = ctx.text_extents(part)?;
    Ok(acc + ext.x_advance())
  })
}

pub fn resize(src: &ImageSurface, filter: Filter, w: i32, h: i32) -> Result<ImageSurface> {
  let dst = ImageSurface::create(Format::ARgb32, w, h)?;
  let ctx = Context::new(&dst)?;

  let pat = SurfacePattern::create(src);
  pat.set_filter(filter);

  let sx = dst.width() as f64 / src.width() as f64;
  let sy = dst.height() as f64 / src.height() as f64;
  ctx.scale(sx, sy);
  ctx.set_source(pat)?;
  ctx.paint()?;

  Ok(dst)
}
