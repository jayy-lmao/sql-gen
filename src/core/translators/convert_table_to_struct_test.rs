use crate::core::{
    models::{
        db::{Table, TableColumnBuilder},
        rust::{RustDbSetField, RustDbSetStruct},
    },
    translators::{
        convert_table_to_struct::convert_table_to_struct,
        models::{ColumnToFieldOptions, TableToStructOptions},
    },
};
use pretty_assertions::assert_eq;

#[test]
fn can_convert_empty_table_to_struct() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![],
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    assert_eq!(
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
    assert_eq!(
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

    let table_to_struct_options = TableToStructOptions {
        override_name: Some("Customer".to_string()),
        ..Default::default()
    };

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);
    assert_eq!(
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
    assert_eq!(
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
fn should_ignore_columns_with_invalid_types() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("title", "badtype", "badtype").build()],
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            struct_name: "Product".to_string(),
            table_name: Some("products".to_string()),
            fields: vec![]
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

    let column_override = ColumnToFieldOptions {
        override_name: None,
        override_type: Some("String".to_string()),
    };
    let mut table_to_struct_options = TableToStructOptions::default();

    table_to_struct_options.add_column_override("id", column_override);

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);

    assert_eq!(
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

#[test]
fn can_convert_table_with_global_type_override() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("id", "int4", "int4").build()],
    };

    let type_override = ColumnToFieldOptions {
        override_name: None,
        override_type: Some("String".to_string()),
    };
    let mut table_to_struct_options = TableToStructOptions::default();

    table_to_struct_options.add_type_override("int4", type_override);

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);

    assert_eq!(
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

#[test]
fn column_override_takes_preference_over_global_type_override() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("price", "int4", "int4").build()],
    };

    let type_override = ColumnToFieldOptions {
        override_name: None,
        override_type: Some("String".to_string()),
    };
    let column_override = ColumnToFieldOptions {
        override_name: None,
        override_type: Some("rust_decimal::Decimal".to_string()),
    };

    let mut table_to_struct_options = TableToStructOptions::default();

    table_to_struct_options.add_type_override("int4", type_override);
    table_to_struct_options.add_column_override("price", column_override);

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);

    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            struct_name: "Product".to_string(),
            table_name: Some("products".to_string()),
            fields: vec![RustDbSetField {
                field_name: "price".to_string(),
                field_type: "rust_decimal::Decimal".to_string()
            }]
        }
    )
}
