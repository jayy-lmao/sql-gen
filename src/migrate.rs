use sqlx::postgres::{PgPool, PgPoolOptions};
use std::fs;
use std::path::PathBuf;

use crate::utils::{convert_data_type_from_pg, parse_struct_fields, to_pascal_case};

pub async fn migrate(
    include_folder: &str,
    output_folder: &str,
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the Postgres database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Read existing struct files from the include folder
    let existing_files = fs::read_dir(include_folder)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>();

    // Create the output folder if it doesn't exist
    fs::create_dir_all(output_folder)?;

    // Generate migrations for struct differences
    for file_path in existing_files {
        // Parse the struct name from the file name
        let file_name = file_path.file_stem().unwrap().to_string_lossy().to_string();
        let struct_name = file_name;

        // Read the struct code from the file
        let struct_code = fs::read_to_string(&file_path)?;

        // Check if the struct fields differ from the database
        let migration_code = generate_migration_code(&struct_name, struct_code, &pool).await?;

        // Generate a timestamp and migration name
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let migration_name = format!("{}_{}.sql", timestamp, struct_name);

        // Write the migration code to a file
        let migration_file_path = format!("{}/{}", output_folder, migration_name);
        if !migration_code.is_empty() {
            fs::write(migration_file_path, migration_code)?;
        }
    }

    Ok(())
}
pub async fn generate_migration_code(
    struct_name: &str,
    struct_code: String,
    pool: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let table_name_lower = struct_name.to_lowercase();
    let table_name_upper = to_pascal_case(struct_name);

    // Get the column names and data types from the struct code
    let fields = parse_struct_fields(&struct_code);

    // Query the database for column information
    let query_lower = format!(
        "

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
         WHERE table_name = '{}'",
        table_name_lower,
    );

    let existing_columns_lower: Vec<(String, String, String)> =
        sqlx::query_as(query_lower.as_str()).fetch_all(pool).await?;

    // Query the database for column information
    let query_upper = format!(
        "
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
         WHERE table_name = '{}'",
        table_name_upper,
    );

    let existing_columns_upper: Vec<(String, String, String)> =
        sqlx::query_as(query_upper.as_str()).fetch_all(pool).await?;

    let (table_name, existing_columns) = match (
        !existing_columns_lower.is_empty(),
        !existing_columns_upper.is_empty(),
    ) {
        (true, _) => (table_name_lower, existing_columns_lower),
        (_, true) => (table_name_upper, existing_columns_upper),
        _ => {
            panic!(
                "Table does not exist for {} or {}",
                table_name_lower, table_name_upper
            );
        }
    };

    // Compare existing columns with struct fields
    let mut migration_statements = Vec::<String>::new();

    for (column_name, data_type, is_nullable) in &fields {
        let matching_column = existing_columns
            .iter()
            .find(|(col_name, _, _)| col_name == column_name);

        if let Some((_, existing_type, existing_nullable)) = matching_column {
            // Compare data types and nullability
            let column_definition = convert_data_type_from_pg(data_type);
            if column_definition != existing_type || is_nullable != existing_nullable {
                let alter_table = format!("ALTER TABLE {}", table_name);

                // Generate appropriate column definition

                // Generate the ALTER TABLE statement
                let nullable_keyword = if is_nullable == "YES" {
                    "DROP NOT NULL"
                } else {
                    "SET NOT NULL"
                };

                let migration_statement = format!(
                    "{} ALTER COLUMN {} TYPE {}, {}",
                    alter_table, column_name, column_definition, nullable_keyword
                );

                migration_statements.push(migration_statement);
            }
        } else {
            let alter_table = format!("ALTER TABLE {}", table_name);
            let column_definition = convert_data_type_from_pg(data_type);

            let nullable_keyword = if is_nullable == "YES" {
                "NULL"
            } else {
                "NOT NULL"
            };
            let migration_statement = format!(
                "{} ADD COLUMN {} {} {}",
                alter_table, column_name, column_definition, nullable_keyword
            );
            migration_statements.push(migration_statement);
        }
    }

    // Compare existing columns with struct fields to identify removed columns
    let removed_columns: Vec<&(String, _, _)> = existing_columns
        .iter()
        .filter(|(col_name, _, _)| {
            !fields
                .iter()
                .any(|(field_name, _, _)| field_name == col_name)
        })
        .collect();

    for (column_name, _, _) in removed_columns {
        let alter_table = format!("ALTER TABLE {}", table_name);
        let drop_column = format!("DROP COLUMN {}", column_name);
        let migration_statement = format!("{} {}", alter_table, drop_column);
        migration_statements.push(migration_statement);
    }

    // Generate the full migration code
    let migration_code = if !migration_statements.is_empty() {
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let _migration_name = format!("{}_{}.sql", timestamp, struct_name);

        let migration_statements_code = migration_statements.join(";\n");

        format!(
            "-- Migration generated for struct: {}\n{}\n",
            struct_name, migration_statements_code
        )
    } else {
        String::new()
    };

    Ok(migration_code)
}
