use std::error::Error as StdError;
use std::io::Result as IoResult;
use std::panic::AssertUnwindSafe;
use std::result::Result as StdResult;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ::serenity::all as serenity;
use futures::{FutureExt, TryFutureExt};
use lib::discord::colors;
use pyo3::{PyErr, Python};
use tokio::signal;
use tokio::signal::unix::{signal, *};

use crate::cache::LruFileCache;
use crate::db;

pub use self::command::*;
pub use self::command_error::*;
pub use self::commands::{CommandTree, Commands};
pub use self::context::*;
pub use self::env::*;
pub use self::traits::*;

pub(crate) use self::commands::commands;

mod command;
mod command_error;
mod commands;
mod context;
mod env;
mod traits;
mod util;

pub type Error = Box<dyn StdError + Send + Sync>;
pub type Result<T> = StdResult<T, Error>;

// pub type Error = anyhow::Error;
// pub type Result<T> = anyhow::Result<T>;

#[derive(Debug)]
pub struct Client {
  pub env: Env,
  pub commands: Commands,
  pub cache: Arc<LruFileCache>,
  pub db: db::Pool,
}

impl Client {
  pub async fn start() -> Result<()> {
    c::fontconfig::add_dir("assets/fonts")?;

    let env = Env::load();
    let commands = crate::commands::build();
    let db = db::init(&env.database_url).await?;
    let cache = {
      let base_url = &env.cache_base_url;
      let working_dir = &env.cache_working_dir;
      let limit_bytes = env.cache_limit_GiB << 30;
      let cache = LruFileCache::new(base_url, working_dir, limit_bytes).await?;
      Arc::new(cache)
    };

    let client = Self {
      env,
      commands,
      cache: cache.clone(),
      db,
    };

    let intents = serenity::GatewayIntents::all();
    let mut client = serenity::Client::builder(&client.env.discord_token, intents)
      .raw_event_handler(client)
      .await?;

    let shard_manager = client.shard_manager.clone();
    let exit = async move {
      let mut sigint = signal(SignalKind::interrupt())?;
      let mut sigterm = signal(SignalKind::terminate())?;

      tokio::select! {
        biased;
        _ = sigint.recv() => {},
        _ = sigterm.recv() => {},
        r = signal::ctrl_c() => r?,
      }

      let timeout = Duration::from_secs(10);
      let shutdown = tokio::time::timeout(timeout, async move {
        tracing::info!("shutting down…");
        shard_manager.shutdown_all().await;
      });

      if shutdown.await.is_err() {
        tracing::warn!("failed to shutdown gracefully (timeout)");
      }

      IoResult::Ok(())
    };

    tokio::select! {
      biased;
      r = client.start() => r?,
      r = cache.watch() => r?,
      r = exit => r?,
    }

    Ok(())
  }

  #[tracing::instrument(name="cmd", skip_all, fields(id=%ctx.id))]
  async fn handle_command(&self, ctx: Context<'_>) {
    let start = Instant::now();

    self.log_command(&ctx).await;

    let run = async {
      if ctx.command.owner_only {
        let info = ctx.serenity.http.get_current_application_info().await?;
        if ctx.event.user.id != info.owner.unwrap().id {
          err::message!("this command is owner-only");
        }
      }

      (ctx.command.run)(&ctx).await
    };

    match AssertUnwindSafe(run).catch_unwind().await {
      Ok(Ok(_)) => {}
      Ok(Err(err)) => {
        if let Some(err) = err.downcast_ref::<PyErr>() {
          Python::with_gil(|py| err.print(py));
        }
        match err.downcast() {
          Ok(err) => match &*err {
            CommandError::Timeout => {
              tracing::info!("timeout");
              drop(ctx.event.delete_response(&ctx).await);
            }
            CommandError::Message(msg) => {
              tracing::info!(%msg, "error");
              self.report_error(&ctx, Some(msg)).await;
            }
          },
          Err(err) => {
            tracing::error!(display=%err, "error");
            tracing::error!(debug=?err, "error");
            self.report_error(&ctx, None).await;
          }
        }
      }
      Err(panic) => {
        let panic = util::panic_message(panic);
        tracing::error!(%panic, "panic");
        self.report_error(&ctx, None).await;
      }
    }

    tracing::debug!("finished ({:.3?})", start.elapsed());
  }

  async fn log_command(&self, ctx: &Context<'_>) {
    tracing::info!("{}", util::SlashCommandDisplay(&ctx.event.data));

    tracing::debug!(cmd=%ctx.event.id);
    tracing::debug!(user=%ctx.event.user.id, tag=%ctx.event.user.tag());

    match ctx.event.channel_id.name(ctx).await {
      Ok(name) => tracing::debug!(channel=%ctx.event.channel_id, %name),
      Err(_) => tracing::debug!(channel=%ctx.event.channel_id),
    }

    if let Some(id) = ctx.event.guild_id {
      match id.name(ctx) {
        Some(name) => tracing::debug!(server=%id, %name),
        None => tracing::debug!(server=%id),
      }
    }
  }

  async fn report_error(&self, ctx: &Context<'_>, msg: Option<&str>) {
    tracing::trace!("reporting error");

    #[rustfmt::skip]
    let default = || format!(concat!(
      "**OOPSIE WOOPSIE!!** Uwu We made a fucky wucky!! A wittle fucko boingo! ",
      "The code monkeys at our [headquarters]({}) are working VEWY HAWD to fix this!"
    ), self.env.discord_dev_server_invite);

    let embed = serenity::CreateEmbed::new()
      .color(colors::ERROR.light)
      .description(msg.map_or_else(default, |msg| format!("**Error:** {}", msg)))
      .footer(serenity::CreateEmbedFooter::new(format!("ERROR ID {}", ctx.id)));

    let create = || {
      let embed = embed.clone();
      let message = serenity::CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);
      let message = serenity::CreateInteractionResponse::Message(message);
      ctx.event.create_response(ctx, message)
    };

    let followup = || async {
      // TODO: probably should delete followups as well, somehow
      ctx.event.delete_response(ctx).await?;

      let embed = embed.clone();
      let message = serenity::CreateInteractionResponseFollowup::new()
        .embed(embed)
        .ephemeral(true);
      ctx.event.create_followup(ctx, message).await?;

      Ok::<_, serenity::Error>(())
    };

    if let Err(err) = create().or_else(|_| followup()).await {
      tracing::warn!(%err, "failed to report error");
    };
  }

  async fn register_commands(&self, ctx: serenity::Context) {
    let commands = commands::serialize(&self.commands);
    match serenity::Command::set_global_commands(&ctx, commands).await {
      Ok(commands) => tracing::debug!("registered {} global commands", commands.len()),
      Err(err) => tracing::error!(%err, "failed to register global commands"),
    }

    let guild = self.env.discord_dev_server;
    let commands = Default::default();
    match guild.set_commands(&ctx, commands).await {
      Ok(commands) => tracing::debug!("registered {} guild-local commands", commands.len()),
      Err(err) => tracing::error!(%err, "failed to register guild-local commands"),
    }
  }

  async fn on_ready(&self, ctx: serenity::Context, ready: serenity::Ready) {
    let (r#as, id) = (ready.user.tag(), ready.user.id.get());
    tracing::info!(%r#as, id, "connected");
    self.register_commands(ctx).await;
  }

  async fn on_command(&self, ctx: serenity::Context, cmd: serenity::CommandInteraction) {
    let ctx = Context::new(self, &ctx, &cmd);
    self.handle_command(ctx).await;
  }
}

#[serenity::async_trait]
impl serenity::RawEventHandler for Client {
  async fn raw_event(&self, ctx: serenity::Context, event: serenity::Event) {
    let mut messages = 0;
    let mut commands = 0;

    let mut user_id = None;
    let mut user_name = None;
    let mut user_status = None;

    {
      use serenity::*;

      match event {
        Event::Ready(ReadyEvent { ready, .. }) => {
          self.on_ready(ctx, ready).await;
        }
        Event::InteractionCreate(InteractionCreateEvent {
          interaction: Interaction::Command(command),
          ..
        }) => {
          commands += 1;
          user_id = Some(command.user.id);
          user_name = Some(command.user.name.clone());
          self.on_command(ctx, command).await;
        }
        Event::MessageCreate(MessageCreateEvent { message, .. }) => {
          messages += 1;
          user_id = Some(message.author.id);
          user_name = Some(message.author.name.clone());
        }
        Event::PresenceUpdate(PresenceUpdateEvent { presence, .. }) => {
          user_id = Some(presence.user.id);
          user_name = presence.user.name.clone();
          user_status = Some(presence.into());
        }
        _ => {}
      }
    }

    let inc = db::counters::increment(&self.db, |names| {
      names.push_bind("events");
      if messages > 0 {
        names.push_bind("messages");
      }
      if commands > 0 {
        names.push_bind("commands");
      }
    });
    inc.await.unwrap();

    if let Some(uid) = user_id {
      let upsert = db::users::upsert(&self.db, uid, user_name, messages, commands);
      upsert.await.unwrap();
    }

    if let (Some(uid), Some(status)) = (user_id, user_status) {
      let insert = db::statuses::insert(&self.db, uid, status);
      insert.await.unwrap();
    }
  }
}
