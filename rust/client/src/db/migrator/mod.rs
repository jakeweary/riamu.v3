pub use sea_orm_migration::prelude::*;

mod m000001_counter;
mod m000002_user;
mod m000003_status;
mod m000004_user_name;
mod m000005_status_clients;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m000001_counter::Migration),
      Box::new(m000002_user::Migration),
      Box::new(m000003_status::Migration),
      Box::new(m000004_user_name::Migration),
      Box::new(m000005_status_clients::Migration),
    ]
  }
}
