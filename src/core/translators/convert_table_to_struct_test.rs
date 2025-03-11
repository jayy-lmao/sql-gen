use crate::core::{
    models::{
        db::{CustomEnum, CustomEnumVariant, Table, TableColumnBuilder},
        rust::{
            auto_attribute, dbset_attribute_with_table_name, key_attribute, unique_attribute,
            RustDbSetField, RustDbSetStruct,
        },
    },
    translators::{
        convert_table_to_struct::convert_table_to_struct,
        models::{ColumnToFieldOptions, TableToStructOptions},
    },
};
use pretty_assertions::assert_eq;

#[test]
fn should_convert_empty_table_to_struct() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        ..Default::default()
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Product".to_string(),
            attributes: vec![dbset_attribute_with_table_name("products")],
            ..Default::default()
        }
    )
}

#[test]
fn should_convert_empty_table_to_struct_2() {
    let table = Table {
        table_name: "inventories".to_string(),
        table_schema: "public".to_string(),
        ..Default::default()
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Inventory".to_string(),
            attributes: vec![dbset_attribute_with_table_name("inventories")],
            ..Default::default()
        }
    )
}

#[test]
fn should_convert_table_to_struct_with_override() {
    let table = Table {
        table_name: "users".to_string(),
        table_schema: "public".to_string(),
        ..Default::default()
    };

    let table_to_struct_options = TableToStructOptions {
        override_name: Some("Customer".to_string()),
        ..Default::default()
    };

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);
    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Customer".to_string(),
            attributes: vec![dbset_attribute_with_table_name("users")],
            ..Default::default()
        }
    )
}

#[test]
fn should_convert_table_with_basic_column() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("title", "text", "text").build()],
        ..Default::default()
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Product".to_string(),
            attributes: vec![dbset_attribute_with_table_name("products")],
            fields: vec![RustDbSetField {
                field_name: "title".to_string(),
                field_type: "String".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }
    )
}

#[test]
fn should_convert_table_with_each_column_attribute_type() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![
            TableColumnBuilder::new("id", "uuid", "uuid")
                .is_auto_populated()
                .is_primary_key()
                .build(),
            TableColumnBuilder::new("title", "text", "text")
                .is_unique()
                .build(),
            TableColumnBuilder::new("description", "text", "text")
                .is_nullable()
                .build(),
        ],
        ..Default::default()
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Product".to_string(),
            attributes: vec![dbset_attribute_with_table_name("products")],
            fields: vec![
                RustDbSetField {
                    field_name: "id".to_string(),
                    field_type: "uuid::Uuid".to_string(),
                    attributes: vec![auto_attribute(), key_attribute(),],
                    ..Default::default()
                },
                RustDbSetField {
                    field_name: "title".to_string(),
                    field_type: "String".to_string(),
                    attributes: vec![unique_attribute()],
                    ..Default::default()
                },
                RustDbSetField {
                    field_name: "description".to_string(),
                    field_type: "String".to_string(),
                    is_optional: true,
                    attributes: vec![],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    )
}

#[test]
fn should_convert_table_with_optional_column() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("description", "text", "text")
            .is_nullable()
            .build()],
        ..Default::default()
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Product".to_string(),
            attributes: vec![dbset_attribute_with_table_name("products")],
            fields: vec![RustDbSetField {
                field_name: "description".to_string(),
                field_type: "String".to_string(),
                is_optional: true,
                ..Default::default()
            }],
            ..Default::default()
        }
    )
}

#[test]
fn should_convert_table_with_array_column() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("tags", "_text", "ARRAY")
            .is_nullable()
            .array_depth(1)
            .build()],
        ..Default::default()
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Product".to_string(),
            attributes: vec![dbset_attribute_with_table_name("products")],
            fields: vec![RustDbSetField {
                field_name: "tags".to_string(),
                field_type: "String".to_string(),
                is_optional: true,
                array_depth: 1,
                ..Default::default()
            }],
            ..Default::default()
        }
    )
}

#[test]
fn should_convert_table_with_enum_column() {
    let table = Table {
        table_name: "orders".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("order_status", "status", "USER-DEFINED").build()],
        ..Default::default()
    };
    let enums: Vec<CustomEnum> = vec![CustomEnum {
        name: "status".to_string(),
        schema: "public".to_string(),
        variants: vec![
            CustomEnumVariant {
                name: "pending".to_string(),
            },
            CustomEnumVariant {
                name: "shipped".to_string(),
            },
            CustomEnumVariant {
                name: "delivered".to_string(),
            },
        ],
        ..Default::default()
    }];
    let table_to_struct_options = TableToStructOptions::default().add_enums(&enums);

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);

    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Order".to_string(),
            attributes: vec![dbset_attribute_with_table_name("orders")],
            fields: vec![RustDbSetField {
                field_name: "order_status".to_string(),
                field_type: "Status".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }
    )
}

#[test]
fn should_ignore_columns_with_invalid_types() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("title", "badtype", "badtype").build()],
        ..Default::default()
    };
    let rust_struct = convert_table_to_struct(table, TableToStructOptions::default());
    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Product".to_string(),
            attributes: vec![dbset_attribute_with_table_name("products")],
            fields: vec![],
            ..Default::default()
        }
    )
}

#[test]
fn should_convert_table_with_column_type_override() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("id", "i32", "i32").build()],
        ..Default::default()
    };

    let column_override = ColumnToFieldOptions {
        override_name: None,
        override_type: Some("String".to_string()),
    };
    let table_to_struct_options =
        TableToStructOptions::default().add_column_override("id", column_override);

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);

    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Product".to_string(),
            attributes: vec![dbset_attribute_with_table_name("products")],
            fields: vec![RustDbSetField {
                field_name: "id".to_string(),
                field_type: "String".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }
    )
}

#[test]
fn should_convert_table_with_global_type_override() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("id", "int4", "int4").build()],
        ..Default::default()
    };

    let type_override = ColumnToFieldOptions {
        override_name: None,
        override_type: Some("String".to_string()),
    };
    let table_to_struct_options =
        TableToStructOptions::default().add_type_override("int4", type_override);

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);

    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Product".to_string(),
            attributes: vec![dbset_attribute_with_table_name("products")],
            fields: vec![RustDbSetField {
                field_name: "id".to_string(),
                field_type: "String".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }
    )
}

#[test]
fn column_override_takes_preference_over_global_type_override() {
    let table = Table {
        table_name: "products".to_string(),
        table_schema: "public".to_string(),
        columns: vec![TableColumnBuilder::new("price", "int4", "int4").build()],
        ..Default::default()
    };

    let type_override = ColumnToFieldOptions {
        override_name: None,
        override_type: Some("String".to_string()),
    };
    let column_override = ColumnToFieldOptions {
        override_name: None,
        override_type: Some("rust_decimal::Decimal".to_string()),
    };

    let table_to_struct_options = TableToStructOptions::default()
        .add_type_override("int4", type_override)
        .add_column_override("price", column_override);

    let rust_struct = convert_table_to_struct(table, table_to_struct_options);

    assert_eq!(
        rust_struct,
        RustDbSetStruct {
            name: "Product".to_string(),
            attributes: vec![dbset_attribute_with_table_name("products")],
            fields: vec![RustDbSetField {
                field_name: "price".to_string(),
                field_type: "rust_decimal::Decimal".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }
    )
}
