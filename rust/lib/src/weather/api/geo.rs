use serde::Deserialize;

pub type Root = Vec<Location>;

#[derive(Debug, Deserialize)]
pub struct Location {
  pub name: String,
  pub local_names: LocalNames,
  pub country: String,
  pub state: Option<String>,
  pub lat: f64,
  pub lon: f64,
}

#[derive(Debug, Deserialize)]
pub struct LocalNames {
  pub feature_name: Option<String>,
  pub ascii: Option<String>,
  pub en: Option<String>,
  // #[serde(flatten)]
  // pub names: HashMap<String, String>,
}
