use sqlx::query_builder::Separated;

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

pub async fn increment<F>(pool: &Pool, f: F) -> sqlx::Result<QueryResult>
where
  F: FnOnce(&mut Separated<'_, '_, Database, &str>),
{
  let mut q = QueryBuilder::new(
    " update counters set count = count + 1
      where name in ",
  );

  let mut sep = q.separated(", ");
  sep.push_unseparated("(");
  f(&mut sep);
  sep.push_unseparated(")");

  q.build().execute(pool).await
}
