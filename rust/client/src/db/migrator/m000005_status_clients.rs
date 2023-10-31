use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up<'s, 'sm>(&'s self, manager: &'sm SchemaManager<'sm>) -> Result<(), DbErr> {
    let desktop = Table::alter()
      .table(Status::Table)
      .add_column(ColumnDef::new(Status::Desktop).string())
      .to_owned();

    let mobile = Table::alter()
      .table(Status::Table)
      .add_column(ColumnDef::new(Status::Mobile).string())
      .to_owned();

    let web = Table::alter()
      .table(Status::Table)
      .add_column(ColumnDef::new(Status::Web).string())
      .to_owned();

    manager.alter_table(desktop).await?;
    manager.alter_table(mobile).await?;
    manager.alter_table(web).await
  }
}

#[derive(DeriveIden)]
pub enum Status {
  Table,
  Desktop,
  Mobile,
  Web,
}
