use super::helpers::pretty_print_tokenstream;
use crate::core::{
    models::rust::{
        dbset_attribute_with_table_name, key_attribute, RustDbSetAttribute, RustDbSetAttributeArg,
        RustDbSetField, RustDbSetStruct,
    },
    writers::struct_writer::write_struct_to_string,
};
use std::str::FromStr;

fn tokenstream_from_string(input: &str) -> Result<proc_macro2::TokenStream, String> {
    proc_macro2::TokenStream::from_str(input)
        .map_err(|err| syn::Error::new(proc_macro2::Span::call_site(), err).to_string())
}

fn format_rust_content_string(input: &str) -> String {
    pretty_print_tokenstream(tokenstream_from_string(input).expect("Could not parse"))
}

#[test]
fn should_write_empty_struct_to_string() {
    let content = write_struct_to_string(RustDbSetStruct {
        struct_name: "Customer".to_string(),
        ..Default::default()
    });
    assert_eq!(content.trim(), "pub struct Customer {}")
}

#[test]
fn should_write_empty_struct_with_comments_to_string() {
    let content = write_struct_to_string(RustDbSetStruct {
        struct_name: "Customer".to_string(),
        comment: Some("A customer table".to_string()),
        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            r#"
            /// A customer table
            pub struct Customer {}
            "#
        )
    )
}

#[test]
fn should_write_struct_with_attributes_to_string() {
    let content = write_struct_to_string(RustDbSetStruct {
        struct_name: "Customer".to_string(),
        attributes: vec![RustDbSetAttribute {
            attribute_name: "dbset".to_string(),
            attribute_args: vec![RustDbSetAttributeArg {
                name: "table_name".to_string(),
                value: Some("customers".to_string()),
            }],
        }],
        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            r#"
                #[dbset(table_name = "customers")]
                pub struct Customer {}
            "#
        )
    )
}

#[test]
fn should_write_struct_with_derives_to_string() {
    let content = write_struct_to_string(RustDbSetStruct {
        struct_name: "Customer".to_string(),
        derives: vec!["Debug".to_string()],
        ..Default::default()
    });
    assert_eq!(content.trim(), "#[derive(Debug)]\npub struct Customer {}")
}

#[test]
fn should_write_struct_with_attributes_and_derives_to_string() {
    let content = write_struct_to_string(RustDbSetStruct {
        struct_name: "Customer".to_string(),
        derives: vec!["Debug".to_string(), "DbSet".to_string()],
        attributes: vec![dbset_attribute_with_table_name("users")],

        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            r#"
                #[derive(Debug, DbSet)]
                #[dbset(table_name = "users")]
                pub struct Customer {}
            "#
        )
    )
}

#[test]
fn should_write_basic_struct_to_string() {
    let content = write_struct_to_string(RustDbSetStruct {
        struct_name: "Product".to_string(),
        fields: vec![
            RustDbSetField {
                field_name: "title".to_string(),
                field_type: "String".to_string(),
                ..Default::default()
            },
            RustDbSetField {
                field_name: "description".to_string(),
                field_type: "String".to_string(),
                is_optional: true,
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            "pub struct Product {
            title: String,
            description: Option<String>,
        }"
        )
    )
}

#[test]
fn should_write_struct_with_field_attributes_to_string() {
    let content = write_struct_to_string(RustDbSetStruct {
        struct_name: "Product".to_string(),
        fields: vec![RustDbSetField {
            field_name: "id".to_string(),
            field_type: "Uuid".to_string(),
            is_optional: false,
            attributes: vec![key_attribute()],
            ..Default::default()
        }],
        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            "pub struct Product {
            #[key]
            id: Uuid,
        }"
        )
    )
}
