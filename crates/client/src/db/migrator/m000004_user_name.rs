use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up<'s, 'sm>(&'s self, manager: &'sm SchemaManager<'sm>) -> Result<(), DbErr> {
    let alter = Table::alter()
      .table(User::Table)
      .add_column(ColumnDef::new(User::Name).string())
      .to_owned();

    manager.alter_table(alter).await
  }
}

#[derive(DeriveIden)]
pub enum User {
  Table,
  Name,
}
