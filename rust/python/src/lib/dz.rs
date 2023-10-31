use std::path::Path;

use pyo3::prelude::*;

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
pub struct Track {
  pub id: u64,
  pub link: String,
  pub title: String,
  pub artist: Artist,
  pub album: Album,
  pub duration: u64,
}

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
pub struct Artist {
  pub id: u64,
  pub name: String,
}

#[derive(FromPyObject)]
#[pyo3(from_item_all)]
pub struct Album {
  pub id: u64,
  pub title: String,
}

pub fn search(query: &str) -> PyResult<Vec<Track>> {
  Python::with_gil(|py| {
    let dz = py.import("lib.dz")?;
    let tracks = dz.call_method1("search", (query,))?;
    tracks.get_item("data")?.extract()
  })
}

pub fn download(url: &str, bitrate: &str, out_dir: &Path) -> PyResult<Track> {
  Python::with_gil(|py| {
    let dz = py.import("lib.dz")?;
    let dl_obj = dz.call_method1("generate_download_object", (url, bitrate))?;
    dz.call_method1("download", (dl_obj, out_dir))?;
    dl_obj.getattr("single")?.get_item("trackAPI")?.extract()
  })
}
