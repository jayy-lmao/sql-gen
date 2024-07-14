#[derive(Debug, PartialEq, Eq)]
pub struct TableColumn {
    pub(crate) column_name: String,
    pub(crate) udt_name: String,
    pub(crate) data_type: String,
    pub(crate) is_nullable: bool,
    pub(crate) is_unique: bool,
    pub(crate) is_primary_key: bool,
    pub(crate) foreign_key_table: Option<String>,
    pub(crate) foreign_key_id: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Table {
    pub(crate) table_name: String,
    pub(crate) table_schema: String,
    pub(crate) columns: Vec<TableColumn>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CustomEnum {
    pub(crate) name: String,
    pub(crate) schema: String,
    pub(crate) variants: Vec<String>,
}

pub struct TableColumnBuilder {
    column_name: String,
    udt_name: String,
    data_type: String,
    is_nullable: bool,
    is_unique: bool,
    is_primary_key: bool,
    foreign_key_table: Option<String>,
    foreign_key_id: Option<String>,
}

impl TableColumnBuilder {
    pub fn new(
        column_name: impl ToString,
        udt_name: impl ToString,
        data_type: impl ToString,
    ) -> Self {
        Self {
            column_name: column_name.to_string(),
            udt_name: udt_name.to_string(),
            data_type: data_type.to_string(),
            is_nullable: false,
            is_unique: false,
            is_primary_key: false,
            foreign_key_table: None,
            foreign_key_id: None,
        }
    }

    pub fn is_nullable(mut self) -> Self {
        self.is_nullable = true;
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
            udt_name: self.udt_name,
            data_type: self.data_type,
            is_nullable: self.is_nullable,
            is_unique: self.is_unique,
            is_primary_key: self.is_primary_key,
            foreign_key_table: self.foreign_key_table,
            foreign_key_id: self.foreign_key_id,
        }
    }
}
