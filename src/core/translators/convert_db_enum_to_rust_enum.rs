use crate::core::models::{db::CustomEnum, rust::RustDbSetEnum};
use convert_case::{Case, Casing};

pub fn convert_db_enum_to_rust_enum(custom_enum: &CustomEnum) -> RustDbSetEnum {
    RustDbSetEnum {
        enum_name: custom_enum.name.to_case(Case::Pascal),
        variants: custom_enum
            .variants
            .iter()
            .map(|v| v.to_case(Case::Pascal))
            .collect(),
    }
}
