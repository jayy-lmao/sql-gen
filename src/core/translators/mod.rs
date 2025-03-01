use std::collections::HashMap;

use convert_case::{Case, Casing};
use pluralizer::pluralize;

use crate::core::models::rust::RustDbSetField;
#[cfg(test)]
use crate::core::models::db::TableColumnBuilder;


use super::models::{
    db::{Table, TableColumn},
    rust::RustDbSetStruct,
};

#[derive(Default)]
pub struct TableToStructOptions {
    override_name: Option<String>,
    column_overrides: HashMap<String, ColumnToFieldOptions>,
    type_overrides: HashMap<String, ColumnToFieldOptions>,
}
impl TableToStructOptions {
    fn add_column_override(&mut self, column_name: &str, options: ColumnToFieldOptions) {
        self.column_overrides.insert(column_name.to_string(), options);
    }
}

#[derive(Default,Clone)]
pub struct ColumnToFieldOptions {
    override_name: Option<String>,
    override_type: Option<String>,
}

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

pub fn convert_table_to_struct(table: Table, options: TableToStructOptions) -> RustDbSetStruct {
    let table_name_pascal_case = table.table_name.clone().to_case(Case::Pascal);
    let table_name_singular = pluralize(&table_name_pascal_case, 1, false);

    let maybe_override = options.override_name;

    let struct_name = maybe_override.unwrap_or(table_name_singular);
    let table_name = table.table_name.clone();
    let fields = table
        .columns
        .iter()
        .filter_map(|c| { 

            let column_override = options.column_overrides.get(&c.column_name).cloned();
            let type_override = options.type_overrides.get(&c.udt_name).cloned();
            let column_to_field_options = column_override.or(type_override).unwrap_or_default();
            let field = convert_column_to_field(c, column_to_field_options  );
            if field.is_none() {
                println!("WARNING: field {} in table {} has no user-defined type or recommended type for {}", c.column_name,&table.table_name,c.udt_name)
            }

            field

        })
        .collect();

    RustDbSetStruct {
        struct_name,
        table_name: Some(table_name),
        fields,
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
fn can_convert_table_to_struct_with_override() {
    let table = Table {
        table_name: "users".to_string(),
        table_schema: "public".to_string(),
        columns: vec![],
    };

    let table_to_struct_options = TableToStructOptions { override_name: Some("Customer".to_string()), ..Default::default()};

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
fn can_convert_table_with_column_type_override() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("id", "i32", "i32").build()],
    };

    let column_override = ColumnToFieldOptions { override_name: None, override_type: Some("String".to_string()) };
    let mut table_to_struct_options = TableToStructOptions::default();
    table_to_struct_options.add_column_override("id", column_override);

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);

    pretty_assertions::assert_eq!(
        rust_struct,
        RustDbSetStruct {
            struct_name: "Product".to_string(),
            table_name: Some("products".to_string()),
            fields: vec![RustDbSetField {
                field_name: "id".to_string(),
                field_type: "String".to_string()
            }]
        }
    )
}

