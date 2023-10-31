use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up<'s, 'sm>(&'s self, manager: &'sm SchemaManager<'sm>) -> Result<(), DbErr> {
    let create = Table::create()
      .table(Counter::Table)
      .if_not_exists()
      .col(ColumnDef::new(Counter::Name).string().not_null().primary_key())
      .col(ColumnDef::new(Counter::Count).big_integer().not_null().default(0))
      .to_owned();

    let seed = Query::insert()
      .into_table(Counter::Table)
      .columns([Counter::Name])
      .values_panic(["events".into()])
      .values_panic(["messages".into()])
      .values_panic(["commands".into()])
      .to_owned();

    manager.create_table(create).await?;
    manager.exec_stmt(seed).await
  }
}

#[derive(DeriveIden)]
pub enum Counter {
  Table,
  Name,
  Count,
}
