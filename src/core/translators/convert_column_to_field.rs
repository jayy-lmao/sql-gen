use convert_case::{Case, Casing};

use crate::core::models::{db::TableColumn, rust::RustDbSetField};

use super::models::ColumnToFieldOptions;

pub fn convert_column_to_field(
    column: &TableColumn,
    options: ColumnToFieldOptions,
) -> Option<RustDbSetField> {
    let field_name = options
        .override_name
        .unwrap_or(column.column_name.to_case(Case::Snake));

    let maybe_field_type: Option<String> = options
        .override_type
        .or(column.recommended_rust_type.clone());

    if let Some(field_type) = maybe_field_type {
        return Some(RustDbSetField {
            field_name,
            field_type,
        });
    }
    None
}
