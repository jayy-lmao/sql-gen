use std::fmt::Display;

use super::helpers::{get_attributes, get_derives, pretty_print_tokenstream};
use crate::core::models::rust::{RustDbSetEnum, RustDbSetEnumVariant};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn get_derives_for_enum(rust_enum: &RustDbSetEnum) -> TokenStream {
    get_derives(&rust_enum.derives)
}

fn get_attributes_for_enum(rust_enum: &RustDbSetEnum) -> TokenStream {
    get_attributes(&rust_enum.attributes)
}
fn get_attributes_for_variant(rust_variant: &RustDbSetEnumVariant) -> TokenStream {
    get_attributes(&rust_variant.attributes)
}

fn get_enum_variants_tokens(rust_enum: &RustDbSetEnum) -> Vec<TokenStream> {
    let mut enum_variants_tokens = vec![];

    for variant in rust_enum.variants.iter() {
        let variant_name = format_ident!("{}", variant.name);

        let attributes = get_attributes_for_variant(variant);
        let variant_ast = quote! {
                #attributes
                #variant_name
        };

        enum_variants_tokens.push(variant_ast);
    }
    enum_variants_tokens
}

impl RustDbSetEnum {
    pub fn to_tokens(&self) -> TokenStream {
        let struct_name = format_ident!("{}", self.name);
        let variants = get_enum_variants_tokens(self);
        let attributes = get_attributes_for_enum(self);
        let derives = get_derives_for_enum(self);

        let comment = if let Some(comment) = &self.comment {
            let comment = format!(" {}", comment);
            quote! {
               #[doc = #comment]
            }
        } else {
            quote! {}
        };

        let enum_tokens = quote! {
            #comment
            #derives
            #attributes
            pub enum #struct_name {
                #(#variants),*
            }
        };

        enum_tokens
    }
}

impl Display for RustDbSetEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", pretty_print_tokenstream(self.to_tokens()))
    }
}
