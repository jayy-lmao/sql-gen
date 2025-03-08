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

pub fn write_enum_to_string(rust_enum: RustDbSetEnum) -> String {
    let struct_name = format_ident!("{}", rust_enum.enum_name);
    let variants = get_enum_variants_tokens(&rust_enum);
    let attributes = get_attributes_for_enum(&rust_enum);
    let derives = get_derives_for_enum(&rust_enum);

    let comment = if let Some(comment) = &rust_enum.comment {
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
        pub enum #struct_name {
            #(#variants),*
        }
    };

    pretty_print_tokenstream(struct_ast)
}
