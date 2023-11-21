use std::hash::{DefaultHasher, Hash, Hasher};

use lib::gcra;

use super::*;

pub use gcra::*;

pub async fn update(pool: &Pool, key: impl Hash, rate: Rate) -> sqlx::Result<Info> {
  update_n(pool, key, rate, 1.0).await
}

pub async fn update_n(pool: &Pool, key: impl Hash, rate: Rate, n: f64) -> sqlx::Result<Info> {
  let key = hash(key) as i64;
  update_n_inner(pool, key, rate, n).await
}

async fn update_n_inner(pool: &Pool, key: i64, rate: Rate, n: f64) -> sqlx::Result<Info> {
  let mut tx = pool.begin().await?;

  let q = sqlx::query_as("select tat from gcra where key = ?");
  let row = q.bind(key).fetch_optional(&mut *tx).await?;
  let tat = row.map_or(0_i64, |(tat,)| tat) as u64;

  let mut state = gcra::State { tat };
  let info = state.update_n(rate, n);

  if info.result.is_ok() {
    let q = sqlx::query("insert or replace into gcra (key, tat) values (?, ?)");
    q.bind(key).bind(state.tat as i64).execute(&mut *tx).await?;
  }

  tx.commit().await?;

  Ok(info)
}

fn hash(input: impl Hash) -> u64 {
  let mut hasher = DefaultHasher::new();
  input.hash(&mut hasher);
  hasher.finish()
}
