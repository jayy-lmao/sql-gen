use crate::core::{
    intepreters::get_file_structs::get_file_structs,
    models::rust::{dbset_attribute_with_table_name, RustDbSetStruct},
};

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
        name: "Customer".to_string(),
        ..Default::default()
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
        name: "Customer".to_string(),
        attributes: vec![dbset_attribute_with_table_name("users")],
        ..Default::default()
    }];

    pretty_assertions::assert_eq!(res, expected);
}
