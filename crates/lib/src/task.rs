// `tracing::Span` propagation to the spawned tasks, more info at:
// https://docs.rs/tracing/latest/tracing/struct.Span.html#method.or_current

use std::future::Future;

use tokio::task::JoinHandle;
use tracing::Instrument;

pub fn spawn<T>(future: T) -> JoinHandle<T::Output>
where
  T: Future + Send + 'static,
  T::Output: Send + 'static,
{
  let future = future.in_current_span();
  tokio::task::spawn(future)
}

pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
  F: FnOnce() -> R + Send + 'static,
  R: Send + 'static,
{
  let span = tracing::Span::current();
  tokio::task::spawn_blocking(move || {
    let _span = span.entered();
    f()
  })
}
