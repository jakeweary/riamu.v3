use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{error, result};

use ::serenity::all as serenity;
use cache::LruFileCache;
use discord::colors;
use futures::{FutureExt, TryFutureExt};
use pyo3::{PyErr, Python};
use tokio::signal::{self, unix::*};

use crate::commands::tree as commands;
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

pub type Error = Box<dyn error::Error + Send + Sync>;
pub type Result<T> = result::Result<T, Error>;

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
    let db = db::init(&env.database_url).await?;
    let cache = {
      let base_url = env.cache_base_url.clone();
      let working_dir = env.cache_working_dir.clone();
      let limit_bytes = env.cache_limit_GiB << 30;
      let cache = LruFileCache::new(base_url, working_dir, limit_bytes);
      Arc::new(cache.await?)
    };

    let client = Self {
      env,
      commands: commands(),
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

      let shutdown = shard_manager.shutdown_all();
      let shutdown = tokio::time::timeout(Duration::from_secs(5), shutdown);
      let sleep = tokio::time::sleep(Duration::from_secs(5));

      tracing::info!("shutting down…");
      if shutdown.await.is_ok() {
        sleep.await // don't exit immediately after sending shutdown signal
      }

      // if the shutdown was successful this line should not be reached
      tracing::warn!("failed to shutdown gracefully");

      Result::Ok(())
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
  async fn handle_command(&self, ctx: Context<'_>) -> serenity::Result<()> {
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
              ctx.event.delete_response(&ctx).await?;
            }
            CommandError::Message(msg) => {
              tracing::info!(%msg, "error");
              self.report_error(&ctx, Some(msg)).await?;
            }
          },
          Err(err) => {
            tracing::error!(display=%err, "error");
            tracing::error!(debug=?err, "error");
            self.report_error(&ctx, None).await?;
          }
        }
      }
      Err(panic) => {
        let panic = util::panic_message(panic);
        tracing::error!(%panic, "panic");
        self.report_error(&ctx, None).await?;
      }
    }

    tracing::debug!("finished ({:.3?})", start.elapsed());

    Ok(())
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

  async fn report_error(&self, ctx: &Context<'_>, msg: Option<&str>) -> serenity::Result<()> {
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

    create().or_else(|_| followup()).await?;

    Ok(())
  }

  async fn register_commands(&self, ctx: &serenity::Context) -> serenity::Result<()> {
    let commands = commands::serialize(&self.commands);
    let commands = serenity::Command::set_global_commands(ctx, commands).await?;
    tracing::debug!("registered {} global commands", commands.len());

    let guild = self.env.discord_dev_server;
    let commands = Default::default();
    let commands = guild.set_commands(ctx, commands).await?;
    tracing::debug!("registered {} guild-local commands", commands.len());

    Ok(())
  }

  async fn track_event(&self, event: &serenity::Event) -> Result<()> {
    use serenity::*;

    let mut messages = 0;
    let mut commands = 0;

    let mut user_id = None;
    let mut user_name = None;
    let mut user_status = None;

    match event {
      Event::InteractionCreate(InteractionCreateEvent {
        interaction: Interaction::Command(command),
        ..
      }) => {
        commands += 1;
        user_id = Some(command.user.id);
        user_name = Some(command.user.name.clone());
      }
      Event::MessageCreate(MessageCreateEvent { message, .. }) => {
        messages += 1;
        user_id = Some(message.author.id);
        user_name = Some(message.author.name.clone());
      }
      Event::PresenceUpdate(PresenceUpdateEvent { presence, .. }) => {
        user_id = Some(presence.user.id);
        user_name.clone_from(&presence.user.name);
        user_status = Some(presence.into());
      }
      _ => {}
    }

    let pairs = [("events", 1), ("messages", messages), ("commands", commands)];
    db::counters::increment(&self.db, &pairs).await?;

    if let Some(uid) = user_id {
      db::users::upsert(&self.db, uid, user_name, messages, commands).await?;
    }

    if let (Some(uid), Some(status)) = (user_id, user_status) {
      db::statuses::insert(&self.db, uid, status).await?;
    }

    Ok(())
  }

  async fn on_event(&self, ctx: &serenity::Context, event: &serenity::Event) -> Result<()> {
    use serenity::*;

    self.track_event(event).await?;

    match event {
      Event::Ready(ReadyEvent { ready, .. }) => {
        self.on_ready(ctx, ready).await?;
      }
      Event::MessageCreate(MessageCreateEvent { message, .. }) => {
        self.on_message(ctx, message).await?;
      }
      Event::InteractionCreate(InteractionCreateEvent {
        interaction: Interaction::Command(command),
        ..
      }) => {
        self.on_command(ctx, command).await?;
      }
      _ => {}
    }

    Ok(())
  }

  async fn on_ready(&self, ctx: &serenity::Context, ready: &serenity::Ready) -> serenity::Result<()> {
    let (r#as, id) = (ready.user.tag(), ready.user.id.get());
    tracing::info!(%r#as, id, "connected");
    self.register_commands(ctx).await
  }

  async fn on_command(&self, ctx: &serenity::Context, cmd: &serenity::CommandInteraction) -> serenity::Result<()> {
    let ctx = Context::new(self, ctx, cmd);
    self.handle_command(ctx).await
  }

  async fn on_message(&self, _ctx: &serenity::Context, _msg: &serenity::Message) -> Result<()> {
    Ok(())
  }
}

#[serenity::async_trait]
impl serenity::RawEventHandler for Client {
  async fn raw_event(&self, ctx: serenity::Context, event: serenity::Event) {
    if let Err(err) = self.on_event(&ctx, &event).await {
      tracing::error!(display=%err, debug=?err, "unhandled error while event handling");
    }
  }
}
