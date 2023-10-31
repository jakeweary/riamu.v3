use pyo3::prelude::*;

pub mod dl;
pub mod dz;

pub fn versions() -> PyResult<Vec<(String, String)>> {
  Python::with_gil(|py| {
    let lib = py.import("lib")?;
    let versions = lib.call_method0("versions")?;
    versions.extract()
  })
}
