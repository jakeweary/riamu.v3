use sea_orm::*;
use sea_orm_migration::prelude::*;

use self::migrator::Migrator;

pub mod entities;
pub mod migrator;
pub mod types;

pub async fn init(options: impl Into<ConnectOptions>) -> Result<DbConn, DbErr> {
  tracing::debug!("initializing database connection…");
  let connection = Database::connect(options).await?;

  tracing::debug!("applying all pending migrations…");
  Migrator::up(&connection, None).await?;

  Ok(connection)
}
