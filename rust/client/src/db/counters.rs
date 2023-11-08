use super::*;

#[derive(sqlx::FromRow)]
pub struct Counter {
  pub name: String,
  pub count: i64,
}

pub async fn all(pool: &Pool) -> sqlx::Result<Vec<Counter>> {
  let q = sqlx::query_as("select name, count from counters");
  q.fetch_all(pool).await
}

pub async fn increment(pool: &Pool, pairs: &[(&str, u32)]) -> sqlx::Result<QueryResult> {
  let mut q = QueryBuilder::new("insert into counters values");
  let mut qs = q.separated(", ");
  for &(name, value) in pairs {
    qs.push("(")
      .push_bind_unseparated(name)
      .push_unseparated(", ")
      .push_bind_unseparated(value)
      .push_unseparated(")");
  }
  q.push("on conflict do update set count = excluded.count + count");
  q.build().execute(pool).await
}
