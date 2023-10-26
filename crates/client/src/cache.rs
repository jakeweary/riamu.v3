use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::ffi::OsString;
use std::hash::Hasher;
use std::path::{Path, PathBuf};
use std::{fs, io};

use filetime::FileTime;
use futures::StreamExt;
use inotify::{EventMask, Inotify, WatchMask};
use lib::fmt::num::Format;
use lru::LruCache;
use reqwest::IntoUrl;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use url::Url;

#[derive(Debug)]
pub enum Name<'a> {
  Keep,
  Set(&'a str),
}

#[derive(Debug)]
pub struct LruFileCache {
  bytes_limit: u64,
  base_url: Url,
  working_dir: PathBuf,
  state: Mutex<State>,
}

#[derive(Debug)]
struct State {
  bytes_stored: u64,
  files: LruCache<OsString, File>,
}

#[derive(Debug)]
struct File {
  size: u64,
}

impl LruFileCache {
  pub async fn new(base_url: impl IntoUrl, path: impl AsRef<Path>, bytes_limit: u64) -> io::Result<Self> {
    tracing::debug!("initializing cache…");
    let cache = Self {
      bytes_limit,
      base_url: base_url.into_url().unwrap(),
      working_dir: path.as_ref().to_path_buf(),
      state: State::new(path).await?.into(),
    };

    tracing::debug!("initializing cache: done");
    cache.log_stats().await;

    Ok(cache)
  }

  pub async fn watch(&self) -> io::Result<()> {
    let mut buffer = [0; 1 << 10];
    let mut inotify = Inotify::init()?.into_event_stream(&mut buffer)?;

    let mask = WatchMask::OPEN | WatchMask::MOVE_SELF | WatchMask::DELETE_SELF;
    inotify.watches().add(&self.working_dir, mask)?;

    tracing::debug!("watching filesystem events…");
    while let Some(event) = inotify.next().await {
      let event = event?;

      if event.mask.intersects(EventMask::MOVE_SELF | EventMask::DELETE_SELF) {
        tracing::warn!("cache directory was moved or deleted");
        break;
      }

      if let (EventMask::OPEN, Some(name)) = (event.mask, event.name) {
        tracing::trace!(?name, "file open event");

        let path = self.working_dir.join(&name);
        let atime = FileTime::now();
        filetime::set_file_atime(&path, atime)?;

        let mut state = self.state.lock().await;
        state.files.promote(&name);
      }
    }

    Ok(())
  }

  pub async fn store_file(&self, path: &Path, name: Name<'_>) -> io::Result<Option<Url>> {
    let meta = tokio::fs::metadata(path).await?;
    let size = meta.len() as u64;
    let name = match name {
      Name::Keep => path.file_name().unwrap().to_string_lossy(),
      Name::Set(name) => Cow::Borrowed(name),
    };
    tracing::debug!(?name, "storing a {}B file…", size.iec());

    let mut hasher = DefaultHasher::new();
    let mut buffer = [0; 1 << 16];
    let mut file = tokio::fs::File::open(path).await?;
    let hash = loop {
      match file.read(&mut buffer[..]).await? {
        0 => break hasher.finish(),
        n => hasher.write(&buffer[..n]),
      }
    };

    let hash = format!("{:016x}", hash);
    let url = self.build_url(&hash, &name).unwrap();
    tracing::debug!(%url);

    let hashed_name = {
      let ext = Path::new(&*name).extension().unwrap();
      let name = Path::new(&hash).with_extension(ext);
      name.into_os_string()
    };

    if self.state.lock().await.files.contains(&hashed_name) {
      tracing::debug!("already stored");
      Ok(Some(url))
    } else if !self.reserve(size).await? {
      tracing::debug!("too big");
      Ok(None)
    } else {
      std::fs::copy(path, self.working_dir.join(&hashed_name))?;
      self.state.lock().await.push(hashed_name, size);
      self.log_stats().await;
      Ok(Some(url))
    }
  }

  async fn reserve(&self, bytes: u64) -> io::Result<bool> {
    if bytes > self.bytes_limit {
      Ok(false)
    } else {
      let mut state = self.state.lock().await;
      let overshoot = (state.bytes_stored + bytes).saturating_sub(self.bytes_limit);
      tracing::debug!("reserving {}B (overshoot: {}B)", bytes.iec(), overshoot.iec());

      while state.bytes_stored + bytes > self.bytes_limit {
        let (name, file) = state.pop().unwrap();
        tracing::debug!(?name, "removing a {}B file…", file.size.iec());
        tokio::fs::remove_file(&self.working_dir.join(&name)).await?;
      }

      Ok(true)
    }
  }

  async fn log_stats(&self) {
    let state = self.state.lock().await;
    tracing::debug!(
      "cache: {} files ({}B/{}B, {:.0}%)",
      state.files.len(),
      state.bytes_stored.iec(),
      self.bytes_limit.iec(),
      state.bytes_stored as f64 / self.bytes_limit as f64 * 100.0,
    );
  }

  fn build_url(&self, hash: &str, name: &str) -> Result<Url, ()> {
    let mut url = self.base_url.clone();
    url.path_segments_mut()?.extend(&[hash, name]);
    Ok(url)
  }
}

impl State {
  async fn new(path: impl AsRef<Path>) -> io::Result<Self> {
    let path = path.as_ref().to_path_buf();
    let span = tracing::Span::current();
    let new = tokio::task::spawn_blocking(move || {
      let _span = span.entered();
      State::new_blocking(path)
    });
    new.await.unwrap()
  }

  fn new_blocking(path: PathBuf) -> io::Result<Self> {
    tracing::trace!("mkdir -p {:?}", path);
    fs::create_dir_all(&path)?;

    let mut bytes_stored = 0;
    let mut file_tuples = Vec::new();
    for entry in fs::read_dir(&path)? {
      let entry = entry?;
      if entry.file_type()?.is_file() {
        let name = entry.file_name();
        let meta = entry.metadata()?;
        let size = meta.len();
        let atime = meta.accessed()?;
        bytes_stored += size;
        file_tuples.push((atime, name, size));
      }
    }

    file_tuples.sort_unstable_by(|(a, ..), (b, ..)| a.cmp(b));

    let mut files = LruCache::unbounded();
    for (_, name, size) in file_tuples {
      files.push(name, File { size });
    }

    Ok(Self { bytes_stored, files })
  }

  fn push(&mut self, name: OsString, size: u64) {
    self.files.push(name, File { size });
    self.bytes_stored += size;
  }

  fn pop(&mut self) -> Option<(OsString, File)> {
    let (name, file) = self.files.pop_lru()?;
    self.bytes_stored -= file.size;
    Some((name, file))
  }
}
