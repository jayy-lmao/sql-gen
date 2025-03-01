use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::File;

use crate::core::models::rust::RustDbSetField;

use super::models::rust::RustDbSetStruct;

fn tokenstream_from_string(input: &str) -> Result<proc_macro2::TokenStream, String> {
    proc_macro2::TokenStream::from_str(input)
        .map_err(|err| syn::Error::new(proc_macro2::Span::call_site(), err).to_string())
}

fn pretty_print_tokenstream(ts: proc_macro2::TokenStream) -> String {
    match syn::parse2::<File>(ts.clone()) {
        Ok(file) => prettyplease::unparse(&file).to_string(),
        Err(err) => format!("Failed to parse TokenStream: {err}. Stream was {ts}"),
    }
}
fn format_rust_content_string(input: &str) -> String {
    pretty_print_tokenstream(tokenstream_from_string(input).expect("Could not parse"))
}

fn get_struct_fields_tokens(rust_struct: &RustDbSetStruct) -> Vec<TokenStream> {
    let mut struct_fields_tokens = vec![];

    for field in rust_struct.fields.iter() {
        let field_name = format_ident!("{}", field.field_name);
        let field_type = format_ident!("{}", field.field_type);

        let field_ast = if field.is_optional {
            quote! { #field_name: Option<#field_type> }
        } else {
            quote! {
                #field_name: #field_type
            }
        };

        struct_fields_tokens.push(field_ast);
    }
    struct_fields_tokens
}

// TODO:
// - [ ] Enum imports
// - [ ] Maybe custom type imports like rust_decimal / uuid?
pub fn write_struct_to_string(rust_struct: RustDbSetStruct) -> String {
    let struct_name = format_ident!("{}", rust_struct.struct_name);
    let fields = get_struct_fields_tokens(&rust_struct);
    let struct_ast = quote! {
        pub struct #struct_name {
            #(#fields),*
        }
    };

    pretty_print_tokenstream(struct_ast)
}

#[test]
fn should_write_empty_struct_to_string() {
    let content = write_struct_to_string(RustDbSetStruct {
        struct_name: "Customer".to_string(),
        table_name: Some("users".to_string()),
        fields: vec![],
    });
    assert_eq!(content.trim(), "pub struct Customer {}")
}

#[test]
fn should_write_basic_struct_to_string() {
    let content = write_struct_to_string(RustDbSetStruct {
        struct_name: "Product".to_string(),
        table_name: Some("products".to_string()),
        fields: vec![
            RustDbSetField {
                field_name: "title".to_string(),
                field_type: "String".to_string(),
                is_optional: false,
            },
            RustDbSetField {
                field_name: "description".to_string(),
                field_type: "String".to_string(),
                is_optional: true,
            },
        ],
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
