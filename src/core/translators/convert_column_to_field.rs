use crate::core::models::{
    db::TableColumn,
    rust::{auto_attribute, key_attribute, unique_attribute, RustDbSetField},
};
use convert_case::{Case, Casing};

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

    let mut attributes = vec![];
    if column.is_auto_populated {
        attributes.push(auto_attribute());
    }
    if column.is_primary_key {
        attributes.push(key_attribute());
    } else if column.is_unique {
        attributes.push(unique_attribute());
    }

    if let Some(field_type) = maybe_field_type {
        return Some(RustDbSetField {
            field_name,
            field_type,
            is_optional: column.is_nullable,
            attributes,
            comment: column.column_comment.clone(),
        });
    }
    None
}
