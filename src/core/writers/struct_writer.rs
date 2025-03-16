use std::fmt::Display;

use super::helpers::{get_attributes, get_derives, pretty_print_tokenstream, sanitize_field_name};
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
        let field_name = sanitize_field_name(&field.field_name);
        //let field_type = format_ident!("{}", field.field_type);
        let field_type: syn::Path =
            syn::parse_str(&field.field_type).expect("Failed to parse path");

        let attributes = get_attributes_for_field(field);
        let mut base_type = quote! { #field_type };

        for _ in 0..field.array_depth {
            base_type = quote! { Vec<#base_type> };
        }

        if field.is_optional {
            base_type = quote! { Option<#base_type> };
        }

        let field = quote! {
            #attributes
            #field_name: #base_type
        };

        struct_fields_tokens.push(field);
    }
    struct_fields_tokens
}

impl RustDbSetStruct {
    pub fn to_tokens(&self) -> TokenStream {
        let struct_name = format_ident!("{}", self.name);
        let fields = get_struct_fields_tokens(self);
        let attributes = get_attributes_for_struct(self);
        let derives = get_derives_for_struct(self);

        let comment = if let Some(comment) = &self.comment {
            let comment = format!(" {}", comment);
            quote! {
               #[doc = #comment]
            }
        } else {
            quote! {}
        };

        let struct_tokens = quote! {
            #comment
            #derives
            #attributes
            pub struct #struct_name {
                #(#fields),*
            }
        };
        struct_tokens
    }
}

impl Display for RustDbSetStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pretty_print_tokenstream(self.to_tokens()))
    }
}
