use std::io;

pub use tracing::*;

use tracing_subscriber::util::TryInitError;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{fmt, prelude::*};

pub fn init() -> Result<(), TryInitError> {
  let stderr = fmt::layer()
    .without_time()
    .with_writer(io::stderr)
    .with_filter(EnvFilter::from_default_env());

  tracing_subscriber::registry()
    // .with(console_subscriber::ConsoleLayer::builder().spawn())
    .with(stderr)
    .try_init()
}
