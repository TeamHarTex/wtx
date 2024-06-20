use crate::database::{orm::TableAssociation, TableSuffix};

/// Contains [TableAssociation] plus some parameters gathered from other sources
#[derive(Debug)]
pub struct FullTableAssociation {
  association: TableAssociation,
  from_id_value: Option<u64>,
  to_table: &'static str,
  to_table_alias: Option<&'static str>,
  to_table_suffix: TableSuffix,
}

impl FullTableAssociation {
  #[inline]
  pub(crate) const fn new(
    association: TableAssociation,
    from_id_value: Option<u64>,
    to_table: &'static str,
    to_table_alias: Option<&'static str>,
    to_table_suffix: TableSuffix,
  ) -> Self {
    Self { association, from_id_value, to_table, to_table_alias, to_table_suffix }
  }

  /// See [TableAssociation].
  #[inline]
  pub const fn association(&self) -> &TableAssociation {
    &self.association
  }

  /// Coalesced ID value of source table
  #[inline]
  pub const fn from_id_value(&self) -> Option<u64> {
    self.from_id_value
  }

  /// Referenced table
  #[inline]
  pub const fn to_table(&self) -> &'static str {
    self.to_table
  }

  /// Referenced table alias
  #[inline]
  pub const fn to_table_alias(&self) -> Option<&'static str> {
    self.to_table_alias
  }

  /// Referenced table suffix
  #[inline]
  pub const fn to_table_suffix(&self) -> TableSuffix {
    self.to_table_suffix
  }
}
