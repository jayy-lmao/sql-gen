#[derive(Debug, PartialEq, Default)]
pub struct RustDbSetStruct {
    pub derives: Vec<String>,
    pub attributes: Vec<RustDbSetAttribute>,
    pub struct_name: String,
    pub fields: Vec<RustDbSetField>,
    pub comment: Option<String>,
}

#[derive(Debug, PartialEq, Default)]
pub struct RustDbSetEnumVariant {
    pub name: String,
    pub attributes: Vec<RustDbSetAttribute>,
}

#[derive(Debug, PartialEq, Default)]
pub struct RustDbSetEnum {
    pub enum_name: String,
    pub comment: Option<String>,
    pub derives: Vec<String>,
    pub attributes: Vec<RustDbSetAttribute>,
    pub variants: Vec<RustDbSetEnumVariant>,
}

#[derive(Debug, PartialEq, Default)]
pub struct RustDbSetField {
    pub field_name: String,
    pub field_type: String,
    pub is_optional: bool,
    pub attributes: Vec<RustDbSetAttribute>,
    pub comment: Option<String>,
}

#[derive(Debug, PartialEq, Default)]
pub struct RustDbSetAttribute {
    pub attribute_name: String,
    pub attribute_args: Vec<RustDbSetAttributeArg>,
}

pub fn dbset_attribute_with_table_name(table_name: impl Into<String>) -> RustDbSetAttribute {
    RustDbSetAttribute {
        attribute_name: "dbset".to_string(),
        attribute_args: vec![RustDbSetAttributeArg {
            name: "table_name".to_string(),
            value: Some(table_name.into()),
        }],
    }
}

pub fn unique_attribute() -> RustDbSetAttribute {
    RustDbSetAttribute {
        attribute_name: "unique".to_string(),
        attribute_args: vec![],
    }
}

pub fn key_attribute() -> RustDbSetAttribute {
    RustDbSetAttribute {
        attribute_name: "key".to_string(),
        attribute_args: vec![],
    }
}

pub fn enum_typename_attribute(type_name: impl Into<String>) -> RustDbSetAttribute {
    RustDbSetAttribute {
        attribute_name: "sqlx".to_string(),
        attribute_args: vec![RustDbSetAttributeArg {
            name: "type_name".to_string(),
            value: Some(type_name.into()),
        }],
    }
}

pub fn enum_variant_rename_attribute(rename_name: impl Into<String>) -> RustDbSetAttribute {
    RustDbSetAttribute {
        attribute_name: "sqlx".to_string(),
        attribute_args: vec![RustDbSetAttributeArg {
            name: "rename".to_string(),
            value: Some(rename_name.into()),
        }],
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct RustDbSetAttributeArg {
    pub name: String,
    pub value: Option<String>,
}
