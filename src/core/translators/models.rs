use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct TableToStructOptions {
    pub override_name: Option<String>,
    pub column_overrides: HashMap<String, ColumnToFieldOptions>,
    pub type_overrides: HashMap<String, ColumnToFieldOptions>,
}
impl TableToStructOptions {
    pub fn add_column_override(&mut self, column_name: &str, options: ColumnToFieldOptions) {
        self.column_overrides
            .insert(column_name.to_string(), options);
    }
    pub fn add_type_override(&mut self, type_name: &str, options: ColumnToFieldOptions) {
        self.type_overrides.insert(type_name.to_string(), options);
    }
}

#[derive(Default, Clone, Debug)]
pub struct ColumnToFieldOptions {
    pub override_name: Option<String>,
    pub override_type: Option<String>,
}
