use super::convert_db_enum_to_rust_enum::convert_db_enum_to_rust_enum;
use crate::{core::models::db::CustomEnum, Mode};
use std::collections::HashMap;

type TableName = String;
type ColumnName = String;
type TypeName = String;

#[derive(Default, Debug, Clone)]
pub struct CodegenOptions {
    pub mode: Mode,
    pub override_name: HashMap<TableName, TableName>,
    pub struct_derives: Vec<String>,
    pub enum_derives: Vec<String>,
    pub table_column_overrides: HashMap<(TableName, ColumnName), ColumnToFieldOptions>,
    pub column_overrides: HashMap<ColumnName, ColumnToFieldOptions>,
    pub type_overrides: HashMap<TypeName, ColumnToFieldOptions>,
}

impl CodegenOptions {
    pub fn add_table_column_override(
        &mut self,
        table_name: &str,
        column_name: &str,
        options: ColumnToFieldOptions,
    ) {
        self.table_column_overrides
            .insert((table_name.to_string(), column_name.to_string()), options);
    }
    pub fn add_column_override(&mut self, column_name: &str, options: ColumnToFieldOptions) {
        self.column_overrides
            .insert(column_name.to_string(), options);
    }
    pub fn add_type_override(&mut self, type_name: &str, options: ColumnToFieldOptions) {
        self.type_overrides.insert(type_name.to_string(), options);
    }

    pub fn set_type_overrides_from_arg(&mut self, type_overrides: &[String]) {
        for t_override in type_overrides {
            let mut parts = t_override.split("=");
            if let (Some(qualifier), Some(override_type)) = (parts.next(), parts.next()) {
                if qualifier.contains(".") {
                    println!("Warning: no support for <schema>.<type> syntax just yet")
                } else {
                    self.add_type_override(
                        qualifier,
                        ColumnToFieldOptions {
                            override_name: None,
                            override_type: Some(override_type.to_string()),
                            mode: self.mode,
                        },
                    );
                };
            }
        }
    }

    pub fn set_table_column_overrides_from_arg(&mut self, table_overrides: &[String]) {
        for t_override in table_overrides {
            let mut parts = t_override.split("=");
            if let (Some(qualifier), Some(override_type)) = (parts.next(), parts.next()) {
                if qualifier.contains(".") {
                    let mut parts = qualifier.split(".");
                    if let (Some(table_name), Some(column_name)) = (parts.next(), parts.next()) {
                        self.add_table_column_override(
                            table_name,
                            column_name,
                            ColumnToFieldOptions {
                                override_name: None,
                                mode: self.mode,
                                override_type: Some(override_type.to_string()),
                            },
                        );
                    }
                } else {
                    self.add_column_override(
                        qualifier,
                        ColumnToFieldOptions {
                            override_name: None,
                            mode: self.mode,
                            override_type: Some(override_type.to_string()),
                        },
                    );
                };
            }
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn set_model_derives(&mut self, derives: &Option<Vec<String>>) {
        let mode_default = match self.mode {
            Mode::Sqlx => "sqlx::FromRow",
            Mode::Dbset => "db_set_macros::DbSet",
        }
        .to_string();

        self.struct_derives = derives
            .clone()
            .unwrap_or_else(|| vec!["Debug".to_string(), "Clone".to_string(), mode_default]);
    }

    pub fn set_enum_derives(&mut self, derives: &Option<Vec<String>>) {
        self.enum_derives = derives.clone().unwrap_or_else(|| {
            vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
                "sqlx::FromRow".to_string(),
            ]
        });
    }

    pub fn add_enums(&mut self, enums: &[CustomEnum]) {
        for custom_enum in enums.iter() {
            // Skip if already manually set by user
            if self.type_overrides.contains_key(&custom_enum.name) {
                continue;
            }

            let rust_enum = convert_db_enum_to_rust_enum(custom_enum, &self);

            self.type_overrides.insert(
                custom_enum.name.clone(),
                ColumnToFieldOptions {
                    override_name: None,
                    override_type: Some(rust_enum.name),
                    mode: self.mode,
                },
            );
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct ColumnToFieldOptions {
    pub override_name: Option<String>,
    pub override_type: Option<String>,
    pub mode: Mode,
}
