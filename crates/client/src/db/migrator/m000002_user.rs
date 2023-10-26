use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up<'s, 'sm>(&'s self, manager: &'sm SchemaManager<'sm>) -> Result<(), DbErr> {
    let create = Table::create()
      .table(User::Table)
      .if_not_exists()
      .col(ColumnDef::new(User::Id).big_integer().not_null().primary_key())
      .col(ColumnDef::new(User::Messages).big_integer().not_null().default(0))
      .col(ColumnDef::new(User::Commands).big_integer().not_null().default(0))
      .to_owned();

    manager.create_table(create).await
  }
}

#[derive(DeriveIden)]
pub enum User {
  Table,
  Id,
  Messages,
  Commands,
}
