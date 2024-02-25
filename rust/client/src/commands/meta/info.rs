use std::fmt::{self, Write};
use std::result::Result as StdResult;
use std::time::Instant;
use std::{env, iter, process, str};

use lib::fmt::num::Format;
use procfs::{process::*, *};
use serenity::all::*;

use crate::client::{Context, Result};
use crate::db::{self, counters::Counter};

#[macros::command(desc = "Show some technical info about me")]
pub async fn run(ctx: &Context<'_>) -> Result<()> {
  let prev = KernelStats::current()?;
  let rtt = Instant::now();
  ctx.event.defer(ctx).await?;
  let rtt = rtt.elapsed().as_secs_f64();
  let curr = KernelStats::current()?;

  let load = LoadAverage::current()?;
  let uptime = Uptime::current()?;
  let meminfo = Meminfo::current()?;
  let me = Process::myself()?;
  let stat = me.stat()?;
  let statm = me.statm()?;

  let cache = &*ctx.serenity.cache;
  let servers = cache.guild_count();
  let channels = cache.guild_channel_count();
  let users = cache.user_count();

  let counters = db::counters::all(&ctx.client.db).await?;

  let embed = CreateEmbed::new()
    .description(desc(&load, &prev, &curr)?)
    .field("System", system(&meminfo, &uptime)?, true)
    .field("Process", process(&stat, &statm, &uptime)?, true)
    .field("Discord", discord(servers, channels, users, rtt)?, true)
    .field("Runtime Info", runtime_info(&versions()?)?, false)
    .field("Build Info", build_info()?, false)
    .footer(footer(&counters)?);

  tracing::debug!("sending response…");
  let edit = EditInteractionResponse::new().embed(embed);
  ctx.event.edit_response(ctx, edit).await?;

  Ok(())
}

// ---

#[rustfmt::skip]
fn desc(load: &LoadAverage, prev: &KernelStats, curr: &KernelStats) -> StdResult<String, fmt::Error> {
  let mut acc = String::new();
  writeln!(acc, "`{:.2}` `{:.2}` `{:.2}` load average", load.one, load.five, load.fifteen)?;
  for (prev, curr) in iter::zip(&prev.cpu_time, &curr.cpu_time) {
    write!(acc, "`{:.0}%` ", 1e2 * cpu_usage(prev, curr))?;
  }
  writeln!(acc, "cpu usage per core")?;
  writeln!(acc, "`{:.0}%` total cpu usage", 1e2 * cpu_usage(&prev.total, &curr.total))?;
  Ok(acc)
}

fn footer(counters: &[Counter]) -> StdResult<CreateEmbedFooter, fmt::Error> {
  let mut acc = String::new();
  for (i, counter) in counters.iter().enumerate() {
    let sep = if i == 0 { "received" } else { "," };
    write!(acc, "{} {} {}", sep, counter.count.k(), counter.name)?;
  }
  Ok(CreateEmbedFooter::new(acc))
}

fn system(mem: &Meminfo, uptime: &Uptime) -> StdResult<String, fmt::Error> {
  let mut acc = String::new();
  if let Some(available) = mem.mem_available {
    let used = mem.mem_total - available;
    writeln!(acc, "`{}B` used", used.iec())?;
    writeln!(acc, "`{}B` available", available.iec())?;
  } else {
    writeln!(acc, "`{}B` free", mem.mem_free.iec())?;
  }
  writeln!(acc, "`{}B` total", mem.mem_total.iec())?;
  writeln!(acc, "`{}` uptime", lib::fmt::dhms(uptime.uptime as u64))?;
  Ok(acc)
}

fn process(stat: &Stat, statm: &StatM, uptime: &Uptime) -> StdResult<String, fmt::Error> {
  let page_size = procfs::page_size();
  let mut acc = String::new();
  writeln!(acc, "`{}B` virtual", (page_size * statm.size).iec())?;
  writeln!(acc, "`{}B` resident", (page_size * statm.resident).iec())?;
  writeln!(acc, "`{}B` shared", (page_size * statm.shared).iec())?;
  writeln!(acc, "`{}` uptime", lib::fmt::dhms(process_uptime(uptime, stat)))?;
  Ok(acc)
}

fn discord(servers: usize, channels: usize, users: usize, rtt: f64) -> StdResult<String, fmt::Error> {
  let mut acc = String::new();
  writeln!(acc, "`{}` servers", servers.k())?;
  writeln!(acc, "`{}` channels", channels.k())?;
  writeln!(acc, "`{}` users", users.k())?;
  writeln!(acc, "`{:.0}ms` ping", 1e3 * rtt)?;
  Ok(acc)
}

fn runtime_info(versions: &[(String, String)]) -> StdResult<String, fmt::Error> {
  let mut acc = String::new();
  for (i, (k, v)) in versions.iter().enumerate() {
    let sep = if i % 3 == 2 { '\n' } else { ' ' };
    let k = k.replace('-', "\u{2011}");
    write!(acc, "{k}\u{00A0}`{v}`{sep}")?;
  }
  Ok(acc)
}

fn build_info() -> StdResult<String, fmt::Error> {
  let normalize = |input: &str| {
    let (name, input) = input.split_once(' ')?;
    let (version, input) = input.split_once(' ')?;
    let (hash, date) = input.trim_matches(&['(', ')'] as &[_]).split_once(' ')?;
    Some(format!("{} {:<14} {} {}", name, version, &hash[..8], date))
  };

  let crate_version = {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let hash = env!("BUILD_COMMIT_HASH");
    let date = env!("BUILD_COMMIT_DATE");
    format!("{name} {version} ({hash} {date})")
  };

  let info = {
    let timestamp = env!("BUILD_TIMESTAMP");
    let profile = env!("BUILD_PROFILE");
    let toolchain = env!("BUILD_RUSTUP_TOOLCHAIN");
    let rustc_version = env!("BUILD_RUSTC_VERSION");
    let cargo_version = env!("BUILD_CARGO_VERSION");

    let mut acc = String::new();
    writeln!(acc, "built <t:{}:R> in `{}` mode", timestamp, profile)?;
    writeln!(acc, "with `{}` toolchain", toolchain)?;
    writeln!(acc, "```")?;
    writeln!(acc, "{}", normalize(&crate_version).unwrap())?;
    writeln!(acc, "{}", normalize(rustc_version).unwrap())?;
    writeln!(acc, "{}", normalize(cargo_version).unwrap())?;
    writeln!(acc, "```")?;
    acc
  };

  Ok(info)
}

// ---

fn versions() -> Result<Vec<(String, String)>> {
  let pango = ("pango".into(), pango::version_string().into());
  let cairo = ("cairo".into(), cairo::version_string().into());
  let rsvg = ("rsvg".into(), c::rsvg::version_string());
  let ffmpeg = {
    let output = process::Command::new("ffmpeg").arg("-version").output()?;
    let version = str::from_utf8(&output.stdout)?.split_ascii_whitespace().nth(2).unwrap();
    let version = version.split_once('-').map_or(version, |(v, _)| v);
    ("ffmpeg".into(), version.into())
  };

  let mut versions = vec![pango, cairo, rsvg, ffmpeg];
  versions.append(&mut python::lib::versions()?);
  Ok(versions)
}

// ---

fn process_uptime(uptime: &Uptime, stat: &Stat) -> u64 {
  let ticks_per_second = procfs::ticks_per_second();
  uptime.uptime as u64 - (stat.starttime / ticks_per_second)
}

fn cpu_usage(prev: &CpuTime, curr: &CpuTime) -> f64 {
  // https://stackoverflow.com/a/23376195
  fn busy_total(cpu_time: &CpuTime) -> (u64, u64) {
    #[rustfmt::skip]
    let CpuTime { user, nice, system, idle, iowait, irq, softirq, steal, .. } = cpu_time;
    let idle = idle + iowait.unwrap_or(0);
    let busy = user + nice + system + irq.unwrap_or(0) + softirq.unwrap_or(0) + steal.unwrap_or(0);
    let total = idle + busy;
    (busy, total)
  }

  let (curr_busy, curr_total) = busy_total(curr);
  let (prev_busy, prev_total) = busy_total(prev);
  match (curr_busy - prev_busy, curr_total - prev_total) {
    (_, 0) => 0.0,
    (b, t) => b as f64 / t as f64,
  }
}
