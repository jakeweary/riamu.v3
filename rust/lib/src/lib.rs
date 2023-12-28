pub mod discord;
pub mod ffmpeg;
pub mod fmt;
pub mod gcra;
pub mod html;
pub mod network;
pub mod random;
pub mod task;
pub mod cairo {
  pub mod blur;
  pub mod ext;
  pub mod util;
}
pub mod color {
  pub mod convert;
  pub mod oklab;
  pub mod srgb;
}
pub mod hash {
  pub mod splitmix64;
}
pub mod text {
  pub mod style;
}
pub mod weather {
  pub mod api;
  pub mod render;
}
