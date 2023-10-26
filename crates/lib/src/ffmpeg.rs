use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::process::Command;

use serde::Deserialize;

pub fn meta(path: impl AsRef<OsStr>) -> io::Result<Meta> {
  let mut cmd = Command::new("ffprobe");

  #[rustfmt::skip]
  let cmd = cmd
    .arg("-of").arg("json=c=1")
    .arg("-show_format")
    .arg(&path);

  let out = cmd.output()?;
  let meta = serde_json::from_slice(&out.stdout)?;
  Ok(meta)
}

pub fn album_cover(path: impl AsRef<OsStr>) -> io::Result<Vec<u8>> {
  let mut cmd = Command::new("ffmpeg");

  #[rustfmt::skip]
  let cmd = cmd
    .arg("-i").arg(path)
    .arg("-c:v").arg("png")
    .arg("-f").arg("image2pipe")
    .arg("-");

  let out = cmd.output()?;
  Ok(out.stdout)
}

// ---

#[derive(Debug, Deserialize)]
pub struct Meta {
  #[serde(rename = "format")]
  pub tags: Tags,
}

#[derive(Debug, Deserialize)]
pub struct Tags {
  #[serde(rename = "tags")]
  pub inner: HashMap<String, String>,
}

impl Tags {
  pub fn get(&self, any_of: &[&str]) -> Option<&str> {
    any_of.iter().find_map(|&k| Some(&**self.inner.get(k)?))
  }

  pub fn get_or_empty(&self, any_of: &[&str]) -> &str {
    self.get(any_of).unwrap_or("")
  }
}
