use core::{
    translators::{self, models::CodegenOptions},
    writers::fs_writer::DbSetsFsWriter,
};

use clap::{Parser, ValueEnum};
use sqlx::{mysql::MySqlPoolOptions, postgres::PgPoolOptions};

pub mod core;
pub mod mysql;
pub mod postgres;
#[cfg(test)]
pub mod tests;

#[derive(Parser, Debug)]
#[command(
    name = "sql-gen",
    version = "0.1.0",
    about = "Database code generation CLI"
)]
struct Cli {
    /// Database URL.
    #[arg(long)]
    db_url: String,

    /// Table names (can accept many).
    #[arg(long, value_name = "SQLGEN_INCLUDE_TABLES", value_delimiter = ',')]
    include_tables: Option<Vec<String>>,

    /// Enum derives to add (can be used multiple times).
    #[arg(
        long = "enum-derive",
        value_name = "SQLGEN_ENUM_DERIVE",
        value_delimiter = ','
    )]
    enum_derives: Option<Vec<String>>,

    /// Model derives to add (can be used multiple times).
    #[arg(
        long = "model-derive",
        value_name = "SQLGEN_MODEL_DERIVE",
        value_delimiter = ','
    )]
    model_derives: Option<Vec<String>>,

    /// Mode of code generation: either sqlx or dbset.
    #[arg(long,
        value_enum,
        value_name = "SQLGEN_MODE",
        default_value_t = Mode::Sqlx)]
    mode: Mode,
    /// Type overrides (can be used multiple times).
    #[arg(
        long = "type-overrides",
        value_name = "SQLGEN_TYPE_OVERRIDES",
        value_delimiter = ','
    )]
    type_overrides: Vec<String>,

    /// Field overrides (can be used multiple times).
    #[arg(
        long = "table-overrides",
        value_name = "SQLGEN_TABLE_OVERRIDES",
        value_delimiter = ','
    )]
    table_overrides: Vec<String>,

    /// Output .
    #[arg(long, default_value = "src/models/")]
    output: String,
    // /// Overwrite files flag (if set, files will be overwritten).
    // #[arg(long, action = clap::ArgAction::SetTrue)]
    // overwrite_files: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum, Default)]
pub enum Mode {
    #[default]
    Sqlx,
    Dbset,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum, Default)]
pub enum DatabaseType {
    #[default]
    Postgres,
    MySql,
}

async fn generate_rust_from_database(args: &Cli) -> DbSetsFsWriter {
    let database_type =
        if args.db_url.starts_with("postgres://") || args.db_url.starts_with("postgresql://") {
            DatabaseType::Postgres
        } else {
            DatabaseType::MySql
        };

    if args.mode == Mode::Dbset && database_type == DatabaseType::MySql {
        panic!("DbSet not currently supported for MySql")
    }

    let (enums, tables) = match database_type {
        DatabaseType::Postgres => {
            let pool = PgPoolOptions::new()
                .connect(&args.db_url)
                .await
                .expect("Failed to connect to postgres container");

            let enums = postgres::queries::get_enums::get_postgres_enums(&pool)
                .await
                .unwrap();

            let tables = postgres::queries::get_tables::get_tables(
                &pool,
                &[String::from("public")],
                &args.include_tables,
            )
            .await
            .unwrap();

            (enums, tables)
        }
        DatabaseType::MySql => {
            let pool = MySqlPoolOptions::new()
                .connect(&args.db_url)
                .await
                .expect("Failed to connect to mysql container");

            let enums = mysql::queries::get_enums::get_mysql_enums(&pool)
                .await
                .unwrap();

            let tables = mysql::queries::get_tables::get_tables(&pool, &[], &args.include_tables)
                .await
                .unwrap();

            (enums, tables)
        }
    };

    let mut options = CodegenOptions::default();
    options.set_mode(args.mode);
    options.set_type_overrides_from_arg(&args.type_overrides);
    options.set_table_column_overrides_from_arg(&args.table_overrides);
    options.add_enums(&enums);
    options.set_model_derives(&args.model_derives);
    options.set_enum_derives(&args.enum_derives);

    let structs_mapped =
        translators::convert_table_to_struct::convert_tables_to_struct(tables, &options);
    let enums_mapped =
        translators::convert_db_enum_to_rust_enum::convert_db_enums_to_rust_enum(enums, &options);

    let mut writer = DbSetsFsWriter::default();

    for rust_struct in structs_mapped {
        writer.add_struct(rust_struct);
    }

    for rust_enum in enums_mapped {
        writer.add_enum(rust_enum);
    }

    writer
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let writer = generate_rust_from_database(&args).await;

    if args.output.as_str() == "-" {
        writer.write_to_std_out();
    } else if args.output.ends_with(".rs") {
        writer.write_to_file(&args.output);
    } else if args.output.ends_with("/") {
        writer.write_db_sets_to_fs(&args.output);
    } else {
        println!(
            "WARNING: invalid output {} must end in .rs if single file or a / if folder",
            args.output
        )
    }
}
