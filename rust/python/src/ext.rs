use std::any::Any;

use pyo3::{prelude::*, types::*};

pub trait DictExt<'a> {
  fn extract<T, K>(self, key: K) -> PyResult<T>
  where
    T: FromPyObject<'a>,
    K: ToPyObject;

  fn extract_optional<T, K>(self, key: K) -> PyResult<Option<T>>
  where
    T: FromPyObject<'a> + Any,
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
    T: FromPyObject<'a> + Any,
    K: ToPyObject,
  {
    self.get_item(key)?.map_or(Ok(None), |item| {
      Ok(match item.extract::<Option<T>>()? {
        Some(item) if !is_none(&item) => Some(item),
        _ => None,
      })
    })
  }
}

fn is_none(any: &dyn Any) -> bool {
  #![allow(clippy::map_clone)]
  let a = any.downcast_ref().map(|s: &String| &**s);
  let b = any.downcast_ref().map(|s: &&str| *s);
  a.or(b).is_some_and(|s| s == "none")
}
