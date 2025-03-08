use crate::core::{
    models::rust::{
        dbset_attribute_with_table_name, enum_typename_attribute, enum_variant_rename_attribute,
        key_attribute, RustDbSetAttribute, RustDbSetAttributeArg, RustDbSetEnum,
        RustDbSetEnumVariant,
    },
    writers::{enum_writer::write_enum_to_string, test_helpers::format_rust_content_string},
};

#[test]
fn should_write_empty_enum_to_string() {
    let content = write_enum_to_string(RustDbSetEnum {
        enum_name: "Weather".to_string(),
        ..Default::default()
    });
    assert_eq!(content.trim(), "pub enum Weather {}")
}

#[test]
fn should_write_empty_enum_with_comments_to_string() {
    let content = write_enum_to_string(RustDbSetEnum {
        enum_name: "Weather".to_string(),
        comment: Some("A weather enum".to_string()),
        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            r#"
            /// A weather enum
            pub enum Weather {}
            "#
        )
    )
}

#[test]
fn should_write_enum_with_attributes_to_string() {
    let content = write_enum_to_string(RustDbSetEnum {
        enum_name: "Weather".to_string(),
        attributes: vec![enum_typename_attribute("weather")],
        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            r#"
                #[sqlx(type_name = "weather")]
                pub enum Weather {}
            "#
        )
    )
}

#[test]
fn should_write_enum_with_derives_to_string() {
    let content = write_enum_to_string(RustDbSetEnum {
        enum_name: "Weather".to_string(),
        derives: vec!["Debug".to_string()],
        ..Default::default()
    });
    assert_eq!(content.trim(), "#[derive(Debug)]\npub enum Weather {}")
}

#[test]
fn should_write_enum_with_attributes_and_derives_to_string() {
    let content = write_enum_to_string(RustDbSetEnum {
        enum_name: "Weather".to_string(),
        derives: vec!["Debug".to_string(), "PartialEq".to_string()],
        attributes: vec![enum_typename_attribute("weather")],

        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            r#"
                #[derive(Debug, PartialEq)]
                #[sqlx(type_name = "weather")]
                pub enum Weather {}
            "#
        )
    )
}

#[test]
fn should_write_basic_enum_to_string() {
    let content = write_enum_to_string(RustDbSetEnum {
        enum_name: "Mood".to_string(),
        variants: vec![
            RustDbSetEnumVariant {
                name: "Happy".to_string(),
                ..Default::default()
            },
            RustDbSetEnumVariant {
                name: "Sadge".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            "pub enum Mood {
                Happy,
                Sadge,
        }"
        )
    )
}

#[test]
fn should_write_enum_with_sqlx_fields_to_string() {
    let content = write_enum_to_string(RustDbSetEnum {
        enum_name: "Mood".to_string(),
        attributes: vec![enum_typename_attribute("mood")],
        derives: vec!["sqlx::Type".to_string()],
        variants: vec![
            RustDbSetEnumVariant {
                name: "Happy".to_string(),
                attributes: vec![enum_variant_rename_attribute("happy")],
            },
            RustDbSetEnumVariant {
                name: "Sadge".to_string(),
                attributes: vec![enum_variant_rename_attribute("sadge")],
            },
        ],
        ..Default::default()
    });
    assert_eq!(
        content,
        format_rust_content_string(
            r#"

            #[derive(sqlx::Type)]
            #[sqlx(type_name = "mood")]
            pub enum Mood {
                #[sqlx(rename = "happy")]
                Happy,
                #[sqlx(rename = "sadge")]
                Sadge,
            }

        "#
        )
    )
}
