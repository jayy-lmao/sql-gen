use crate::core::models::{
    db::CustomEnum,
    rust::{
        enum_typename_attribute, enum_variant_rename_attribute, RustDbSetEnum, RustDbSetEnumVariant,
    },
};
use convert_case::{Case, Casing};

pub fn convert_db_enum_to_rust_enum(custom_enum: &CustomEnum) -> RustDbSetEnum {
    RustDbSetEnum {
        enum_name: custom_enum.name.to_case(Case::Pascal),
        variants: custom_enum
            .variants
            .iter()
            .map(|v| RustDbSetEnumVariant {
                name: v.name.to_case(Case::Pascal),
                attributes: vec![enum_variant_rename_attribute(&v.name)],
            })
            .collect(),
        derives: vec!["sqlx::Type".to_string()],
        attributes: vec![enum_typename_attribute(&custom_enum.name)],
        comment: None,
    }
}
