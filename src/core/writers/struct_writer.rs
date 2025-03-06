use super::helpers::pretty_print_tokenstream;
use crate::core::models::rust::{RustDbSetAttribute, RustDbSetField, RustDbSetStruct};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn get_derives(rust_struct: &RustDbSetStruct) -> TokenStream {
    if rust_struct.derives.is_empty() {
        return quote! {};
    }

    let struct_derives = rust_struct
        .derives
        .iter()
        .map(|derive| {
            let derive_ident = format_ident!("{}", derive);
            quote! {
                #derive_ident
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #[derive(#(#struct_derives),*)]
    }
}

fn get_attributes(attributes: &[RustDbSetAttribute]) -> TokenStream {
    if attributes.is_empty() {
        return quote! {};
    }

    let struct_fields_tokens = attributes
        .iter()
        .map(|attribute| {
            let attribute_name = format_ident!("{}", attribute.attribute_name);

            if attribute.attribute_args.is_empty() {
                return quote! {
                    #[#attribute_name]
                };
            }

            let attribute_args = attribute.attribute_args.iter().map(|a| {
                let arg_name = format_ident!("{}", a.name);
                if let Some(arg_value) = &a.value {
                    quote! { #arg_name = #arg_value }
                } else {
                    quote! { #arg_name }
                }
            });

            quote! {
                #[#attribute_name(#(#attribute_args),*)]
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#struct_fields_tokens),*
    }
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
    let struct_name = format_ident!("{}", rust_struct.struct_name);
    let fields = get_struct_fields_tokens(&rust_struct);
    let attributes = get_attributes_for_struct(&rust_struct);
    let derives = get_derives(&rust_struct);

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
