use core::{
    translators::{self, models::TableToStructOptions},
    writers::fs_writer::DbSetsFsWriter,
};

use clap::{Parser, ValueEnum};
use sqlx::postgres::PgPoolOptions;

pub mod core;
pub mod postgres;

#[derive(Parser, Debug)]
#[command(
    name = "db-codegen",
    version = "0.1.0",
    about = "Database code generation CLI"
)]
struct Cli {
    /// Database URL.
    #[arg(long)]
    db_url: String,

    /// Schema names (can accept many).
    #[arg(
        long,
        value_name = "SCHEMAS",
        default_value = "public",
        value_delimiter = ','
    )]
    schemas: Vec<String>,

    /// Table names (can accept many).
    #[arg(long, value_name = "INCLUDE_TABLES", value_delimiter = ',')]
    include_tables: Option<Vec<String>>,

    /// Model derives to add (can be used multiple times).
    #[arg(
        long = "model-derive",
        value_name = "MODEL_DERIVE",
        value_delimiter = ','
    )]
    model_derives: Vec<String>,

    // /// Mode of code generation: either sqlx or dbset.
    #[arg(long, value_enum, default_value_t = Mode::Sqlx)]
    mode: Mode,
    /// Type overrides (can be used multiple times).
    #[arg(
        long = "type-override",
        value_name = "TYPE_OVERRIDE",
        value_delimiter = ','
    )]
    type_overrides: Vec<String>,

    /// Field overrides (can be used multiple times).
    #[arg(
        long = "field-override",
        value_name = "FIELD_OVERRIDE",
        value_delimiter = ','
    )]
    field_overrides: Vec<String>,

    /// Output folder.
    #[arg(long, default_value = "src/models/")]
    output_folder: String,

    /// Overwrite files flag (if set, files will be overwritten).
    #[arg(long, action = clap::ArgAction::SetTrue)]
    overwrite_files: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
enum Mode {
    Sqlx,
    Dbset,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let pool = PgPoolOptions::new()
        .connect(&args.db_url)
        .await
        .expect("Failed to connect to postgres container");

    let enums = postgres::queries::get_enums::get_postgres_enums(&pool)
        .await
        .unwrap();

    let tables =
        postgres::queries::get_tables::get_tables(&pool, &args.schemas, &args.include_tables)
            .await
            .unwrap();

    let tables_options = TableToStructOptions::default().add_enums(&enums);

    let structs_mapped =
        translators::convert_table_to_struct::convert_tables_to_struct(tables, tables_options);
    let enums_mapped =
        translators::convert_db_enum_to_rust_enum::convert_db_enums_to_rust_enum(enums);

    let mut writer = DbSetsFsWriter::default();

    for rust_struct in structs_mapped {
        writer.add_struct(rust_struct);
    }

    for rust_enum in enums_mapped {
        writer.add_enum(rust_enum);
    }

    println!("[Debug] args {:#?}", args);

    if args.output_folder.as_str() == "-" {
        println!("{}", writer.write_as_one_file())
    }
}
