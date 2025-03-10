use crate::core::{
    models::{
        db::{CustomEnum, CustomEnumVariant},
        rust::{enum_typename_attribute, enum_variant_rename_attribute},
    },
    translators::convert_db_enum_to_rust_enum::convert_db_enum_to_rust_enum,
};

#[test]
fn test_empty_variants() {
    let custom_enum = CustomEnum {
        name: "examples".to_string(),
        schema: "public".to_string(),
        variants: vec![],
        comments: Some("Example comment".to_string()),
    };

    let rust_enum = convert_db_enum_to_rust_enum(&custom_enum);
    assert_eq!(rust_enum.name, "Examples");
    assert!(rust_enum.variants.is_empty());
    assert_eq!(rust_enum.derives, vec!["sqlx::Type".to_string()]);
    let expected_attr = enum_typename_attribute(&custom_enum.name);
    assert_eq!(rust_enum.attributes, vec![expected_attr]);
    assert_eq!(rust_enum.comment, Some("Example comment".to_string()));
}

#[test]
fn test_multiple_variants() {
    let custom_enum = CustomEnum {
        name: "color".to_string(),
        schema: "public".to_string(),
        variants: vec![
            CustomEnumVariant {
                name: "red".to_string(),
            },
            CustomEnumVariant {
                name: "green".to_string(),
            },
            CustomEnumVariant {
                name: "blue".to_string(),
            },
        ],
        comments: None,
    };

    let rust_enum = convert_db_enum_to_rust_enum(&custom_enum);

    assert_eq!(rust_enum.name, "Color");

    let expected_variant_names = vec!["Red", "Green", "Blue"];
    let variant_names: Vec<_> = rust_enum.variants.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(variant_names, expected_variant_names);

    for (i, variant) in rust_enum.variants.iter().enumerate() {
        let expected_variant_attr = enum_variant_rename_attribute(&custom_enum.variants[i].name);
        assert_eq!(variant.attributes, vec![expected_variant_attr]);
    }

    assert_eq!(rust_enum.derives, vec!["sqlx::Type".to_string()]);
    let expected_typename_attr = enum_typename_attribute(&custom_enum.name);
    assert_eq!(rust_enum.attributes, vec![expected_typename_attr]);

    assert_eq!(rust_enum.comment, None);
}

#[test]
fn test_pascal_case_conversion() {
    let custom_enum = CustomEnum {
        name: "my_custom_enum".to_string(),
        schema: "public".to_string(),
        variants: vec![
            CustomEnumVariant {
                name: "first_variant".to_string(),
            },
            CustomEnumVariant {
                name: "second_variant".to_string(),
            },
        ],
        comments: Some("Test comment".to_string()),
    };

    let rust_enum = convert_db_enum_to_rust_enum(&custom_enum);

    assert_eq!(rust_enum.name, "MyCustomEnum");

    let expected_variant_names = vec!["FirstVariant", "SecondVariant"];
    let variant_names: Vec<_> = rust_enum.variants.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(variant_names, expected_variant_names);

    assert_eq!(rust_enum.comment, Some("Test comment".to_string()));
}
