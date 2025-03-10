use convert_case::{Case, Casing};

use crate::core::models::rust::{RustDbSetEnum, RustDbSetStruct};

#[derive(PartialEq, Debug)]
pub enum FsWriterContent {
    Enum(RustDbSetEnum),
    Struct(RustDbSetStruct),
}

#[derive(PartialEq, Debug)]
pub struct DbSetsFsWriterFile {
    pub name: String,
    pub content: FsWriterContent,
}

#[derive(Default, PartialEq, Debug)]
pub struct DbSetsFsWriter {
    pub files: Vec<DbSetsFsWriterFile>,
}

impl DbSetsFsWriter {
    pub fn add_enum(&mut self, rust_enum: RustDbSetEnum) -> &Self {
        self.files.push(DbSetsFsWriterFile {
            name: rust_enum.name.to_case(Case::Snake),
            content: FsWriterContent::Enum(rust_enum),
        });
        self
    }
    pub fn add_struct(&mut self, rust_struct: RustDbSetStruct) -> &Self {
        self.files.push(DbSetsFsWriterFile {
            name: rust_struct.name.to_case(Case::Snake),
            content: FsWriterContent::Struct(rust_struct),
        });
        self
    }
}
