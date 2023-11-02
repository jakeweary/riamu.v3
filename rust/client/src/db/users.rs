use serenity::all::*;

use super::*;

pub async fn upsert(
  pool: &Pool,
  user_id: UserId,
  user_name: Option<String>,
  messages: u32,
  commands: u32,
) -> sqlx::Result<QueryResult> {
  let q = sqlx::query(
    " insert into users (id, name, messages, commands) values (?, ?, ?, ?)
      on conflict do update set
        name = coalesce(excluded.name, name),
        messages = excluded.messages + messages,
        commands = excluded.commands + commands ",
  );

  let q = q
    .bind(user_id.get() as i64)
    .bind(user_name)
    .bind(messages)
    .bind(commands);

  q.execute(pool).await
}
