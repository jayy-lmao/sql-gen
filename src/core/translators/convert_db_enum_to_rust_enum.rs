use crate::core::models::{
    db::CustomEnum,
    rust::{
        enum_typename_attribute, enum_variant_rename_attribute, RustDbSetEnum, RustDbSetEnumVariant,
    },
};
use convert_case::{Case, Casing};
use pluralizer::pluralize;

use super::models::CodegenOptions;

pub fn convert_db_enums_to_rust_enum(
    custom_enums: Vec<CustomEnum>,
    options: &CodegenOptions,
) -> Vec<RustDbSetEnum> {
    custom_enums
        .iter()
        .map(|e| convert_db_enum_to_rust_enum(e, options))
        .collect()
}

pub fn convert_db_enum_to_rust_enum(
    custom_enum: &CustomEnum,
    options: &CodegenOptions,
) -> RustDbSetEnum {
    let name = if let Some(parent_table_name) = &custom_enum.child_of_table {
        let table_name_singular = pluralize(parent_table_name, 1, false);
        format!(
            "{}{}",
            table_name_singular.to_case(Case::Pascal),
            custom_enum.name.to_case(Case::Pascal),
        )
    } else {
        custom_enum.name.to_case(Case::Pascal)
    };

    RustDbSetEnum {
        name,
        attributes: if let Some(type_name) = &custom_enum.type_name {
            vec![enum_typename_attribute(type_name)]
        } else {
            vec![]
        },
        variants: custom_enum
            .variants
            .iter()
            .map(|v| RustDbSetEnumVariant {
                name: v.name.to_case(Case::Pascal),
                attributes: vec![enum_variant_rename_attribute(&v.name)],
            })
            .collect(),
        derives: if options.enum_derives.is_empty() {
            vec!["sqlx::Type".to_string()]
        } else {
            options.enum_derives.clone()
        },
        comment: custom_enum.comments.clone(),
    }
}
