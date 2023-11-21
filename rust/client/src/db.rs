use std::str::FromStr;

use sqlx::sqlite::*;

pub type Pool = sqlx::Pool<Sqlite>;
pub type QueryBuilder<'a> = sqlx::QueryBuilder<'a, Sqlite>;
pub type QueryResult = SqliteQueryResult;

pub mod counters;
pub mod ratelimits;
pub mod statuses;
pub mod users;

pub async fn init(url: &str) -> sqlx::Result<Pool> {
  let options = SqliteConnectOptions::from_str(url)?
    .synchronous(SqliteSynchronous::Normal)
    .locking_mode(SqliteLockingMode::Normal)
    .journal_mode(SqliteJournalMode::Wal)
    .extension("deps/sqlean");

  tracing::debug!("initializing database connection…");
  let pool = SqlitePoolOptions::new()
    .max_connections(4)
    .connect_with(options)
    .await?;

  tracing::debug!("applying all pending migrations…");
  sqlx::migrate!("../../migrations").run(&pool).await?;

  Ok(pool)
}
