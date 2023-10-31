use cairo::Context;

pub trait ContextExt {
  fn set_source_rgb_u8(&self, r: u8, g: u8, b: u8);
  fn set_source_rgba_u8(&self, r: u8, g: u8, b: u8, a: u8);
  fn set_source_rgb_u32(&self, rgb: u32);
  fn set_source_rgba_u32(&self, argb: u32);
}

impl ContextExt for Context {
  fn set_source_rgb_u8(&self, r: u8, g: u8, b: u8) {
    self.set_source_rgb(float(r), float(g), float(b));
  }

  fn set_source_rgba_u8(&self, r: u8, g: u8, b: u8, a: u8) {
    self.set_source_rgba(float(r), float(g), float(b), float(a));
  }

  fn set_source_rgb_u32(&self, rgb: u32) {
    let [b, g, r, _] = rgb.to_le_bytes();
    self.set_source_rgb_u8(r, g, b);
  }

  fn set_source_rgba_u32(&self, argb: u32) {
    let [b, g, r, a] = argb.to_le_bytes();
    self.set_source_rgba_u8(r, g, b, a);
  }
}

fn float(byte: u8) -> f64 {
  byte as f64 / u8::MAX as f64
}
