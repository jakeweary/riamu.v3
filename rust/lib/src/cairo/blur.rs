use std::f32::consts::TAU;
use std::{array, iter, slice};

use cairo::*;

pub fn gaussian_blur(srf: &mut ImageSurface, sigma: f32) -> Result<()> {
  let width = srf.width();
  let height = srf.height();

  // NOTE: this buffer is transposed to make it a bit more cache-friendly on the 2nd step
  let mut tmp = ImageSurface::create(Format::ARgb32, height, width)?;
  let mut tmp = tmp.data().unwrap();
  let mut srf = srf.data().unwrap();

  // https://desmos.com/calculator/st8xmj1ig7
  let scale = 4.0;
  let radius = (scale * sigma).ceil() as i32;
  let gauss = {
    let k0 = 1.0 / (sigma * TAU.sqrt());
    let k1 = 0.5 / (sigma * sigma);
    let f = |x: f32| k0 * (-k1 * x * x).exp();
    let lut: Vec<_> = (0..=radius).map(|i| f(i as f32)).collect();
    move |x: i32| unsafe { *lut.get_unchecked(x.unsigned_abs() as usize) }
  };

  let srgb_eotf = {
    let lut: [_; 0x100] = array::from_fn(|i| srgb::eotf(i as u8));
    move |x: u8| unsafe { *lut.get_unchecked(x as usize) }
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
        let mut sum = 0.0;
        for (&src, dx) in iter::zip(src, x_min - x..) {
          let g = gauss(dx);
          acc[0] += g * srgb_eotf(src[0]);
          acc[1] += g * srgb_eotf(src[1]);
          acc[2] += g * srgb_eotf(src[2]);
          acc[3] += g * srgb_eotf(src[3]);
          sum += g;
        }

        let a = srgb::oetf(acc[0] / sum);
        let b = srgb::oetf(acc[1] / sum);
        let c = srgb::oetf(acc[2] / sum);
        let d = srgb::oetf(acc[3] / sum);
        *dst = [a, b, c, d];
      }
    }
  };

  // step 1: horizontal blur (srf → tmp)
  blur(&mut tmp, &srf, width, height);

  // step 2: verical blur (srf ← tmp)
  blur(&mut srf, &tmp, height, width);

  Ok(())
}

mod srgb {
  use crate::color::srgb::f32;

  pub fn eotf(x: u8) -> f32 {
    f32::eotf(x as f32 / 0xff as f32)
  }

  pub fn oetf(x: f32) -> u8 {
    (f32::oetf(x) * 0x100 as f32) as u8
  }
}
