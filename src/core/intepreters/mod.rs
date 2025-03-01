use super::models::rust::RustDbSetStruct;

fn extract_table_name(tokens: impl IntoIterator<Item = String>) -> Option<String> {
    let mut iter = tokens.into_iter();

    while let Some(token) = iter.next() {
        if token == "table_name" {
            if let (Some(eq), Some(value)) = (iter.next(), iter.next()) {
                if eq == "=" {
                    return Some(value.replacen("\"", "", 2));
                }
            }
        }
    }
    None
}

pub fn get_file_structs(text: &str) -> Vec<RustDbSetStruct> {
    let parsed = syn::parse_file(text).expect("Failed to parse rust code");

    parsed
        .items
        .into_iter()
        .filter_map(|item| match item {
            syn::Item::Struct(item_struct) => {
                let mut table_name: Option<String> = None;
                item_struct.attrs.into_iter().for_each(|a| {
                    if a.path().is_ident("dbset") {
                        if let syn::Meta::List(meta_list) = a.meta {
                            let meta_strings = meta_list.tokens.into_iter().map(|t| t.to_string());
                            table_name = extract_table_name(meta_strings);
                        }
                    }
                });

                Some(RustDbSetStruct {
                    struct_name: item_struct.ident.to_string(),
                    table_name,
                    fields: vec![],
                })
            }

            _ => None,
        })
        .collect()
}

#[test]
fn should_parse_simple_struct() {
    let res = get_file_structs(
        r#"
    pub struct Customer {
        id: String,
        first_name: String,
        last_name: String
    }
    "#,
    );

    let expected = vec![RustDbSetStruct {
        struct_name: "Customer".to_string(),
        table_name: None,
        fields: vec![],
    }];

    pretty_assertions::assert_eq!(res, expected);
}

#[test]
fn should_parse_struct_with_db_set_macro() {
    let res = get_file_structs(
        r#"
        #[dbset(table_name = "users")]
        pub struct Customer {
        id: String,
        first_name: String,
        last_name: String
    }
    "#,
    );

    let expected = vec![RustDbSetStruct {
        struct_name: "Customer".to_string(),
        table_name: Some("users".to_string()),
        fields: vec![],
    }];

    pretty_assertions::assert_eq!(res, expected);
}
