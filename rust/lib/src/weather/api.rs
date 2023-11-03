use url::Url;

pub use self::geo::Root as Geo;
pub use self::onecall::Root as Onecall;

pub mod geo;
pub mod onecall;

pub struct Api {
  key: String,
}

impl Api {
  pub fn new(key: impl ToString) -> Self {
    Self { key: key.to_string() }
  }

  pub async fn geo(&self, query: &str) -> reqwest::Result<Geo> {
    let appid = ("appid", &*self.key);
    let q = ("q", query);

    let url = "http://api.openweathermap.org/geo/1.0/direct";
    let url = Url::parse_with_params(url, &[appid, q]).unwrap();

    let resp = reqwest::get(url).await?.error_for_status()?;
    let json = resp.json().await?;
    Ok(json)
  }

  pub async fn onecall(&self, lat: f64, lon: f64) -> reqwest::Result<Onecall> {
    let appid = ("appid", &*self.key);
    let lat = ("lat", &*lat.to_string());
    let lon = ("lon", &*lon.to_string());
    let units = ("units", "metric");

    let url = "https://api.openweathermap.org/data/2.5/onecall";
    let url = Url::parse_with_params(url, &[appid, lat, lon, units]).unwrap();

    let resp = reqwest::get(url).await?.error_for_status()?;
    let json = resp.json().await?;
    Ok(json)
  }
}
