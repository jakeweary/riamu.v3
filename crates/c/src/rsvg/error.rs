use std::ffi::CStr;
use std::fmt::{self, Debug, Display, Formatter};
use std::ptr::NonNull;

use crate::bindings as c;

pub(super) unsafe fn g_error(ptr: *mut c::GError) -> Error {
  let ptr = NonNull::new_unchecked(ptr);
  Error::GError(GError { ptr })
}

// ---

#[derive(Debug)]
pub enum Error {
  GError(GError),
  Other,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::GError(err) => Display::fmt(err, f),
      Self::Other => Display::fmt("something went wrong", f),
    }
  }
}

impl std::error::Error for Error {}

// ---

pub struct GError {
  ptr: NonNull<c::GError>,
}

impl Debug for GError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let err = unsafe { self.ptr.as_ref() };
    let msg = unsafe { CStr::from_ptr(err.message) };
    f.debug_struct("GError")
      .field("domain", &err.domain)
      .field("code", &err.code)
      .field("message", &msg)
      .finish()
  }
}

impl Display for GError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let err = unsafe { self.ptr.as_ref() };
    let msg = unsafe { CStr::from_ptr(err.message) };
    Display::fmt(&*msg.to_string_lossy(), f)
  }
}

impl Drop for GError {
  fn drop(&mut self) {
    let ptr = self.ptr.as_ptr();
    unsafe { c::g_error_free(ptr) }
  }
}

impl std::error::Error for GError {}

unsafe impl Send for GError {}
unsafe impl Sync for GError {}
