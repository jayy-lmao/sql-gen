use crate::core::models::rust::{RustDbSetEnum, RustDbSetStruct};
use convert_case::{Case, Casing};
use quote::{format_ident, quote};

use super::helpers::pretty_print_tokenstream;

#[derive(PartialEq, Debug)]
pub struct DbSetsFsWriterStructFile {
    pub name: String,
    pub content: RustDbSetStruct,
}

#[derive(PartialEq, Debug)]
pub struct DbSetsFsWriterEnumFile {
    pub name: String,
    pub content: RustDbSetEnum,
}

#[derive(Default, PartialEq, Debug)]
pub struct DbSetsFsWriter {
    pub enum_files: Vec<DbSetsFsWriterEnumFile>,
    pub struct_files: Vec<DbSetsFsWriterStructFile>,
}

impl DbSetsFsWriter {
    pub fn add_enum(&mut self, rust_enum: RustDbSetEnum) -> &Self {
        self.enum_files.push(DbSetsFsWriterEnumFile {
            name: rust_enum.name.to_case(Case::Snake),
            content: rust_enum,
        });
        self
    }
    pub fn add_struct(&mut self, rust_struct: RustDbSetStruct) -> &Self {
        self.struct_files.push(DbSetsFsWriterStructFile {
            name: rust_struct.name.to_case(Case::Snake),
            content: rust_struct,
        });
        self
    }

    pub fn write_as_one_file(&self) -> String {
        let mut outputs = vec![];

        for enum_tokens in &self.enum_files {
            outputs.push(enum_tokens.content.to_tokens());
        }

        for struct_tokens in &self.struct_files {
            outputs.push(struct_tokens.content.to_tokens());
        }

        outputs.into_iter().fold(String::new(), |acc, output| {
            format!("{}\n{}", acc, pretty_print_tokenstream(output))
        })
    }
}
