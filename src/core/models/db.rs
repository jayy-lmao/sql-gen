use crate::mysql::queries::convert_type::convert_data_type;

#[derive(Debug, PartialEq, Eq)]
pub struct TableColumn {
    pub column_name: String,
    pub column_comment: Option<String>,
    pub udt_name: String,
    pub data_type: String,
    pub recommended_rust_type: Option<String>,
    pub is_nullable: bool,
    pub array_depth: i16,
    pub is_unique: bool,
    pub is_primary_key: bool,
    pub foreign_key_table: Option<String>,
    pub foreign_key_id: Option<String>,
    pub is_auto_populated: bool,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Table {
    pub table_name: String,
    pub table_comment: Option<String>,
    pub table_schema: Option<String>,
    pub columns: Vec<TableColumn>,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct CustomEnumVariant {
    pub name: String,
}
#[derive(Debug, PartialEq, Eq, Default)]
pub struct CustomEnum {
    pub name: String,
    // Only for Postgres
    pub type_name: Option<String>,
    /// Only for MySQL
    pub child_of_table: Option<String>,
    pub schema: Option<String>,
    pub variants: Vec<CustomEnumVariant>,
    pub comments: Option<String>,
}

pub struct TableColumnBuilder {
    column_name: String,
    column_comment: Option<String>,
    recommended_rust_type: Option<String>,
    udt_name: String,
    data_type: String,
    is_nullable: bool,
    array_depth: i16,
    is_unique: bool,
    is_primary_key: bool,
    foreign_key_table: Option<String>,
    foreign_key_id: Option<String>,
    is_auto_populated: bool,
}

impl TableColumnBuilder {
    pub fn new(
        column_name: impl ToString,
        udt_name: impl ToString,
        data_type: impl ToString,
        recommended_rust_type: Option<String>,
    ) -> Self {
        Self {
            column_name: column_name.to_string(),
            column_comment: None,
            udt_name: udt_name.to_string(),
            data_type: data_type.to_string(),
            is_nullable: false,
            is_unique: false,
            is_primary_key: false,
            foreign_key_table: None,
            foreign_key_id: None,
            is_auto_populated: false,
            array_depth: 0,
            recommended_rust_type,
        }
    }

    pub fn is_nullable(mut self) -> Self {
        self.is_nullable = true;
        self
    }

    pub fn array_depth(mut self, depth: i16) -> Self {
        self.array_depth = depth;
        self
    }

    pub fn is_auto_populated(mut self) -> Self {
        self.is_auto_populated = true;
        self
    }

    pub fn is_unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    pub fn is_primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self
    }

    pub fn add_column_comment(mut self, column_comment: impl Into<String>) -> Self {
        self.column_comment = Some(column_comment.into());
        self
    }

    pub fn foreign_key_table(mut self, foreign_key_table: impl ToString) -> Self {
        self.foreign_key_table = Some(foreign_key_table.to_string());
        self
    }
    pub fn foreign_key_id(mut self, foreign_key_id: impl ToString) -> Self {
        self.foreign_key_id = Some(foreign_key_id.to_string());
        self
    }

    pub fn build(self) -> TableColumn {
        TableColumn {
            column_name: self.column_name,
            recommended_rust_type: self.recommended_rust_type,
            udt_name: self.udt_name,
            data_type: self.data_type,
            is_nullable: self.is_nullable,
            array_depth: self.array_depth,
            is_unique: self.is_unique,
            is_primary_key: self.is_primary_key,
            foreign_key_table: self.foreign_key_table,
            foreign_key_id: self.foreign_key_id,
            column_comment: self.column_comment,
            is_auto_populated: self.is_auto_populated,
        }
    }
}
