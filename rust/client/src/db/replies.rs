use super::*;

pub async fn random_reply(pool: &Pool) -> sqlx::Result<Option<String>> {
  let q = sqlx::query_scalar("select reply from replies order by random() limit 1");
  q.fetch_optional(pool).await
}
