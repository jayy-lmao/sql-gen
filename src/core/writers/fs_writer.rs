use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
};

use crate::core::models::rust::{RustDbSetEnum, RustDbSetStruct};
use convert_case::{Case, Casing};

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
        self.struct_files.sort_by(|a, b| a.name.cmp(&b.name));
        self
    }

    pub fn write_to_string(&self) -> String {
        let mut outputs = vec![];

        for enum_tokens in &self.enum_files {
            let is_used = self.struct_files.iter().any(|s| {
                s.content
                    .fields
                    .iter()
                    .any(|f| f.field_type == enum_tokens.content.name)
            });

            if is_used {
                outputs.push(enum_tokens.content.to_tokens());
            }
        }

        for struct_tokens in &self.struct_files {
            outputs.push(struct_tokens.content.to_tokens());
        }

        outputs.into_iter().fold(String::new(), |acc, output| {
            format!("{}\n{}", acc, pretty_print_tokenstream(output))
        })
    }
    pub fn write_to_std_out(&self) {
        println!("{}", self.write_to_string())
    }

    pub fn write_to_file(&self, filename: &str) {
        fs::write(filename, self.write_to_string()).unwrap();
    }

    fn detect_dependencies(
        &self,
        struct_or_enum_name: &str,
        content: &str,
        known_types: &HashSet<String>,
    ) -> Vec<String> {
        known_types
            .iter()
            .filter(|&t| content.contains(&format!(": {t}")) && t != struct_or_enum_name) // Check if struct/enum name appears in the content
            .cloned()
            .collect()
    }
    pub fn write_db_sets_to_fs(&self, output_dir: &str) {
        // Create the directory (including parent directories if needed)
        fs::create_dir_all(output_dir).unwrap();

        let mut mod_contents = String::new();
        let all_types: HashSet<String> = self
            .struct_files
            .iter()
            .map(|s| s.content.name.clone())
            .chain(self.enum_files.iter().map(|e| e.content.name.clone()))
            .collect();

        // Write struct files
        for struct_file in &self.struct_files {
            let file_content = struct_file.content.to_string();
            let dependencies =
                self.detect_dependencies(&struct_file.content.name, &file_content, &all_types);
            let mut content_with_imports = String::new();
            for dep in &dependencies {
                content_with_imports.push_str(&format!("use super::{};\n", dep));
            }
            if !dependencies.is_empty() {
                content_with_imports.push('\n');
            }
            content_with_imports.push_str(file_content.as_str());
            let file_path = format!("{}/{}.rs", output_dir, struct_file.name);
            let mut file = File::create(&file_path).unwrap();

            file.write_all(&content_with_imports.into_bytes()).unwrap();

            // Add to mod file
            mod_contents.push_str(&format!("pub mod {};\n", struct_file.name));
            mod_contents.push_str(&format!("pub use {}::*;\n", struct_file.name));
        }

        // Write enum files
        for enum_file in &self.enum_files {
            let file_content = enum_file.content.to_string();
            let dependencies =
                self.detect_dependencies(&enum_file.content.name, &file_content, &all_types);
            let mut content_with_imports = String::new();
            for dep in &dependencies {
                content_with_imports.push_str(&format!("use super::{};\n", dep));
            }
            if !dependencies.is_empty() {
                content_with_imports.push('\n');
            }
            content_with_imports.push_str(file_content.as_str());

            let file_path = format!("{}/{}.rs", output_dir, enum_file.name);
            let mut file = File::create(&file_path).unwrap();
            file.write_all(&content_with_imports.into_bytes()).unwrap();

            // Add to mod file
            mod_contents.push_str(&format!("pub mod {};\n", enum_file.name));
            mod_contents.push_str(&format!("pub use {}::*;\n", enum_file.name));
        }

        // Write the mod.rs file
        let mod_file_path = format!("{}/mod.rs", output_dir);
        let mut mod_file = File::create(&mod_file_path).unwrap();
        mod_file.write_all(mod_contents.as_bytes()).unwrap();
    }
}
