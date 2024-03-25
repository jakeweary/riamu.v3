use std::borrow::Cow;
use std::ffi::OsString;
use std::hash::{DefaultHasher, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};

use filetime::FileTime;
use fmt::num::Format as _;
use inotify::{EventMask, Inotify, WatchMask};
use lru::LruCache;
use parking_lot::Mutex;
use url::Url;
use util::task;

#[derive(Debug)]
pub enum Name {
  Keep,
  Set(String),
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
  pub async fn new(base_url: Url, working_dir: PathBuf, bytes_limit: u64) -> io::Result<Self> {
    task::spawn_blocking(move || Self::new_blocking(base_url, working_dir, bytes_limit)).await?
  }

  pub async fn watch(self: &Arc<Self>) -> io::Result<()> {
    let cache = self.to_owned();
    task::spawn_blocking(move || cache.watch_blocking()).await?
  }

  pub async fn store_file(self: &Arc<Self>, path: PathBuf, name: Name) -> io::Result<Option<Url>> {
    let cache = self.to_owned();
    task::spawn_blocking(move || cache.store_file_blocking(&path, name)).await?
  }
}

impl LruFileCache {
  pub fn new_blocking(base_url: Url, working_dir: PathBuf, bytes_limit: u64) -> io::Result<Self> {
    let cache = Self {
      state: State::new(&working_dir)?.into(),
      bytes_limit,
      base_url,
      working_dir,
    };

    cache.log_stats();

    Ok(cache)
  }

  pub fn watch_blocking(&self) -> io::Result<()> {
    let mut inotify = Inotify::init()?;
    let mut buffer = [0; 1 << 10];

    let mask = WatchMask::OPEN | WatchMask::MOVE_SELF | WatchMask::DELETE_SELF;
    inotify.watches().add(&self.working_dir, mask)?;

    tracing::debug!("watching filesystem events…");
    loop {
      let events = inotify.read_events_blocking(&mut buffer)?;

      for event in events {
        if event.mask.intersects(EventMask::MOVE_SELF | EventMask::DELETE_SELF) {
          tracing::warn!("cache directory was moved or deleted");
          break;
        }

        if let (EventMask::OPEN, Some(name)) = (event.mask, event.name) {
          tracing::trace!(?name, "file open event");

          // NOTE: apparently similar api exists in std
          // but for some reason they put it under `std::fs::File::set_times`
          // instead of `std::fs::set_times` which makes it unusable
          // for this use case (`File::open` triggers an fs event and
          // we're literally in the loop that listens to these events)
          let path = self.working_dir.join(name);
          let atime = FileTime::now();
          filetime::set_file_atime(&path, atime)?;

          let mut state = self.state.lock();
          state.files.promote(name);
        }
      }
    }
  }

  pub fn store_file_blocking(&self, path: &Path, name: Name) -> io::Result<Option<Url>> {
    let meta = fs::metadata(path)?;
    let size = meta.len() as u64;
    let name = match name {
      Name::Keep => path.file_name().unwrap().to_string_lossy(),
      Name::Set(name) => Cow::Owned(name),
    };
    tracing::debug!(?name, "storing a {}B file…", size.iec());

    let hash = hash_file(path)?;
    let hash = format!("{:016x}", hash);
    let url = self.build_url(&hash, &name).unwrap();
    tracing::debug!(%url);

    let hashed_name = {
      let ext = Path::new(&*name).extension().unwrap();
      let name = Path::new(&hash).with_extension(ext);
      name.into_os_string()
    };

    if self.state.lock().files.contains(&hashed_name) {
      tracing::debug!("already stored");
      Ok(Some(url))
    } else if !self.reserve(size)? {
      tracing::debug!("too big");
      Ok(None)
    } else {
      std::fs::copy(path, self.working_dir.join(&hashed_name))?;
      self.state.lock().push(hashed_name, size);
      self.log_stats();
      Ok(Some(url))
    }
  }
}

impl LruFileCache {
  fn reserve(&self, bytes: u64) -> io::Result<bool> {
    let fits = bytes <= self.bytes_limit;

    if fits {
      let mut state = self.state.lock();

      let overshoot = (state.bytes_stored + bytes).saturating_sub(self.bytes_limit);
      tracing::debug!("reserving {}B (overshoot: {}B)", bytes.iec(), overshoot.iec());

      while state.bytes_stored + bytes > self.bytes_limit {
        let (name, file) = state.pop().unwrap();
        tracing::debug!(?name, "removing a {}B file…", file.size.iec());
        fs::remove_file(&self.working_dir.join(&name))?;
      }
    }

    Ok(fits)
  }

  fn log_stats(&self) {
    let state = self.state.lock();
    let files = state.files.len();
    let stored = state.bytes_stored.iec();
    let limit = self.bytes_limit.iec();
    let ratio = state.bytes_stored as f64 / self.bytes_limit as f64 * 100.0;
    tracing::debug!("cache: {} files ({}B/{}B, {:.0}%)", files, stored, limit, ratio);
  }

  fn build_url(&self, hash: &str, name: &str) -> Result<Url, ()> {
    let mut url = self.base_url.clone();
    url.path_segments_mut()?.extend(&[hash, name]);
    Ok(url)
  }
}

impl State {
  fn new(path: &Path) -> io::Result<Self> {
    tracing::debug!("initializing cache state…");

    tracing::trace!("mkdir -p {:?}", path);
    fs::create_dir_all(path)?;

    let mut bytes_stored = 0;
    let mut file_tuples = Vec::new();
    for entry in fs::read_dir(path)? {
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

    tracing::debug!("initializing cache state: done");

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

fn hash_file(path: &Path) -> io::Result<u64> {
  let mut hasher = DefaultHasher::new();
  let mut buffer = [0; 1 << 12];
  let mut file = fs::File::open(path)?;

  loop {
    match file.read(&mut buffer[..])? {
      0 => break Ok(hasher.finish()),
      n => hasher.write(&buffer[..n]),
    }
  }
}
