use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::process::Command;

use serde::Deserialize;

pub fn meta(path: impl AsRef<OsStr>) -> io::Result<Meta> {
  let mut cmd = Command::new("ffprobe");
  cmd.arg("-of").arg("json=c=1");
  cmd.arg("-show_format");
  cmd.arg(&path);

  let out = cmd.output()?;
  let meta = serde_json::from_slice(&out.stdout)?;
  Ok(meta)
}

pub fn album_cover(path: impl AsRef<OsStr>, codec: &str) -> io::Result<Vec<u8>> {
  let mut cmd = Command::new("ffmpeg");
  cmd.arg("-i").arg(path);
  cmd.arg("-c:v").arg(codec);
  cmd.arg("-f").arg("image2pipe");
  cmd.arg("-");

  let out = cmd.output()?;
  Ok(out.stdout)
}

// ---

#[derive(Debug, Deserialize)]
pub struct Meta {
  pub format: Format,
}

#[derive(Debug, Deserialize)]
pub struct Format {
  pub tags: HashMap<String, String>,
}

impl Format {
  pub fn tag(&self, any_of: &[&str]) -> Option<&str> {
    any_of.iter().find_map(|&k| Some(&**self.tags.get(k)?))
  }

  pub fn tag_or_empty(&self, any_of: &[&str]) -> &str {
    self.tag(any_of).unwrap_or("")
  }
}
