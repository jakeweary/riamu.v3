use std::ffi::CString;
use std::mem::MaybeUninit;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::ptr::{self, NonNull};

use crate::bindings as c;

use super::error::{g_error, Error};
use super::Result;

#[derive(Debug, Copy, Clone)]
pub struct IntrinsicDimensions {
  pub width: Option<c::RsvgLength>,
  pub height: Option<c::RsvgLength>,
  pub viewbox: Option<c::RsvgRectangle>,
}

pub struct Handle {
  ptr: NonNull<c::RsvgHandle>,
}

impl Handle {
  pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
    let mut err = ptr::null_mut();
    let path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let rsvg = unsafe { c::rsvg_handle_new_from_file(path.as_ptr(), &mut err) };
    Self::from_ptr(rsvg).ok_or_else(|| unsafe { g_error(err) })
  }

  pub fn from_data(data: &[u8]) -> Result<Self> {
    let mut err = ptr::null_mut();
    let (data, data_len) = (data.as_ptr(), data.len() as u64);
    let rsvg = unsafe { c::rsvg_handle_new_from_data(data, data_len, &mut err) };
    Self::from_ptr(rsvg).ok_or_else(|| unsafe { g_error(err) })
  }

  pub fn set_stylesheet(&self, css: &str) -> Result<()> {
    let mut err = ptr::null_mut();
    let (rsvg, css, css_len) = (self.ptr.as_ptr(), css.as_ptr(), css.len() as u64);
    let ok = unsafe { c::rsvg_handle_set_stylesheet(rsvg, css, css_len, &mut err) } != 0;
    ok.then_some(()).ok_or_else(|| unsafe { g_error(err) })
  }

  pub fn render_cairo(&self, ctx: &cairo::Context) -> Result<()> {
    let (rsvg, ctx) = (self.ptr.as_ptr(), ctx.to_raw_none().cast());
    let ok = unsafe { c::rsvg_handle_render_cairo(rsvg, ctx) } != 0;
    ok.then_some(()).ok_or(Error::Other)
  }

  pub fn intrinsic_dimensions(&self) -> IntrinsicDimensions {
    let (mut has_width, mut width) = (MaybeUninit::uninit(), MaybeUninit::uninit());
    let (mut has_height, mut height) = (MaybeUninit::uninit(), MaybeUninit::uninit());
    let (mut has_viewbox, mut viewbox) = (MaybeUninit::uninit(), MaybeUninit::uninit());
    unsafe {
      #[rustfmt::skip]
      c::rsvg_handle_get_intrinsic_dimensions(
        self.ptr.as_ptr(),
        has_width.as_mut_ptr(), width.as_mut_ptr(),
        has_height.as_mut_ptr(), height.as_mut_ptr(),
        has_viewbox.as_mut_ptr(), viewbox.as_mut_ptr(),
      );
      let width = (has_width.assume_init() != 0).then(|| width.assume_init());
      let height = (has_height.assume_init() != 0).then(|| height.assume_init());
      let viewbox = (has_viewbox.assume_init() != 0).then(|| viewbox.assume_init());
      IntrinsicDimensions { width, height, viewbox }
    }
  }
}

impl Handle {
  fn from_ptr(ptr: *mut c::RsvgHandle) -> Option<Self> {
    NonNull::new(ptr).map(|ptr| Self { ptr })
  }
}

impl Drop for Handle {
  fn drop(&mut self) {
    let rsvg = self.ptr.as_ptr();
    unsafe {
      c::rsvg_handle_close(rsvg, ptr::null_mut());
      c::g_object_unref(rsvg as c::gpointer);
    }
  }
}
