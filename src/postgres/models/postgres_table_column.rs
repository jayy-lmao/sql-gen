use sqlx::prelude::FromRow;

use crate::core::models::TableColumn;

#[derive(FromRow)]
pub struct PostgresTableColumn {
    pub(crate) table_name: String,
    pub(crate) column_name: String,
    pub(crate) udt_name: String,
    pub(crate) data_type: String,
    pub(crate) is_nullable: bool,
    pub(crate) is_unique: bool,
    pub(crate) is_primary_key: bool,
    pub(crate) foreign_key_table: Option<String>,
    pub(crate) foreign_key_id: Option<String>,
    pub(crate) table_schema: String,
}

impl From<PostgresTableColumn> for TableColumn {
    fn from(value: PostgresTableColumn) -> Self {
        Self {
            column_name: value.column_name,
            udt_name: value.udt_name,
            data_type: value.data_type,
            is_nullable: value.is_nullable,
            is_unique: value.is_unique,
            is_primary_key: value.is_primary_key,
            foreign_key_table: value.foreign_key_table,
            foreign_key_id: value.foreign_key_id,
        }
    }
}
