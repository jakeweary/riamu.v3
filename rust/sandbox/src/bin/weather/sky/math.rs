use glam::Vec3;

// https://stackoverflow.com/a/50065711/8802501
pub fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
  let d = b.mul_add(b, -4.0 * a * c);
  let q = -0.5 * (d.sqrt().copysign(b) + b);
  (d >= 0.0).then(|| (q / a, c / q))
}

pub fn ray_sphere_intersect(ro: Vec3, rd: Vec3, radius: f32) -> Option<(f32, f32)> {
  let a = rd.dot(rd);
  let b = rd.dot(ro) * 2.0;
  let c = radius.mul_add(-radius, ro.dot(ro));
  let (t0, t1) = solve_quadratic(a, b, c)?;
  Some(if t0 > t1 { (t1, t0) } else { (t0, t1) })
}
