use sqlx::prelude::FromRow;

use crate::{core::models::db::TableColumn, mysql::queries::convert_type::convert_data_type};

#[derive(FromRow)]
pub struct MySqlTableColumn {
    pub table_name: String,
    pub table_comment: Option<String>,
    pub column_name: String,
    pub column_comment: Option<String>,
    pub udt_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub array_depth: i16,
    pub is_unique: bool,
    pub is_primary_key: bool,
    pub foreign_key_table: Option<String>,
    pub foreign_key_id: Option<String>,
    pub table_schema: String,
    pub is_auto_populated: bool,
}

impl From<MySqlTableColumn> for TableColumn {
    fn from(value: MySqlTableColumn) -> Self {
        let recommended_rust_type = convert_data_type(&value.udt_name);

        let udt_name = if value.data_type.as_str() == "enum" {
            value.column_name.clone()
        } else {
            value.udt_name.clone()
        };

        Self {
            column_name: value.column_name,
            udt_name,
            array_depth: value.array_depth,
            data_type: value.data_type,
            is_nullable: value.is_nullable,
            is_unique: value.is_unique,
            is_primary_key: value.is_primary_key,
            foreign_key_table: value.foreign_key_table,
            foreign_key_id: value.foreign_key_id,
            recommended_rust_type,
            column_comment: value.column_comment,
            is_auto_populated: value.is_auto_populated,
        }
    }
}
