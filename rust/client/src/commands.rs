use crate::client;

mod deezer;
mod download;
mod imgur;
mod tiktok;
mod weather;
mod lookup {
  pub mod imdb;
  pub mod urban_dictionary;
  pub mod wikipedia;
}
mod meta {
  pub mod info;
  pub mod speed;
  pub mod speed_to_discord;
}
mod repost {
  pub mod _2ch;
  pub mod _4chan;
}
mod user {
  pub mod status {
    pub mod history;
  }
}

pub fn build() -> client::Commands {
  client::commands! {
    // downloading, uploading, etc.
    "download" => download::run,
    "tiktok" => tiktok::run,
    "deezer" => {
      "as-file" => deezer::as_file,
      "as-direct-link" => deezer::as_direct_link,
    },
    "imgur" => {
      "upload" => {
        "file" => imgur::file,
        "url" => imgur::url,
      },
    },

    // other stuff
    "weather" => weather::run,
    "lookup" => {
      "imdb" => lookup::imdb::run,
      "wikipedia" => {
        "en" => lookup::wikipedia::en,
        "ru" => lookup::wikipedia::ru,
      },
      "urban" => {
        "dictionary" => lookup::urban_dictionary::run,
      },
    },
    "meta" => {
      "info" => meta::info::run,
      "speed" => meta::speed::run,
      "speed-to-discord" => meta::speed_to_discord::run,
    },
    "repost" => {
      "2ch" => repost::_2ch::run,
      "4chan" => repost::_4chan::run,
    },
    "user" => {
      "status" => {
        "history" => user::status::history::run,
      },
    },
  }
}
