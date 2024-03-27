use std::result;

pub use error::*;
pub use handle::*;

use crate::bindings as c;

mod error;
mod handle;

pub type Result<T> = result::Result<T, Error>;

pub fn version_string() -> String {
  let [major, minor, micro] = version();
  format!("{major}.{minor}.{micro}")
}

pub fn version() -> [c::guint; 3] {
  unsafe {
    let major = c::rsvg_major_version;
    let minor = c::rsvg_minor_version;
    let micro = c::rsvg_micro_version;
    [major, minor, micro]
  }
}
