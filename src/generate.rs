use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::fs;
use std::path::Path;

use crate::db_queries::get_table_columns;
use crate::models::TableColumn;
use crate::utils::{generate_struct_code, to_pascal_case, to_snake_case};

use crate::query_generate::generate_query_code;

pub async fn generate(
    output_folder: &str,
    database_url: &str,
    context: Option<&str>,
    force: bool,
    include_tables: Option<Vec<&str>>,
    schemas: Option<Vec<&str>>,
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

    // Create the output folder if it doesn't exist
    fs::create_dir_all(output_folder)?;

    let mut unique = std::collections::BTreeSet::new();
    for row in &rows {
        unique.insert(row.table_name.clone());
    }
    let tables: Vec<String> = unique.into_iter().collect::<Vec<String>>();

    println!("Outputting tables: {:?}", tables);

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

    let context_code = generate_db_context(context.unwrap_or(&database_name), &tables, &rows);
    let context_file_path = format!("{}/mod.rs", output_folder);
    fs::write(context_file_path, context_code)?;
    Ok(())
}

fn generate_db_context(database_name: &str, tables: &[String], _rows: &[TableColumn]) -> String {
    let mut db_context_code = String::new();

    db_context_code.push_str("#![allow(dead_code)]\n");
    db_context_code
        .push_str("// Generated with SQLGEN\n//https://github.com/jayy-lmao/sql-codegen\n\n");
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
