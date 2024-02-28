use crate::client;

mod _2ch;
mod _4chan;
mod deezer;
mod download;
mod imgur;
mod random;
mod tiktok;
mod weather;
mod lookup {
  pub mod imdb;
  pub mod omdb;
  pub mod urban_dictionary;
  pub mod wikipedia;
}
mod meta {
  pub mod info;
  pub mod shell;
  pub mod speed;
  pub mod speed_to_discord;
}
mod text {
  pub mod style;
}
mod user {
  pub mod profile;
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
      "omdb" => lookup::omdb::run,
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
      "shell" => meta::shell::run,
      "speed" => meta::speed::run,
      "speed-to-discord" => meta::speed_to_discord::run,
    },
    "random" => {
      "int" => random::int,
      "real" => random::real,
      "coin" => random::coin,
      "color" => random::color,
      "2ch" => {
        "post" => _2ch::random_post,
        "post-with-attachments" => _2ch::random_post_with_attachments,
      },
      "4chan" => {
        "post" => _4chan::random_post,
        "post-with-attachment" => _4chan::random_post_with_attachment,
      },
    },
    "repost" => {
      "2ch" => _2ch::repost,
      "4chan" => _4chan::repost,
    },
    "text" => {
      "style" => text::style::run,
    },
    "user" => {
      "avatar" => user::profile::avatar,
      "banner" => user::profile::banner,
      "status" => {
        "history" => user::status::history::run,
      },
    },
  }
}
