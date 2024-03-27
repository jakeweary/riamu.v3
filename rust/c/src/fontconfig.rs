use std::ffi::{c_int, CString};
use std::fmt::{self, Display, Formatter};
use std::os::unix::prelude::*;
use std::path::Path;
use std::{error, ptr, result};

use crate::bindings as c;

pub type Result<T> = result::Result<T, Error>;

// ---

pub fn version_string() -> String {
  let [major, minor, patch] = version();
  format!("{major}.{minor}.{patch}")
}

pub fn version() -> [c_int; 3] {
  let n = unsafe { c::FcGetVersion() };
  [n / 10000, n / 100 % 100, n % 100]
}

pub fn add_file(path: impl AsRef<Path>) -> Result<()> {
  let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
  let (config, path) = (ptr::null_mut(), path.as_ptr().cast());
  match unsafe { c::FcConfigAppFontAddFile(config, path) } {
    1 => Ok(()),
    _ => Err(Error),
  }
}

pub fn add_dir(path: impl AsRef<Path>) -> Result<()> {
  let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
  let (config, path) = (ptr::null_mut(), path.as_ptr().cast());
  match unsafe { c::FcConfigAppFontAddDir(config, path) } {
    1 => Ok(()),
    _ => Err(Error),
  }
}

// ---

#[derive(Debug)]
pub struct Error;

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    "failed to add fonts".fmt(f)
  }
}

impl error::Error for Error {}
