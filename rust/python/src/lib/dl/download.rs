use pyo3::{prelude::*, types::*};

use crate::ext::DictExt;

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
pub struct Result {
  pub id: String,
  pub title: String,
  pub webpage_url: String,
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
    })
  }
}

impl Format {
  pub fn size(&self) -> Option<i64> {
    self.filesize.or(self.filesize_approx)
  }

  pub fn is_audio(&self) -> bool {
    self.audio_ext.is_some() && self.video_ext.is_none()
  }

  pub fn is_video(&self) -> bool {
    self.audio_ext.is_none() && self.video_ext.is_some()
  }
}
