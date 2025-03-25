use crate::core::models::rust::{RustDbSetAttribute, RustDbSetAttributeArg, RustDbSetStruct};
use syn::ItemStruct;

pub fn get_file_structs(text: &str) -> Vec<RustDbSetStruct> {
    let parsed = syn::parse_file(text).expect("Failed to parse rust code");
    parsed
        .items
        .into_iter()
        .filter_map(|item| match item {
            syn::Item::Struct(item_struct) => Some(RustDbSetStruct {
                name: item_struct.ident.to_string(),
                attributes: extract_table_attributes(&item_struct),
                ..Default::default()
            }),

            _ => None,
        })
        .collect()
}

fn extract_table_attribute_args(
    tokens: impl IntoIterator<Item = String>,
) -> Vec<RustDbSetAttributeArg> {
    let mut iter = tokens.into_iter();
    let mut attributes_args = vec![];

    while let Some(token) = iter.next() {
        if let (Some(eq), Some(value)) = (iter.next(), iter.next()) {
            if eq == "=" {
                let value_inner = value.replacen("\"", "", 2);
                attributes_args.push(RustDbSetAttributeArg {
                    name: token,
                    value: Some(value_inner),
                })
            }
        } else {
            attributes_args.push(RustDbSetAttributeArg {
                name: token,
                value: None,
            })
        }
    }
    attributes_args
}

fn extract_table_attributes(item_struct: &ItemStruct) -> Vec<RustDbSetAttribute> {
    let mut attributes = vec![];
    item_struct.attrs.iter().for_each(|a| {
        let mut attr_args: Vec<RustDbSetAttributeArg> = vec![];
        let attribute_name = a.path().get_ident().unwrap().to_string();
        if let syn::Meta::List(meta_list) = &a.meta {
            let meta_strings = meta_list.tokens.clone().into_iter().map(|t| t.to_string());
            attr_args = extract_table_attribute_args(meta_strings);
        }
        //}
        attributes.push(RustDbSetAttribute {
            attribute_name,
            attribute_args: attr_args,
        });
    });
    attributes
}
