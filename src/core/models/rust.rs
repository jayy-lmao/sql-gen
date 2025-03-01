#[derive(Debug, PartialEq)]
pub struct RustDbSetStruct {
    pub(crate) struct_name: String,
    pub(crate) table_name: Option<String>,
    pub fields: Vec<RustDbSetField>,
}

#[derive(Debug, PartialEq)]
pub struct RustDbSetField {
    pub(crate) field_name: String,
    pub(crate) field_type: String,
}
