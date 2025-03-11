use super::convert_db_enum_to_rust_enum::convert_db_enum_to_rust_enum;
use crate::{core::models::db::CustomEnum, Mode};
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct TableToStructOptions {
    pub override_name: Option<String>,
    pub struct_derives: Vec<String>,
    pub column_overrides: HashMap<String, ColumnToFieldOptions>,
    pub type_overrides: HashMap<String, ColumnToFieldOptions>,
}
impl TableToStructOptions {
    pub fn add_column_override(mut self, column_name: &str, options: ColumnToFieldOptions) -> Self {
        self.column_overrides
            .insert(column_name.to_string(), options);
        self
    }
    pub fn add_type_override(mut self, type_name: &str, options: ColumnToFieldOptions) -> Self {
        self.type_overrides.insert(type_name.to_string(), options);
        self
    }

    pub fn set_model_derives(mut self, mode: Mode, derives: &Option<Vec<String>>) -> Self {
        let mode_default = match mode {
            Mode::Sqlx => "sqlx::FromRow",
            Mode::Dbset => "sqlx::DbSet",
        }
        .to_string();

        self.struct_derives = derives
            .clone()
            .unwrap_or_else(|| vec!["Debug".to_string(), "Clone".to_string(), mode_default]);
        self
    }
    pub fn add_enums(mut self, enums: &[CustomEnum]) -> Self {
        for custom_enum in enums.iter() {
            // Skip if already manually set by user
            if self.type_overrides.contains_key(&custom_enum.name) {
                continue;
            }

            let rust_enum = convert_db_enum_to_rust_enum(custom_enum);

            self.type_overrides.insert(
                custom_enum.name.clone(),
                ColumnToFieldOptions {
                    override_name: None,
                    override_type: Some(rust_enum.name),
                },
            );
        }
        self
    }
}

#[derive(Default, Clone, Debug)]
pub struct ColumnToFieldOptions {
    pub override_name: Option<String>,
    pub override_type: Option<String>,
}
