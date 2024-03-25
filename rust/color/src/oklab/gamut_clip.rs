use super::*;

pub fn preserve_chroma(rgb: RGB) -> RGB {
  inner(rgb, |_, L, _| clamp(L, 0.0, 1.0))
}

pub fn project_to_0_5(rgb: RGB) -> RGB {
  inner(rgb, |_, _, _| 0.5)
}

pub fn project_to_L_cusp(rgb: RGB) -> RGB {
  inner(rgb, |_, _, L_cusp| L_cusp)
}

pub fn adaptive_L0_0_5(rgb: RGB, α: f32) -> RGB {
  inner(rgb, |C, L, _| {
    let Ld = L - 0.5;
    let e1 = 0.5 + Ld.abs() + α * C;
    0.5 * (1.0 + sgn(Ld) * (e1 - (e1 * e1 - 2.0 * Ld.abs()).sqrt()))
  })
}

pub fn adaptive_L0_L_cusp(rgb: RGB, α: f32) -> RGB {
  inner(rgb, |C, L, L_cusp| {
    let Ld = L - L_cusp;
    let k = 2.0 * if Ld > 0.0 { 1.0 - L_cusp } else { L_cusp };
    let e1 = 0.5 * k + Ld.abs() + α * C / k;
    L_cusp + 0.5 * (sgn(Ld) * (e1 - (e1 * e1 - 2.0 * k * Ld.abs()).sqrt()))
  })
}

// ---

fn inner(rgb: RGB, map: impl FnOnce(f32, f32, f32) -> f32) -> RGB {
  let in_gamut = |x| 0.0 <= x && x <= 1.0;
  if in_gamut(rgb.r) && in_gamut(rgb.g) && in_gamut(rgb.b) {
    return rgb;
  }

  let Lab { L, a, b } = rgb.into();
  let C = a.hypot(b).max(0.00001);
  let (aʹ, bʹ) = (a / C, b / C);
  let (L_cusp, C_cusp) = find_cusp(aʹ, bʹ);

  let L0 = map(C, L, L_cusp);

  let t = find_gamut_intersection(aʹ, bʹ, L, C, L0, L_cusp, C_cusp);
  let (L_clipped, C_clipped) = (L0 * (1.0 - t) + t * L, t * C);
  let (L, a, b) = (L_clipped, C_clipped * aʹ, C_clipped * bʹ);

  Lab { L, a, b }.into()
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
  match x {
    x if x < min => min,
    x if x > max => max,
    x => x,
  }
}

fn sgn(x: f32) -> f32 {
  1f32.copysign(x)
}
