use std::hash::{DefaultHasher, Hash, Hasher};

use lib::gcra;

use super::*;

pub use gcra::*;

pub async fn update(pool: &Pool, key: impl Hash, rate: Rate) -> sqlx::Result<ResultAndInfo> {
  update_n(pool, key, rate, 1.0).await
}

pub async fn update_n(pool: &Pool, key: impl Hash, rate: Rate, n: f64) -> sqlx::Result<ResultAndInfo> {
  let key = hash(key) as i64;
  update_n_inner(pool, key, rate, n).await
}

async fn update_n_inner(pool: &Pool, key: i64, rate: Rate, n: f64) -> sqlx::Result<ResultAndInfo> {
  let mut tx = pool.begin().await?;

  let q = sqlx::query_scalar("select tat from gcra where key = ?");
  let tat = q.bind(key).fetch_optional(&mut *tx).await?;
  let tat = tat.unwrap_or(0_i64) as u64;

  let mut state = gcra::State { tat };
  let (result, info) = state.update(rate, n);

  if result.is_ok() {
    let q = sqlx::query("insert or replace into gcra (key, tat) values (?, ?)");
    q.bind(key).bind(state.tat as i64).execute(&mut *tx).await?;
  }

  tx.commit().await?;

  Ok((result, info))
}

fn hash(input: impl Hash) -> u64 {
  let mut hasher = DefaultHasher::new();
  input.hash(&mut hasher);
  hasher.finish()
}
