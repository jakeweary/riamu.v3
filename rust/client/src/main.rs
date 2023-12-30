#![feature(array_windows)]
#![feature(array_chunks)]

// https://stackoverflow.com/a/57049687/8802501
// extern crate self as riamu;

use tokio::runtime::Builder as Runtime;

mod cache;
mod client;
mod commands;
mod db;
mod tracing;

fn main() -> client::Result<()> {
  tracing::init()?;

  tracing::debug!("initializing python…");
  python::init()?;

  tracing::debug!("initializing async runtime…");
  let rt = Runtime::new_current_thread().enable_all().build()?;

  tracing::debug!("starting client…");
  let res = rt.block_on(client::Client::start());

  // https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#shutdown
  rt.shutdown_background();

  res
}
