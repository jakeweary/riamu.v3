#![allow(non_snake_case)]

use std::f32::consts::TAU as π;

use glam::Vec3;

use crate::sky::math;

const NUM_SAMPLES: i32 = 16;
const NUM_SAMPLES_LIGHT: i32 = 8;

const BETA_R: Vec3 = Vec3::new(3.8e-6, 13.5e-6, 33.1e-6);
const BETA_M: Vec3 = Vec3::splat(21e-6);

pub struct Atmosphere {
  /// The sun direction (normalized)
  pub sun: Vec3,
  /// In the paper this is usually Rg or Re (radius ground, earth)
  pub Re: f32,
  /// In the paper this is usually R or Ra (radius atmosphere)
  pub Ra: f32,
  /// Thickness of the atmosphere if density was uniform (Hr)
  pub Hr: f32,
  /// Same as above but for Mie scattering (Hm)
  pub Hm: f32,
}

impl Default for Atmosphere {
  fn default() -> Self {
    let sun = Vec3::Y;
    let (Re, Ra) = (6360e3, 6420e3);
    let (Hr, Hm) = (7994e0, 1200e0);
    Self { sun, Re, Ra, Hr, Hm }
  }
}

impl Atmosphere {
  pub fn compute_incident_light(&self, orig: Vec3, dir: Vec3, tmin: f32, tmax: f32) -> Option<Vec3> {
    let (t0, t1) = math::ray_sphere_intersect(orig, dir, self.Ra)?;
    let tmin = if t0 > tmin && t0 > 0.0 { t0 } else { tmin };
    let tmax = if t1 < tmax { t1 } else { tmax };

    let segment_length = (tmax - tmin) / NUM_SAMPLES as f32;
    let mut t_current = tmin;

    // rayleigh and mie contribution
    let mut sum_r = Vec3::ZERO;
    let mut sum_m = Vec3::ZERO;

    let mut optical_depth_r = 0.0;
    let mut optical_depth_m = 0.0;

    // mu in the paper which is the cosine of the angle between the sun direction and the ray direction
    let mu = dir.dot(self.sun);

    let phase_r = 3.0 / (16.0 * π) * (1.0 + mu * mu);
    let phase_m = {
      let g = 0.76;
      let n = 3.0 / (8.0 * π) * ((1.0 - g * g) * (1.0 + mu * mu));
      let m = (2.0 + g * g) * (1.0 + g * g - 2.0 * g * mu).powf(1.5);
      n / m
    };

    for _i in 0..NUM_SAMPLES {
      let sample_position = orig + (t_current + segment_length * 0.5) * dir;
      let height = sample_position.length() - self.Re;

      // compute optical depth for light
      let hr = (-height / self.Hr).exp() * segment_length;
      let hm = (-height / self.Hm).exp() * segment_length;

      optical_depth_r += hr;
      optical_depth_m += hm;

      // light optical depth
      let (_t0_light, t1_light) = math::ray_sphere_intersect(sample_position, self.sun, self.Ra).unwrap();

      let segment_length_light = t1_light / NUM_SAMPLES_LIGHT as f32;

      let mut t_current_light = 0.0;
      let mut optical_depth_light_r = 0.0;
      let mut optical_depth_light_m = 0.0;

      let mut j = 0;

      while j < NUM_SAMPLES_LIGHT {
        let sample_position_light = sample_position + (t_current_light + segment_length_light * 0.5) * self.sun;
        let height_light = sample_position_light.length() - self.Re;
        if height_light < 0.0 {
          break;
        }
        optical_depth_light_r += (-height_light / self.Hr).exp() * segment_length_light;
        optical_depth_light_m += (-height_light / self.Hm).exp() * segment_length_light;
        t_current_light += segment_length_light;
        j += 1
      }

      if j == NUM_SAMPLES_LIGHT {
        let tau_r = BETA_R * (optical_depth_r + optical_depth_light_r);
        let tau_m = BETA_M * (optical_depth_m + optical_depth_light_m) * 1.1;
        let tau = tau_r + tau_m;
        let attenuation = (-tau).exp();
        sum_r += attenuation * hr;
        sum_m += attenuation * hm;
      }

      t_current += segment_length;
    }

    sum_r *= BETA_R * phase_r;
    sum_m *= BETA_M * phase_m;

    // We use a magic number here for the intensity of the sun (20). We will make it more
    // scientific in a future revision of this lesson/code
    Some((sum_r + sum_m) * 20.0)
  }
}
