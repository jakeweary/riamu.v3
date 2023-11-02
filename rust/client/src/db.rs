use std::str::FromStr;

use sqlx::sqlite::*;

pub type Database = Sqlite;
pub type Pool = sqlx::Pool<Sqlite>;
pub type QueryBuilder<'a> = sqlx::QueryBuilder<'a, Sqlite>;
pub type QueryResult = SqliteQueryResult;

pub mod counters;
pub mod statuses;
pub mod users;

pub async fn init(url: &str) -> sqlx::Result<Pool> {
  tracing::debug!("initializing database connection…");
  let options = SqliteConnectOptions::from_str(url)?;
  let pool = SqlitePoolOptions::new().connect_with(options).await?;

  tracing::debug!("applying all pending migrations…");
  sqlx::migrate!("../../migrations").run(&pool).await?;

  Ok(pool)
}
