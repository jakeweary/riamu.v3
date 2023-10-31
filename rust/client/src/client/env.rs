#![allow(non_snake_case)]

use std::env::{self, VarError};
use std::error::Error as StdError;
use std::result::Result as StdResult;

use serenity::all::*;

type Error = Box<dyn StdError + Send + Sync>;
type Result<T> = StdResult<T, Error>;
type EnvResult = StdResult<String, VarError>;

macro_rules! fallback {
  ($fallback:tt) => { $fallback };
  ($fallback:tt $($tt:tt)+) => { $($tt)+ };
}

macro_rules! impl_env {
  ($($ENV:ident => $field:ident$(: |$ident:ident| -> $T:ty $block:block)?;)*) => {
    #[derive(Debug)]
    pub struct Env {
      $(pub $field: fallback!(String $($T)*)),*
    }

    impl Env {
      pub fn load() -> Self {
        $(let $field = {
          let name = stringify!($ENV);
          let env = env::var(name);
          $(
            let map = |$ident: EnvResult| -> Result<$T> { Ok($block) };
            let env = map(env);
          )?
          env.expect(name)
        };)*

        Self { $($field),* }
      }
    }
  }
}

impl_env! {
  DATABASE_URL => database_url;
  CACHE_WORKING_DIR => cache_working_dir;
  CACHE_BASE_URL => cache_base_url;
  CACHE_LIMIT_GiB => cache_limit_GiB: |id| -> u64 { id?.parse()? };
  DISCORD_TOKEN => discord_token;
  DISCORD_DEV_SERVER_ID => discord_dev_server: |id| -> GuildId { id?.parse::<u64>()?.into() };
  DISCORD_DEV_SERVER_INVITE => discord_dev_server_invite;
  DEEZER_ARL => deezer_arl;
  SPOTIFY_APP_ID => spotify_app_id;
  SPOTIFY_APP_SECRET => spotify_app_secret;
  IMGUR_APP_ID => imgur_app_id;
  IMGUR_APP_SECRET => imgur_app_secret;
  OPENWEATHERMAP_API_KEY => openweathermap_api_key;
}
