use std::path::Path;
use std::{env, iter};

use itertools::Itertools;
use pyo3::{self, prelude::*, types::*};

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
    let receive = PyCFunction::new_closure(py, None, None, |args, _kwargs| {
      let (record,) = args.extract::<(Record<'_>,)>()?;
      record.dispatch();
      PyResult::Ok(())
    })?;
    let globals = [("send", receive)].into_py_dict(py);
    py.run(REDIRECT, Some(globals), None)
  })
}

// ---

#[derive(FromPyObject)]
struct Record<'a> {
  #[pyo3(attribute("getMessage"), from_py_with = "message")]
  message: &'a str,
  name: &'a str,
  levelno: u32,
  lineno: Option<u32>,
  pathname: Option<&'a str>,
}

impl<'a> Record<'a> {
  fn dispatch(self) {
    let ignore_reason = match self.message {
      msg if msg.contains('\n') => Some("multi-line"),
      msg if msg.chars().nth(1000).is_some() => Some("too long"),
      _ => None,
    };

    if let Some(reason) = ignore_reason {
      let target = self.name;
      let level = self.level();
      return tracing::trace!(?level, %target, "ignored log entry ({})", reason);
    }

    // wanted to use `tracing` directly but there seems to be a blocker:
    // https://github.com/tokio-rs/tracing/pull/2048
    log::logger().log(
      &log::Record::builder()
        .args(format_args!("{}", self.message))
        .level(self.level())
        .target(&self.target())
        .file(self.file().or(self.pathname))
        .line(self.lineno)
        .build(),
    );
  }

  fn level(&self) -> log::Level {
    match self.levelno / 10 {
      0 => log::Level::Trace,
      1 => log::Level::Debug,
      2 => log::Level::Info,
      3 => log::Level::Warn,
      _ => log::Level::Error,
    }
  }

  fn target(&self) -> String {
    let segments = self.name.split('.');
    iter::once("python").chain(segments).join("::")
  }

  fn file(&self) -> Option<&str> {
    let path = Path::new(self.pathname?);
    let cwd = env::current_dir().ok()?;
    let stripped = path.strip_prefix(cwd).ok()?;
    stripped.to_str()
  }
}

#[pyfunction]
fn message(item: &PyAny) -> PyResult<&str> {
  item.call0()?.extract()
}
