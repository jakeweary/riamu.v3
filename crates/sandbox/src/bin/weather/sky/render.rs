use cairo::{Format, ImageSurface};
use glam::{EulerRot, Quat, Vec3};

use crate::color;

use super::{math, nishita};

pub fn fisheye(w: i32, h: i32, sun_zenith_angle: f32) -> cairo::Result<ImageSurface> {
  let (sin, cos) = sun_zenith_angle.to_radians().sin_cos();
  let atm = nishita::Atmosphere {
    sun: Vec3::new(0.0, cos, -sin),
    ..Default::default()
  };

  let fov = 120_f32.to_radians();
  let ratio = w as f32 / h as f32;
  let ro = Vec3::new(0.0, atm.Re + 100.0, 0.0);

  let mut img = ImageSurface::create(Format::ARgb32, w, h)?;

  for (i, bgra) in img.data().unwrap().array_chunks_mut::<4>().enumerate() {
    let x = i % w as usize;
    let y = i / w as usize;

    let nx = (x as f32 + 0.5) / w as f32 - 0.5;
    let ny = (y as f32 + 0.5) / h as f32 - 0.5;

    let rx = 45_f32.to_radians() - fov * ny;
    let ry = ratio * fov * nx;
    let rd = Quat::from_euler(EulerRot::XYZ, rx, ry, 0.0) * Vec3::NEG_Z;

    // Does the ray intersect the planetory body? (the intersection test is against the Earth here
    // not against the atmosphere). If the ray intersects the Earth body and that the intersection
    // is ahead of us, then the ray intersects the planet in 2 points, t0 and t1. But we
    // only want to comupute the atmosphere between t=0 and t=t0 (where the ray hits
    // the Earth first). If the viewing ray doesn't hit the Earth, or course the ray
    // is then bounded to the range [0:INF]. In the method computeIncidentLight() we then
    // compute where this primary ray intersects the atmosphere and we limit the max t range
    // of the ray to the point where it leaves the atmosphere.
    let t_max = match math::ray_sphere_intersect(ro, rd, atm.Re) {
      Some((t0, t1)) if t1 > 0.0 => t0.max(0.0),
      _ => f32::INFINITY,
    };

    // The *viewing or camera ray* is bounded to the range [0:tMax]
    let rgb = atm.compute_incident_light(ro, rd, 0.0, t_max).unwrap_or_default();

    let rgb = color::aces(rgb);
    let [r, g, b] = color::to_bytes(rgb);
    *bgra = [b, g, r, 0xff];
  }

  Ok(img)
}
