use std::f64::consts::TAU as τ;
use std::iter::zip;
use std::slice;

use cairo::*;
use color::srgb::f32_to_srgb8_v2 as f32_to_srgb8;
use color::srgb::srgb8_to_f32;

// https://en.wikipedia.org/wiki/Gaussian_blur#Implementation
// https://en.wikipedia.org/wiki/Separable_filter
pub fn gaussian_blur(srf: &mut ImageSurface, σ: f64) -> Result<()> {
  let (Format::Rgb24 | Format::ARgb32) = srf.format() else {
    panic!("unsupported image format");
  };

  let σ = match srf.device_scale() {
    (x, y) if x.abs() == y.abs() => σ * x.abs(),
    _ => panic!("unsupported device scale"),
  };

  let width = srf.width();
  let height = srf.height();

  // NOTE: this buffer is transposed to make it more cache-friendly on the 2nd step
  let mut tmp = ImageSurface::create(Format::Rgb24, height, width)?;
  let mut tmp = tmp.data().unwrap();
  let mut srf = srf.data().unwrap();

  // https://desmos.com/calculator/st8xmj1ig7
  let scale = 4.0;
  let radius = (scale * σ).ceil() as i32;
  let gauss = {
    let k0 = 1.0 / (σ * τ.sqrt());
    let k1 = 0.5 / (σ * σ);
    let f = |x: f64| k0 * (-k1 * x * x).exp();
    let lut: Vec<_> = (0..=radius).map(|i| f(i as f64) as f32).collect();
    move |x: i32| unsafe { *lut.get_unchecked(x.unsigned_abs() as usize) }
  };

  let blur = |dst: &mut [u8], src: &[u8], width: i32, height: i32| {
    let dst = dst.as_mut_ptr() as *mut [u8; 4];
    let src = src.as_ptr() as *const [u8; 4];

    for y in 0..height {
      for x in 0..width {
        let x_min = (x - radius).max(0);
        let x_max = (x + radius + 1).min(width);

        let i = (x_min + y * width) as usize;
        let n = (x_max - x_min) as usize;
        let src = unsafe { slice::from_raw_parts(src.add(i), n) };

        let i = (y + x * height) as usize;
        let dst = unsafe { &mut *dst.add(i) };

        let mut acc = [0.0; 4];
        for (&[r, g, b, _], dx) in zip(src, x_min - x..) {
          let gʹ = gauss(dx);
          acc[0] += gʹ * srgb8_to_f32(r);
          acc[1] += gʹ * srgb8_to_f32(g);
          acc[2] += gʹ * srgb8_to_f32(b);
          acc[3] += gʹ;
        }

        let [r, g, b, gʹ] = acc;
        let r = f32_to_srgb8(r / gʹ);
        let g = f32_to_srgb8(g / gʹ);
        let b = f32_to_srgb8(b / gʹ);
        *dst = [r, g, b, 0xff];
      }
    }
  };

  blur(&mut tmp, &srf, width, height); // horizontal
  blur(&mut srf, &tmp, height, width); // vertical

  Ok(())
}
