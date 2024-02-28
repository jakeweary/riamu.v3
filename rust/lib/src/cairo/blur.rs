use std::f32::consts::TAU as τ;
use std::{iter, slice};

use cairo::*;

use crate::color::srgb::f32_to_srgb8_v2 as srgb_oetf;
use crate::color::srgb::srgb8_to_f32 as srgb_eotf;

pub fn gaussian_blur(srf: &mut ImageSurface, σ: f32) -> Result<()> {
  assert_eq!(srf.format(), Format::Rgb24);

  let width = srf.width();
  let height = srf.height();

  // NOTE: this buffer is transposed to make it a bit more cache-friendly on the 2nd step
  let mut tmp = ImageSurface::create(Format::Rgb24, height, width)?;
  let mut tmp = tmp.data().unwrap();
  let mut srf = srf.data().unwrap();

  // https://desmos.com/calculator/st8xmj1ig7
  let scale = 4.0;
  let radius = (scale * σ).ceil() as i32;
  let gauss = {
    let k0 = 1.0 / (σ * τ.sqrt());
    let k1 = 0.5 / (σ * σ);
    let f = |x: f32| k0 * (-k1 * x * x).exp();
    let lut: Vec<_> = (0..=radius).map(|i| f(i as f32)).collect();
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
        for (&[r, g, b, _], dx) in iter::zip(src, x_min - x..) {
          let gʹ = gauss(dx);
          acc[0] += gʹ * srgb_eotf(r);
          acc[1] += gʹ * srgb_eotf(g);
          acc[2] += gʹ * srgb_eotf(b);
          acc[3] += gʹ;
        }

        let [r, g, b, gʹ] = acc;
        let r = srgb_oetf(r / gʹ);
        let g = srgb_oetf(g / gʹ);
        let b = srgb_oetf(b / gʹ);
        *dst = [r, g, b, 0xff];
      }
    }
  };

  // step 1: horizontal blur (srf → tmp)
  blur(&mut tmp, &srf, width, height);

  // step 2: verical blur (srf ← tmp)
  blur(&mut srf, &tmp, height, width);

  Ok(())
}
