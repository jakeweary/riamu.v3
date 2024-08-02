use std::path::Path;
use std::{env, iter};

use itertools::Itertools;
use log::Level;
use pyo3::pybacked::PyBackedStr;
use pyo3::{prelude::*, types::*};

const REDIRECT: &str = r#"
import logging

class Handler(logging.Handler):
  def emit(self, record: logging.LogRecord):
    send(record)

logging.basicConfig(level=logging.NOTSET, handlers=[Handler()])
logging.debug('redirected logs: python -> rust')
"#;

pub fn redirect() -> PyResult<()> {
  Python::with_gil(|py| {
    let receive = PyCFunction::new_closure_bound(py, None, None, |args, _kwargs| {
      let (record,) = args.extract::<(Record,)>()?;
      record.dispatch();
      PyResult::Ok(())
    })?;
    let globals = [("send", receive)].into_py_dict_bound(py);
    py.run_bound(REDIRECT, Some(&globals), None)
  })
}

// ---

#[derive(FromPyObject)]
struct Record {
  #[pyo3(attribute("getMessage"), from_py_with = "message")]
  message: PyBackedStr,
  name: PyBackedStr,
  levelno: u32,
  lineno: Option<u32>,
  pathname: Option<PyBackedStr>,
}

impl Record {
  fn dispatch(self) {
    let ignore_reason = match &self.message {
      msg if msg.contains('\n') => Some("multi-line"),
      msg if msg.chars().nth(1000).is_some() => Some("too long"),
      _ => None,
    };

    if let Some(reason) = ignore_reason {
      let target = &self.name;
      let level = self.level();
      return tracing::trace!(?level, %target, "ignored log entry ({})", reason);
    }

    // wanted to use `tracing` directly but there seems to be a blocker:
    // https://github.com/tokio-rs/tracing/pull/2048
    log::logger().log(&{
      log::Record::builder()
        .args(format_args!("{}", self.message))
        .level(self.level())
        .target(&self.target())
        .file(self.file().or(self.pathname.as_deref()))
        .line(self.lineno)
        .build()
    });
  }

  fn level(&self) -> Level {
    match self.levelno / 10 {
      0 => Level::Trace,
      1 => Level::Debug,
      2 => Level::Info,
      3 => Level::Warn,
      _ => Level::Error,
    }
  }

  fn target(&self) -> String {
    let segments = self.name.split('.');
    iter::once("python").chain(segments).join("::")
  }

  fn file(&self) -> Option<&str> {
    let path = Path::new(self.pathname.as_deref()?);
    let cwd = env::current_dir().ok()?;
    let stripped = path.strip_prefix(cwd).ok()?;
    stripped.to_str()
  }
}

#[pyfunction]
fn message(item: &Bound<'_, PyAny>) -> PyResult<PyBackedStr> {
  item.call0()?.extract()
}
