use super::helpers::{get_attributes, get_derives, pretty_print_tokenstream};
use crate::core::models::rust::{RustDbSetField, RustDbSetStruct};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn get_derives_for_struct(rust_struct: &RustDbSetStruct) -> TokenStream {
    get_derives(&rust_struct.derives)
}

fn get_attributes_for_struct(rust_struct: &RustDbSetStruct) -> TokenStream {
    get_attributes(&rust_struct.attributes)
}

fn get_attributes_for_field(rust_field: &RustDbSetField) -> TokenStream {
    get_attributes(&rust_field.attributes)
}

fn get_struct_fields_tokens(rust_struct: &RustDbSetStruct) -> Vec<TokenStream> {
    let mut struct_fields_tokens = vec![];

    for field in rust_struct.fields.iter() {
        let field_name = format_ident!("{}", field.field_name);
        let field_type = format_ident!("{}", field.field_type);

        let attributes = get_attributes_for_field(field);
        let field_ast = if field.is_optional {
            quote! {
                #attributes
                #field_name: Option<#field_type>
            }
        } else {
            quote! {
                #attributes
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
    let struct_name = format_ident!("{}", rust_struct.name);
    let fields = get_struct_fields_tokens(&rust_struct);
    let attributes = get_attributes_for_struct(&rust_struct);
    let derives = get_derives_for_struct(&rust_struct);

    let comment = if let Some(comment) = &rust_struct.comment {
        let comment = format!(" {}", comment);
        quote! {
           #[doc = #comment]
        }
    } else {
        quote! {}
    };

    let struct_ast = quote! {
        #comment
        #derives
        #attributes
        pub struct #struct_name {
            #(#fields),*
        }
    };

    pretty_print_tokenstream(struct_ast)
}
