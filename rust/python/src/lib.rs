use pyo3::prelude::*;

mod ext;
mod logs;

#[path = "lib/lib.rs"]
pub mod lib;

pub fn init() -> PyResult<()> {
  pyo3::prepare_freethreaded_python();
  logs::redirect()
}
