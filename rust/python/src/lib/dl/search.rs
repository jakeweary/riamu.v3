use pyo3::prelude::*;

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
pub struct Result {
  pub url: String,
  pub title: String,
  pub channel: Option<String>,
}

pub fn search(query: &str) -> PyResult<Vec<Result>> {
  Python::with_gil(|py| {
    let dl = py.import("lib.dl")?;
    let info = dl.call_method1("ytsearch", (query,))?;
    info.get_item("entries")?.extract()
  })
}
