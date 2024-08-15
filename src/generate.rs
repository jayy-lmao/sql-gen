use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::fs;
use std::path::Path;

use crate::db_queries::{get_table_columns, get_user_defined_enums};
use crate::models::TableColumn;
use crate::utils::{generate_enum_code, generate_struct_code, to_pascal_case, to_snake_case};

use crate::query_generate::generate_query_code;
use crate::utils::{DateTimeLib, SqlGenState};
use crate::STATE;

pub async fn generate(
    output_folder: &str,
    database_url: &str,
    context: Option<&str>,
    force: bool,
    include_tables: Option<Vec<&str>>,
    exclude_tables: Vec<String>,
    schemas: Option<Vec<&str>>,
    date_time_lib: DateTimeLib,
    struct_derives: Vec<String>,
    enum_derives: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the Postgres database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    let database_name = get_database_name(&pool).await?;
    println!("Generating for database: {}", database_name);

    let default_schema = "public";
    let rows = get_table_columns(&pool, schemas.unwrap_or(vec![default_schema]), None).await?;
    let user_defined = rows
        .iter()
        .filter_map(|e| {
            if e.data_type.as_str() == "USER-DEFINED" && e.udt_name.as_str() != "geometry" {
                Some(e.udt_name.clone())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    let enum_rows = get_user_defined_enums(&user_defined, &pool).await?;
    let mut unique_enums = std::collections::BTreeSet::new();
    for row in &enum_rows {
        unique_enums.insert(row.enum_name.clone());
    }
    let enums = unique_enums.into_iter().collect::<Vec<String>>();
    // Create the output folder if it doesn't exist
    fs::create_dir_all(output_folder)?;

    let mut unique = std::collections::BTreeSet::new();
    for row in &rows {
        unique.insert(row.table_name.clone());
    }
    let tables: Vec<String> = unique
        .into_iter()
        .collect::<Vec<String>>()
        .into_iter()
        .filter(|e| !exclude_tables.contains(e))
        .collect();

    if !enums.is_empty() {
        println!("Outputting user defined enums: {:?}", enums);
    }
    println!("Outputting tables: {:?}", tables);

    STATE
        .set(SqlGenState {
            user_defined: enums.clone(),
            date_time_lib,
            struct_derives,
            enum_derives,
        })
        .expect("Unable to set state");

    let mut rs_enums = Vec::new();

    for user_enum in enums {
        let enum_code = generate_enum_code(&user_enum, &enum_rows);
        rs_enums.push(enum_code);
    }

    // Generate structs and queries for each table
    for table in &tables {
        if let Some(ts) = include_tables.clone() {
            if !ts.contains(&table.as_str()) {
                continue;
            }
        }
        // Generate the struct code based on the row
        let struct_code = generate_struct_code(&table, &rows);

        // Generate the query code based on the row
        let query_code = generate_query_code(&table, &rows);

        let struct_file_path = format!("{}/{}.rs", output_folder, to_snake_case(&table));
        if Path::new(&struct_file_path).exists() && !force {
            eprintln!(
                "{} already exists, skipping. use --force flag to overwrite",
                struct_file_path
            );
        } else {
            fs::write(struct_file_path, struct_code)?;
        }

        // Write the query code to a file
        let query_file_path = format!("{}/{}_db_set.rs", output_folder, to_snake_case(&table));
        if Path::new(&query_file_path).exists() && !force {
            eprintln!(
                "{} already exists, skipping. use --force flag to overwrite",
                query_file_path
            );
        } else {
            fs::write(query_file_path, query_code)?;
        }
    }

    let context_code =
        generate_db_context(context.unwrap_or(&database_name), &rs_enums, &tables, &rows);
    let context_file_path = format!("{}/mod.rs", output_folder);
    fs::write(context_file_path, context_code)?;
    Ok(())
}

fn generate_db_context(
    database_name: &str,
    enums: &[String],
    tables: &[String],
    _rows: &[TableColumn],
) -> String {
    let mut db_context_code = String::new();

    db_context_code.push_str("#![allow(dead_code)]\n");
    db_context_code
        .push_str("// Generated with sql-gen\n//https://github.com/jayy-lmao/sql-gen\n\n");
    for enum_item in enums {
        db_context_code.push_str(enum_item);
        db_context_code.push_str("\n\n");
    }
    for table in tables {
        db_context_code.push_str(&format!("pub mod {};\n", to_snake_case(table)));
        db_context_code.push_str(&format!(
            "pub use {}::{};\n",
            to_snake_case(table),
            to_pascal_case(table),
        ));
        db_context_code.push_str(&format!("pub mod {}_db_set;\n", to_snake_case(table)));
        db_context_code.push_str(&format!(
            "pub use {}_db_set::{}Set;\n\n",
            to_snake_case(table),
            to_pascal_case(table),
        ));
    }

    db_context_code.push_str("\n");
    db_context_code.push_str(&format!(
        "pub struct {}Context;\n\n",
        to_pascal_case(database_name)
    ));
    db_context_code.push_str(&format!(
        "impl {}Context {{\n",
        to_pascal_case(database_name)
    ));
    for table in tables {
        db_context_code.push_str(&format!(
            "  pub fn {}(&self) -> {}Set {{ {}Set }}\n\n",
            to_snake_case(table),
            to_pascal_case(table),
            to_pascal_case(table),
        ));
    }
    db_context_code.push_str("}");
    db_context_code
}

async fn get_database_name(pool: &PgPool) -> Result<String, sqlx::Error> {
    let query = "SELECT current_database()";
    let row: (String,) = sqlx::query_as::<_, (String,)>(query)
        .fetch_one(pool)
        .await?;
    let database_name = row.0;

    Ok(database_name)
}
