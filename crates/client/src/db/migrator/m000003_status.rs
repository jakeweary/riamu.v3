use sea_orm_migration::prelude::*;

use super::m000002_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up<'s, 'sm>(&'s self, manager: &'sm SchemaManager<'sm>) -> Result<(), DbErr> {
    let create = Table::create()
      .table(Status::Table)
      .if_not_exists()
      .col({
        ColumnDef::new(Status::Id)
          .integer()
          .not_null()
          .auto_increment()
          .primary_key()
      })
      .col(ColumnDef::new(Status::User).big_integer().not_null())
      .col(ColumnDef::new(Status::Status).string().not_null())
      .col(ColumnDef::new(Status::Time).big_integer().not_null())
      .foreign_key({
        ForeignKey::create()
          .from(Status::Table, Status::User)
          .to(User::Table, User::Id)
      })
      .to_owned();

    manager.create_table(create).await
  }
}

#[derive(DeriveIden)]
pub enum Status {
  Table,
  Id,
  User,
  Status,
  Time,
}
