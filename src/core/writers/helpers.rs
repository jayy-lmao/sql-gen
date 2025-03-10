use crate::core::models::rust::RustDbSetAttribute;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::File;

pub fn pretty_print_tokenstream(ts: proc_macro2::TokenStream) -> String {
    match syn::parse2::<File>(ts.clone()) {
        Ok(file) => prettyplease::unparse(&file).to_string(),
        Err(err) => format!("Failed to parse TokenStream: {err}. Stream was {ts}"),
    }
}

pub fn get_attributes(attributes: &[RustDbSetAttribute]) -> TokenStream {
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
        #(#struct_fields_tokens)*
    }
}

pub fn get_derives(derives: &[String]) -> TokenStream {
    if derives.is_empty() {
        return quote! {};
    }

    let struct_derives = derives
        .iter()
        .map(|derive| syn::parse_str::<syn::Path>(derive).unwrap())
        .collect::<Vec<_>>();
    quote! {
        #[derive(#(#struct_derives),*)]
    }
}
