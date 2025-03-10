use crate::core::{
    models::rust::{RustDbSetEnum, RustDbSetEnumVariant, RustDbSetField, RustDbSetStruct},
    writers::fs_writer::{DbSetsFsWriter, DbSetsFsWriterFile, FsWriterContent},
};
use pretty_assertions::assert_eq;

#[tokio::test]
async fn should_store_enums_and_structs() {
    let mut fs_writer = DbSetsFsWriter::default();

    let my_enum = RustDbSetEnum {
        name: "Mood".to_string(),
        variants: vec![
            RustDbSetEnumVariant {
                name: "Happy".to_string(),
                ..Default::default()
            },
            RustDbSetEnumVariant {
                name: "Sadge".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let my_struct = RustDbSetStruct {
        name: "Product".to_string(),
        fields: vec![RustDbSetField {
            field_name: "id".to_string(),
            field_type: "Uuid".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    fs_writer.add_enum(my_enum.clone());
    fs_writer.add_struct(my_struct.clone());

    assert_eq!(
        fs_writer,
        DbSetsFsWriter {
            files: vec![
                DbSetsFsWriterFile {
                    name: String::from("mood"),
                    content: FsWriterContent::Enum(my_enum)
                },
                DbSetsFsWriterFile {
                    name: String::from("product"),
                    content: FsWriterContent::Struct(my_struct)
                },
            ]
        }
    )
}
