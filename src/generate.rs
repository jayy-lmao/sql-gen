use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::fs;

use crate::models::TableColumn;
use crate::utils::{generate_struct_code, to_pascal_case, to_snake_case};

use crate::query_generate::generate_query_code;

pub async fn generate(
    output_folder: &str,
    database_url: &str,
    context: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the Postgres database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    let database_name = get_database_name(&pool).await?;
    println!("Generating for database: {}", database_name);

    // Get all tables from the database
    let query = "
            SELECT
    c.table_name,
    c.column_name,
    c.udt_name,
    c.is_nullable = 'YES' AS is_nullable,
    CASE
        WHEN k.column_name IS NOT NULL THEN TRUE
        ELSE FALSE
    END AS is_primary_key
FROM
    information_schema.columns c
LEFT JOIN
    information_schema.key_column_usage k ON
    c.table_schema = k.table_schema AND
    c.table_name = k.table_name AND
    c.column_name = k.column_name AND
    k.constraint_name IN (
        SELECT
            constraint_name
        FROM
            information_schema.table_constraints
        WHERE
            constraint_type = 'PRIMARY KEY'
    )
WHERE
    c.table_schema = 'public'
ORDER BY
    c.table_name,
    c.ordinal_position

        ";

    let rows = sqlx::query_as::<_, TableColumn>(query)
        .fetch_all(&pool)
        .await?;

    // Create the output folder if it doesn't exist
    fs::create_dir_all(output_folder)?;

    // let tables_duplicated = rows.iter().map(|row| row.table_name.clone()).collect::<Vec<String>>();
    let mut unique = std::collections::BTreeSet::new();
    for row in &rows {
        unique.insert(row.table_name.clone());
    }
    let tables: Vec<String> = unique.into_iter().collect::<Vec<String>>();

    println!("Outputting tables: {:?}", tables);

    // Generate structs and queries for each table
    for table in &tables {
        // Generate the struct code based on the row
        let struct_code = generate_struct_code(&table, &rows);

        // Generate the query code based on the row
        let query_code = generate_query_code(&table, &rows);

        let struct_file_path = format!("{}/{}.rs", output_folder, to_snake_case(&table));
        fs::write(struct_file_path, struct_code)?;

        // Write the query code to a file
        let query_file_path = format!("{}/{}_db_set.rs", output_folder, to_snake_case(&table));
        fs::write(query_file_path, query_code)?;
    }

    let context_code = generate_db_context(context.unwrap_or(&database_name), &tables, &rows);
    let context_file_path = format!("{}/mod.rs", output_folder);
    fs::write(context_file_path, context_code)?;
    Ok(())
}

fn generate_db_context(database_name: &str, tables: &[String], rows: &[TableColumn]) -> String {
    let mut db_context_code = String::new();
    db_context_code.push_str(&format!(
        "pub struct {}Context;\n\n",
        to_pascal_case(database_name)
    ));
    db_context_code.push_str(&format!(
        "impl {}Context {{\n",
        to_pascal_case(database_name)
    ));
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
