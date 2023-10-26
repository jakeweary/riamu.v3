use pyo3::{prelude::*, types::*};

pub trait DictExt<'a> {
  fn extract<T, K>(self, key: K) -> PyResult<T>
  where
    T: FromPyObject<'a>,
    K: ToPyObject;

  fn extract_optional<T, K>(self, key: K) -> PyResult<Option<T>>
  where
    T: FromPyObject<'a> + AsRef<str>,
    K: ToPyObject;
}

impl<'a> DictExt<'a> for &'a PyDict {
  fn extract<T, K>(self, key: K) -> PyResult<T>
  where
    T: FromPyObject<'a>,
    K: ToPyObject,
  {
    let item = self.get_item(key)?.unwrap_or_else(|| {
      let py = self.py();
      py.None().into_ref(py)
    });
    item.extract()
  }

  fn extract_optional<T, K>(self, key: K) -> PyResult<Option<T>>
  where
    T: FromPyObject<'a> + AsRef<str>,
    K: ToPyObject,
  {
    self.get_item(key)?.map_or(Ok(None), |item| {
      Ok(match item.extract::<Option<T>>()? {
        Some(item) if item.as_ref() != "none" => Some(item),
        _ => None,
      })
    })
  }
}
