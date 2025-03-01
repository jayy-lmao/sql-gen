use convert_case::{Case, Casing};
use pluralizer::pluralize;

use crate::core::models::{db::TableColumnBuilder, rust::RustDbSetField};

use super::models::{
    db::{Table, TableColumn},
    rust::RustDbSetStruct,
};

#[derive(Default)]
pub struct TableToStructOptions {
    override_struct_name_for_tables: Vec<(String, String)>,
}

pub fn convert_column_to_field(
    column: TableColumn,
    options: TableToStructOptions,
) -> RustDbSetField {
    todo!()
}

pub fn convert_table_to_struct(table: Table, options: TableToStructOptions) -> RustDbSetStruct {
    let table_name_pascal_case = table.table_name.clone().to_case(Case::Pascal);
    let table_name_singular = pluralize(&table_name_pascal_case, 1, false);

    let maybe_override = options
        .override_struct_name_for_tables
        .iter()
        .find(|(table_name, _)| table_name == &table.table_name)
        .map(|(_, struct_name)| struct_name);

    let struct_name = maybe_override.unwrap_or(&table_name_singular).clone();

    RustDbSetStruct {
        struct_name,
        table_name: Some(table.table_name),
        fields: vec![],
    }
}

#[test]
fn can_convert_empty_table_to_struct() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![],
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    pretty_assertions::assert_eq!(
        rust_struct,
        RustDbSetStruct {
            struct_name: "Product".to_string(),
            table_name: Some("products".to_string()),
            fields: vec![]
        }
    )
}

#[test]
fn can_convert_empty_table_to_struct_2() {
    let table = Table {
        table_name: "inventories".to_string(),
        table_schema: "public".to_string(),
        columns: vec![],
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    pretty_assertions::assert_eq!(
        rust_struct,
        RustDbSetStruct {
            struct_name: "Inventory".to_string(),
            table_name: Some("inventories".to_string()),
            fields: vec![]
        }
    )
}

#[test]
fn can_convert_table_with_basic_column() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("title", "text", "text").build()],
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    pretty_assertions::assert_eq!(
        rust_struct,
        RustDbSetStruct {
            struct_name: "Product".to_string(),
            table_name: Some("products".to_string()),
            fields: vec![RustDbSetField {
                field_name: "title".to_string(),
                field_type: "String".to_string()
            }]
        }
    )
}

#[test]
fn can_convert_table_to_struct_with_override() {
    let table = Table {
        table_name: "users".to_string(),
        table_schema: "public".to_string(),
        columns: vec![],
    };

    let mut table_to_struct_options = TableToStructOptions::default();

    table_to_struct_options.override_struct_name_for_tables =
        vec![("users".to_string(), "Customer".to_string())];

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);
    pretty_assertions::assert_eq!(
        rust_struct,
        RustDbSetStruct {
            struct_name: "Customer".to_string(),
            table_name: Some("users".to_string()),
            fields: vec![]
        }
    )
}
