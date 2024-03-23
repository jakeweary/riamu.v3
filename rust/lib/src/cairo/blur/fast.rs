use std::slice;
use std::{cmp::max, iter::zip, mem::swap};

use cairo::*;

use crate::color::srgb::f32_to_srgb8_v2 as f32_to_srgb8;
use crate::color::srgb::srgb8_to_f32;

// https://blog.ivank.net/fastest-gaussian-blur.html
// http://elynxsdk.free.fr/ext-docs/Blur/Fast_box_blur.pdf
pub fn gaussian_blur(srf: &mut ImageSurface, σ: f64, n: usize) -> Result<()> {
  gaussian_blur_xy(srf, [σ; 2], [n; 2])
}

pub fn gaussian_blur_xy(srf: &mut ImageSurface, [σx, σy]: [f64; 2], [nx, ny]: [usize; 2]) -> Result<()> {
  let (Format::Rgb24 | Format::ARgb32) = srf.format() else {
    panic!("unsupported image format");
  };

  let width = srf.width() as usize;
  let height = srf.height() as usize;
  let (scale_x, scale_y) = srf.device_scale();

  let mut srf = srf.data().unwrap();
  let srf: &mut [[u8; 4]] = {
    let len = srf.len() / 4;
    let ptr = srf.as_mut_ptr().cast();
    unsafe { slice::from_raw_parts_mut(ptr, len) }
  };

  let mid = max(width, height);
  let mut tmp = vec![[0.0; 3]; 2 * mid];
  let (mut tmp0, mut tmp1) = tmp.split_at_mut(mid);

  let mut blur = |size, stride, σ, n| {
    let [width, height] = size;
    let [stride_x, stride_y] = stride;
    let (_, box_blur_widths) = box_blur_widths(σ, n);

    for y in 0..height {
      let i = stride_y * y;

      // read current row of pixels
      let row = srf[i..].iter_mut().step_by(stride_x);
      for (dst, src) in zip(&mut tmp0[..width], row) {
        dst[0] = srgb8_to_f32(src[0]);
        dst[1] = srgb8_to_f32(src[1]);
        dst[2] = srgb8_to_f32(src[2]);
      }

      // n iterations of box blur
      for (w, nʹ) in box_blur_widths {
        let wʹ = 1.0 / w as f32;
        let r = w / 2;

        for _ in 0..nʹ {
          // cumulative sum
          let mut acc = [0.0; 3];
          for rgb in &mut *tmp0 {
            acc[0] += rgb[0];
            acc[1] += rgb[1];
            acc[2] += rgb[2];
            *rgb = acc;
          }

          // the left edge
          let dst = &mut tmp1[0..1 + r];
          let src = &tmp0[r..width];
          for (i, (dst, src)) in zip(dst, src).enumerate() {
            let wʹ = 1.0 / (r + i + 1) as f32;
            let r = wʹ * src[0];
            let g = wʹ * src[1];
            let b = wʹ * src[2];
            *dst = [r, g, b];
          }

          // the middle
          let dst = &mut tmp1[1 + r..width];
          let src0 = &tmp0[0..width];
          let src1 = &tmp0[w..width];
          for (dst, (src0, src1)) in zip(dst, zip(src0, src1)) {
            let r = wʹ * (src1[0] - src0[0]);
            let g = wʹ * (src1[1] - src0[1]);
            let b = wʹ * (src1[2] - src0[2]);
            *dst = [r, g, b];
          }

          // the right edge
          let dst = &mut tmp1[width - r..width];
          let src0 = &tmp0[width - w..width];
          let src1 = &tmp0[width - 1];
          for (i, (dst, src0)) in zip(dst, src0).enumerate() {
            let wʹ = 1.0 / (w - i - 1) as f32;
            let r = wʹ * (src1[0] - src0[0]);
            let g = wʹ * (src1[1] - src0[1]);
            let b = wʹ * (src1[2] - src0[2]);
            *dst = [r, g, b];
          }

          swap(&mut tmp0, &mut tmp1);
        }
      }

      // write back current row of pixels
      let row = srf[i..].iter_mut().step_by(stride_x);
      for (src, dst) in zip(&tmp0[..width], row) {
        dst[0] = f32_to_srgb8(src[0]);
        dst[1] = f32_to_srgb8(src[1]);
        dst[2] = f32_to_srgb8(src[2]);
      }
    }
  };

  if σx > 0.0 {
    blur([width, height], [1, width], σx * scale_x.abs(), nx); // horizontal
  }

  if σy > 0.0 {
    blur([height, width], [width, 1], σy * scale_y.abs(), ny); // vertical
  }

  Ok(())
}

// https://peterkovesi.com/papers/FastGaussianSmoothing.pdf
// https://peterkovesi.com/matlabfns/Spatial/solveinteg.m
// https://peterkovesi.com/matlabfns/
fn box_blur_widths(σ: f64, n: usize) -> (f64, [(usize, usize); 2]) {
  let n = n as f64;

  let w_ideal = (12.0 * σ * σ / n + 1.0).sqrt();
  let w = 2.0 * (0.5 * w_ideal).round();
  let (wl, wu) = (w - 1.0, w + 1.0);

  let m_ideal = (12.0 * σ * σ - n * wl * wl - 4.0 * n * wl - 3.0 * n) / -(4.0 * wl + 4.0);
  let m = m_ideal.round();

  let σ_actual = ((m * wl * wl + (n - m) * wu * wu - n) / 12.0).sqrt();

  let w0 = (wl as usize, m as usize);
  let w1 = (wu as usize, (n - m) as usize);
  (σ_actual, [w0, w1])
}
