use std::path::Path;

use pyo3::{exceptions::*, prelude::*, types::*};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use util::task;

use super::download::*;

pub struct Downloader {
  /// STEP 1: get context with all available formats
  pub context: oneshot::Receiver<Context>,
  /// STEP 2: select 1 or 2 formats and send back their ids
  pub selected: oneshot::Sender<Vec<String>>,
  /// STEP 3: finish downloading
  pub finish: JoinHandle<PyResult<Info>>,
}

impl Downloader {
  /// STEP 0: init `Downloader` struct
  pub fn new(url: impl ToString, out_dir: impl AsRef<Path>) -> Self {
    let (context_in, context_out) = oneshot::channel();
    let (selected_in, selected_out) = oneshot::channel();

    let fs = |py: Python<'_>, ctx: Context| {
      tracing::trace!("format selector: sending available formats…");
      context_in.send(ctx).map_err(drop).unwrap();

      tracing::trace!("format selector: receiving selected formats…");
      match py.allow_threads(|| selected_out.blocking_recv()) {
        Ok(selected) => {
          tracing::trace!("format selector: done");
          Ok(PyList::new(py, selected).into())
        }
        Err(_) => {
          tracing::trace!("format selector: no formats were selected");
          Err(PyException::new_err("no formats were selected"))
        }
      }
    };

    let url = url.to_string();
    let out_dir = out_dir.as_ref().to_owned();
    let fs = FormatSelector { f: Some(Box::new(fs)) };

    let join = task::spawn_blocking(|| {
      Python::with_gil(|py| {
        let dl = py.import("lib.dl")?;
        let fs = PyCell::new(py, fs)?;

        tracing::trace!("downloading…");
        let res = dl.call_method1("download", (url, out_dir, fs));

        // make sure format selector gets dropped
        // to prevent channel-related deadlocks
        drop(fs.borrow_mut().f.take());

        tracing::trace!("downloading: done");
        res?.extract()
      })
    });

    Self {
      context: context_out,
      selected: selected_in,
      finish: join,
    }
  }
}

// ---

#[pyclass]
struct FormatSelector {
  #[allow(clippy::type_complexity)]
  f: Option<Box<dyn FnOnce(Python<'_>, Context) -> PyResult<Py<PyList>> + Send>>,
}

#[pymethods]
impl FormatSelector {
  fn __call__(&mut self, ctx: &PyAny) -> PyResult<Py<PyList>> {
    // FIXME: panics on playlists, should do something about it
    let f = self.f.take().expect("should be called only once");
    f(ctx.py(), ctx.extract()?)
  }
}
