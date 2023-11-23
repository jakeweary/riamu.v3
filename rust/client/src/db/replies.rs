use rand::prelude::*;

use super::*;

pub async fn random_reply(pool: &Pool) -> sqlx::Result<Option<String>> {
  let mut tx = pool.begin().await?;

  let q = sqlx::query_scalar("select count(*) from replies");
  let count = q.fetch_one(&mut *tx).await?;

  let reply = match count {
    0 => None,
    n => {
      let q = sqlx::query_scalar("select reply from replies limit 1 offset ?");
      let offset = thread_rng().gen_range(0_i64..n);
      let reply = q.bind(offset).fetch_one(&mut *tx).await?;

      Some(reply)
    }
  };

  tx.commit().await?;

  Ok(reply)
}
