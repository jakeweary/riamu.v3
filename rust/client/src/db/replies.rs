use rand::prelude::*;

use super::*;

pub async fn random_reply(pool: &Pool) -> sqlx::Result<Option<String>> {
  let mut tx = pool.begin().await?;

  let q = sqlx::query_as("select count(*) from replies");
  let (count,) = q.fetch_one(&mut *tx).await?;

  let reply = match count {
    0 => None,
    n => {
      let q = sqlx::query_as("select reply from replies limit 1 offset ?");
      let i = Rng::gen_range::<i64, _>(&mut thread_rng(), 0..n);
      let (reply,) = q.bind(i).fetch_one(&mut *tx).await?;
      Some(reply)
    }
  };

  tx.commit().await?;

  Ok(reply)
}
