use super::*;

// Alternative representation of (L_cusp, C_cusp)
// Encoded so S = C_cusp/L_cusp and T = C_cusp/(1-L_cusp)
// The maximum value for C in the triangle is then found as fmin(S*L, T*(1-L)), for a given L
pub fn to_ST(L_cusp: f32, C_cusp: f32) -> (f32, f32) {
  let S = C_cusp / L_cusp;
  let T = C_cusp / (1.0 - L_cusp);
  (S, T)
}

// Returns a smooth approximation of the location of the cusp
// This polynomial was created by an optimization process
// It has been designed so that S_mid < S_max and T_mid < T_max
pub fn get_ST_mid(aʹ: f32, bʹ: f32) -> (f32, f32) {
  let x = -4.24894561 + 5.38770819 * bʹ + aʹ * 4.69891013;
  let x = -2.13704948 + -10.02301043 * bʹ + aʹ * x;
  let x = -2.19557347 + 1.75198401 * bʹ + aʹ * x;
  let x = 7.44778970 + 4.15901240 * bʹ + aʹ * x;
  let S = 0.11516993 + 1.0 / x;

  let x = 0.00299215 + -0.45399568 * bʹ + aʹ * -0.14661872;
  let x = -0.27087943 + 0.61223990 * bʹ + aʹ * x;
  let x = 0.40370612 + 0.90148123 * bʹ + aʹ * x;
  let x = 1.61320320 + -0.68124379 * bʹ + aʹ * x;
  let T = 0.11239642 + 1.0 / x;

  (S, T)
}

// finds L_cusp and C_cusp for a given hue
// a and b must be normalized so a^2 + b^2 == 1
pub fn find_cusp(a: f32, b: f32) -> (f32, f32) {
  // First, find the maximum saturation (saturation S = C/L)
  let S_cusp = compute_max_saturation(a, b);

  // Convert to linear sRGB to find the first point where at least one of r,g or b >= 1:
  let (L, a, b) = (1.0, S_cusp * a, S_cusp * b);
  let RGB { r, g, b } = Lab { L, a, b }.into();
  let L_cusp = (1.0 / r.max(g).max(b)).cbrt();
  let C_cusp = L_cusp * S_cusp;

  (L_cusp, C_cusp)
}

pub fn get_Cs(L: f32, aʹ: f32, bʹ: f32) -> (f32, f32, f32) {
  let (L_cusp, C_cusp) = find_cusp(aʹ, bʹ);

  let C_max = find_gamut_intersection(aʹ, bʹ, L, 1.0, L, L_cusp, C_cusp);
  let (S_max, T_max) = to_ST(L_cusp, C_cusp);

  // Scale factor to compensate for the curved part of gamut shape:
  let k = C_max / (L * S_max).min((1.0 - L) * T_max);

  let C_mid = {
    let (S_mid, T_mid) = get_ST_mid(aʹ, bʹ);

    // Use a soft minimum function, instead of a sharp triangle shape
    // to get a smooth value for chroma.
    let C_a = L * S_mid;
    let C_b = (1.0 - L) * T_mid;
    0.9 * k * (1.0 / (1.0 / C_a.powi(4) + 1.0 / C_b.powi(4))).sqrt().sqrt()
  };

  let C_0 = {
    // for C_0, the shape is independent of hue, so ST are constant.
    // Values picked to roughly be the average values of ST.
    let C_a = L * 0.4;
    let C_b = (1.0 - L) * 0.8;

    // Use a soft minimum function, instead of a sharp triangle shape
    // to get a smooth value for chroma.
    (1.0 / (1.0 / C_a.powi(2) + 1.0 / C_b.powi(2))).sqrt()
  };

  (C_0, C_mid, C_max)
}

// Finds the maximum saturation possible for a given hue that fits in sRGB
// Saturation here is defined as S = C/L
// a and b must be normalized so a^2 + b^2 == 1
pub fn compute_max_saturation(a: f32, b: f32) -> f32 {
  // Max saturation will be when one of r, g, or b goes below zero.

  // Select different coefficients depending on which component goes below zero first
  let (k, wl, wm, ws) = if -1.88170328 * a - 0.80936493 * b > 1.0 {
    let k = (1.19086277, 1.76576728, 0.59662641, 0.75515197, 0.56771245);
    (k, 4.0767416621, -3.3077115913, 0.2309699292) // Red component
  } else if 1.81444104 * a - 1.19445276 * b > 1.0 {
    let k = (0.73956515, -0.45954404, 0.08285427, 0.12541070, 0.14503204);
    (k, -1.2684380046, 2.6097574011, -0.3413193965) // Green component
  } else {
    let k = (1.35733652, -0.00915799, -1.15130210, -0.50559606, 0.00692167);
    (k, -0.0041960863, -0.7034186147, 1.7076147010) // Blue component
  };

  // Approximate max saturation using a polynomial:
  let S = k.0 + k.1 * a + k.2 * b + k.3 * a * a + k.4 * a * b;

  // Do one step Halley's method to get closer
  // this gives an error less than 10e6, except for some blue hues where the dS/dh is close to infinite
  // this should be sufficient for most applications, otherwise do two/three steps

  let k_l = 0.3963377774 * a + 0.2158037573 * b;
  let k_m = -0.1055613458 * a - 0.0638541728 * b;
  let k_s = -0.0894841775 * a - 1.2914855480 * b;

  let lʹ = 1.0 + S * k_l;
  let mʹ = 1.0 + S * k_m;
  let sʹ = 1.0 + S * k_s;

  let l = lʹ * lʹ * lʹ;
  let m = mʹ * mʹ * mʹ;
  let s = sʹ * sʹ * sʹ;

  let lʹdS = 3.0 * k_l * lʹ * lʹ;
  let mʹdS = 3.0 * k_m * mʹ * mʹ;
  let sʹdS = 3.0 * k_s * sʹ * sʹ;

  let lʹdS2 = 6.0 * k_l * k_l * lʹ;
  let mʹdS2 = 6.0 * k_m * k_m * mʹ;
  let sʹdS2 = 6.0 * k_s * k_s * sʹ;

  let f = wl * l + wm * m + ws * s;
  let f1 = wl * lʹdS + wm * mʹdS + ws * sʹdS;
  let f2 = wl * lʹdS2 + wm * mʹdS2 + ws * sʹdS2;

  S - f * f1 / (f1 * f1 - 0.5 * f * f2)
}

// Finds intersection of the line defined by
// L = L0 * (1 - t) + t * L1;
// C = t * C1;
// a and b must be normalized so a^2 + b^2 == 1
pub fn find_gamut_intersection(a: f32, b: f32, L1: f32, C1: f32, L0: f32, L_cusp: f32, C_cusp: f32) -> f32 {
  // Find the intersection for upper and lower half separately
  // Lower half
  if (L1 - L0) * C_cusp - (L_cusp - L0) * C1 <= 0.0 {
    C_cusp * L0 / (C1 * L_cusp + C_cusp * (L0 - L1))
  }
  // Upper half
  else {
    // First intersect with the triangle
    let mut t = C_cusp * (L0 - 1.0) / (C1 * (L_cusp - 1.0) + C_cusp * (L0 - L1));

    // Then one step Halley's method
    let dL = L1 - L0;
    let dC = C1;

    let k_l = 0.3963377774 * a + 0.2158037573 * b;
    let k_m = -0.1055613458 * a - 0.0638541728 * b;
    let k_s = -0.0894841775 * a - 1.2914855480 * b;

    let l_dt = dL + dC * k_l;
    let m_dt = dL + dC * k_m;
    let s_dt = dL + dC * k_s;

    // If higher accuracy is required, 2 or 3 iterations of the following block can be used:
    {
      let L = L0 * (1.0 - t) + t * L1;
      let C = t * C1;

      let lʹ = L + C * k_l;
      let mʹ = L + C * k_m;
      let sʹ = L + C * k_s;

      let l = lʹ * lʹ * lʹ;
      let m = mʹ * mʹ * mʹ;
      let s = sʹ * sʹ * sʹ;

      let ldt = 3.0 * l_dt * lʹ * lʹ;
      let mdt = 3.0 * m_dt * mʹ * mʹ;
      let sdt = 3.0 * s_dt * sʹ * sʹ;

      let ldt2 = 6.0 * l_dt * l_dt * lʹ;
      let mdt2 = 6.0 * m_dt * m_dt * mʹ;
      let sdt2 = 6.0 * s_dt * s_dt * sʹ;

      let f = |k0: f32, k1: f32, k2: f32| {
        let x = k0 * l + k1 * m + k2 * s - 1.0;
        let x1 = k0 * ldt + k1 * mdt + k2 * sdt;
        let x2 = k0 * ldt2 + k1 * mdt2 + k2 * sdt2;

        let u_x = x1 / (x1 * x1 - 0.5 * x * x2);
        let t_x = -x * u_x;
        let t_x = if u_x >= 0.0 { t_x } else { f32::MAX };
        t_x
      };

      let t_r = f(4.0767416621, -3.3077115913, 0.2309699292);
      let t_g = f(-1.2684380046, 2.6097574011, -0.3413193965);
      let t_b = f(-0.0041960863, -0.7034186147, 1.7076147010);

      t += t_r.min(t_g).min(t_b);
    }

    t
  }
}

pub mod toe {
  const K1: f32 = 0.206;
  const K2: f32 = 0.03;
  const K3: f32 = (1.0 + K1) / (1.0 + K2);

  pub fn f(x: f32) -> f32 {
    let k = K3 * x - K1;
    0.5 * (k + (k * k + 4.0 * K2 * K3 * x).sqrt())
  }

  pub fn inv(x: f32) -> f32 {
    (x * x + K1 * x) / (K3 * (x + K2))
  }
}

pub mod srgb_transfer_function {
  pub fn f(a: f32) -> f32 {
    match a {
      a if 0.0031308 < a => 1.055 * a.powf(0.4166666666666667) - 0.055,
      a => 12.92 * a,
    }
  }

  pub fn inv(a: f32) -> f32 {
    match a {
      a if 0.04045 < a => ((a + 0.055) / 1.055).powf(2.4),
      a => a / 12.92,
    }
  }
}
