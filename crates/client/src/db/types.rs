use sea_orm::prelude::*;
use sea_orm::sea_query::*;
use sea_orm::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DiscordId(pub u64);

impl TryFromU64 for DiscordId {
  fn try_from_u64(n: u64) -> Result<Self, DbErr> {
    Ok(Self(n))
  }
}

impl From<DiscordId> for Value {
  fn from(source: DiscordId) -> Self {
    (source.0 as i64).into()
  }
}

impl TryGetable for DiscordId {
  fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
    <i64 as TryGetable>::try_get_by(res, idx).map(|v| Self(v as u64))
  }
}

impl ValueType for DiscordId {
  fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
    <i64 as ValueType>::try_from(v).map(|v| Self(v as u64))
  }

  fn type_name() -> String {
    stringify!(DiscordId).to_owned()
  }

  fn array_type() -> ArrayType {
    ArrayType::BigUnsigned
  }

  fn column_type() -> ColumnType {
    ColumnType::BigUnsigned
  }
}
