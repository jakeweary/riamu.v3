pub use tracing::*;

use tracing_subscriber::prelude::*;
use tracing_subscriber::util::TryInitError;
use tracing_subscriber::EnvFilter;

pub fn init() -> Result<(), TryInitError> {
  // let console = console_subscriber::ConsoleLayer::builder().spawn();

  let stderr = tracing_subscriber::fmt::layer()
    .without_time()
    .with_writer(std::io::stderr)
    .with_filter(EnvFilter::from_default_env());

  tracing_subscriber::registry()
    // .with(console)
    .with(stderr)
    .try_init()
}
