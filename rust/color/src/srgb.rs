pub use self::fast::f32_to_srgb8::*;
pub use self::fast::srgb8_to_f32::*;
pub use self::srgb::*;

#[allow(clippy::module_inception)]
mod srgb;

mod fast {
  pub mod f32_to_srgb8;
  pub mod srgb8_to_f32;
}

pub mod transfer_fns;
