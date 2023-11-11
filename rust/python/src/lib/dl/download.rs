use pyo3::{prelude::*, types::*};

use crate::ext::DictExt;

pub struct Info {
  pub id: String,
  pub title: String,
  pub webpage_url: String,
  pub thumbnail: Option<String>,
}

impl<'a> FromPyObject<'a> for Info {
  fn extract(any: &'a PyAny) -> PyResult<Self> {
    let dict: &PyDict = any.extract()?;
    Ok(Self {
      id: dict.extract("id")?,
      title: dict.extract("title")?,
      webpage_url: dict.extract("webpage_url")?,
      thumbnail: dict.extract_optional("thumbnail")?,
    })
  }
}

// ---

pub struct Context {
  pub duration: Option<f64>,
  pub formats: Vec<Format>,
  pub has_merged_format: bool,
  pub incomplete_formats: bool,
}

impl<'a> FromPyObject<'a> for Context {
  fn extract(any: &'a PyAny) -> PyResult<Self> {
    let dict: &PyDict = any.extract()?;
    Ok(Self {
      duration: dict.extract_optional("duration")?,
      formats: dict.extract("formats")?,
      has_merged_format: dict.extract("has_merged_format")?,
      incomplete_formats: dict.extract("incomplete_formats")?,
    })
  }
}

impl Context {
  pub fn resolve(&self, format_ids: &'_ [impl AsRef<str>]) -> Option<Vec<&Format>> {
    format_ids
      .iter()
      .map(|id| self.formats.iter().find(|f| f.format_id == id.as_ref()))
      .collect()
  }
}

// ---

pub struct Format {
  pub format: String,
  pub format_id: String,
  pub filesize: Option<i64>,
  pub filesize_approx: Option<i64>,
  pub ext: String,
  pub audio_ext: Option<String>,
  pub video_ext: Option<String>,
  pub acodec: Option<String>,
  pub vcodec: Option<String>,
  pub abr: Option<f64>,
  pub vbr: Option<f64>,
  pub tbr: Option<f64>,
  pub width: Option<u64>,
  pub height: Option<u64>,
}

impl<'a> FromPyObject<'a> for Format {
  fn extract(any: &'a PyAny) -> PyResult<Self> {
    let dict: &PyDict = any.extract()?;
    Ok(Self {
      format: dict.extract("format")?,
      format_id: dict.extract("format_id")?,
      filesize: dict.extract("filesize")?,
      filesize_approx: dict.extract("filesize_approx")?,
      ext: dict.extract("ext")?,
      audio_ext: dict.extract_optional("audio_ext")?,
      video_ext: dict.extract_optional("video_ext")?,
      acodec: dict.extract_optional("acodec")?,
      vcodec: dict.extract_optional("vcodec")?,
      abr: dict.extract("abr")?,
      vbr: dict.extract("vbr")?,
      tbr: dict.extract("tbr")?,
      width: dict.extract_optional("width")?,
      height: dict.extract_optional("height")?,
    })
  }
}

impl Format {
  pub fn is_video(&self) -> bool {
    self.video_ext.is_some() && self.audio_ext.is_none()
  }

  pub fn is_audio(&self) -> bool {
    self.audio_ext.is_some() && self.video_ext.is_none()
  }
}
