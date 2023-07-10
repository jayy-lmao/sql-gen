use clap::{App, Arg, SubCommand};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use std::fs;
use std::path::PathBuf;

#[derive(sqlx::FromRow)]
struct TableColumn {
    table_name: String,
    column_name: String,
    data_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("SQL Gen")
        .subcommand(
            SubCommand::with_name("generate")
                .about("Generate structs and queries for tables")
                .arg(
                    Arg::with_name("output")
                        .short('o')
                        .long("output")
                        .value_name("FOLDER")
                        .help("Sets the output folder")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("database")
                        .short('d')
                        .long("database")
                        .value_name("URL")
                        .help("Sets the database connection URL")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("migrate")
                .about("Generate SQL migrations based on struct differences")
                .arg(
                    Arg::with_name("include")
                        .short('i')
                        .long("include")
                        .value_name("FOLDER")
                        .help("Sets the folder containing existing struct files")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .short('o')
                        .long("output")
                        .value_name("FOLDER")
                        .help("Sets the output folder for migrations")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("database")
                        .short('d')
                        .long("database")
                        .value_name("URL")
                        .help("Sets the database connection URL")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        let output_folder = matches.value_of("output").unwrap();
        let database_url = matches.value_of("database").unwrap();

        // Connect to the Postgres database
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Get all tables from the database
        let query = "
            SELECT table_name, column_name, data_type
            FROM information_schema.columns
            WHERE table_schema = 'public'
        ";

        let rows = sqlx::query_as::<_, TableColumn>(query)
            .fetch_all(&pool)
            .await?;

        // Create the output folder if it doesn't exist
        fs::create_dir_all(output_folder)?;

        // Generate structs and queries for each table
        for row in rows {
            // Generate the struct code based on the row
            let struct_code = generate_struct_code(&row);

            // Generate the query code based on the row
            // let query_code = generate_query_code(&row);

            let struct_file_path = format!("{}/{}_struct.rs", output_folder, row.table_name);
            fs::write(struct_file_path, struct_code)?;

            // Write the query code to a file
            // let query_file_path = format!("{}/{}_query.rs", output_folder, row.table_name);
            // fs::write(query_file_path, query_code)?;
        }
    } else if let Some(matches) = matches.subcommand_matches("migrate") {
        let include_folder = matches.value_of("include").unwrap();
        let output_folder = matches.value_of("output").unwrap();
        let database_url = matches.value_of("database").unwrap();

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
            let struct_name = file_name.strip_suffix("_struct").unwrap();

            // Read the struct code from the file
            let struct_code = fs::read_to_string(&file_path)?;

            // Check if the struct fields differ from the database
            let migration_code = generate_migration_code(struct_name, struct_code, &pool).await?;

            // Generate a timestamp and migration name
            let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
            let migration_name = format!("{}_{}.sql", timestamp, struct_name);

            // Write the migration code to a file
            let migration_file_path = format!("{}/{}", output_folder, migration_name);
            fs::write(migration_file_path, migration_code)?;
        }
    }

    Ok(())
}

async fn generate_migration_code(
    struct_name: &str,
    struct_code: String,
    pool: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let table_name = struct_name.to_lowercase();

    // Get the column names and data types from the struct code
    let fields = parse_struct_fields(&struct_code);

    // Query the database for column information
    let query = format!(
        "SELECT column_name, data_type, is_nullable
         FROM information_schema.columns
         WHERE table_name = '{}'
           AND table_schema = 'public'",
        table_name
    );

    let existing_columns: Vec<(String, String, String)> = sqlx::query_as(query.as_str())
        .fetch_all(pool)
        .await?;

    // Compare existing columns with struct fields
    let mut migration_statements = Vec::<String>::new();

    for (column_name, data_type, is_nullable) in &fields {
        let matching_column = existing_columns.iter().find(|(col_name, _, _)| col_name == column_name);

        if let Some((_, existing_type, existing_nullable)) = matching_column {
            // Compare data types and nullability
            if data_type != existing_type || is_nullable != existing_nullable {
                let alter_table = format!("ALTER TABLE {}", table_name);

                // Generate appropriate column definition
                let column_definition = match data_type.as_str() {
                    "bool" => "BOOLEAN",
                    "i32" => "INTEGER",
                    "i64" => "BIGINT",
                    "f32" => "REAL",
                    "f64" => "DOUBLE PRECISION",
                    "String" => "TEXT",
                    // Add more data types as needed

                    _ => {
                        // Handle unsupported data types
                        println!("Warning: Unsupported data type: {}", data_type);
                        continue;
                    }
                };

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
            let column_definition = match data_type.as_str() {
                "bool" => "BOOLEAN",
                "i32" => "INTEGER",
                "i64" => "BIGINT",
                "f32" => "REAL",
                "f64" => "DOUBLE PRECISION",
                "String" => "TEXT",
                // Add more data types as needed

                _ => {
                    // Handle unsupported data types
                    println!("Warning: Unsupported data type: {}", data_type);
                    continue;
                }
            };
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
        .filter(|(col_name, _, _)| !fields.iter().any(|(field_name, _, _)| field_name == col_name))
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
        let migration_name = format!("{}_{}.sql", timestamp, struct_name);

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

fn generate_struct_code(row: &TableColumn) -> String {
    let struct_name = to_pascal_case(&row.table_name);
    let mut struct_code = format!("#[derive(sqlx::FromRow)]\n");
    struct_code.push_str(&format!("struct {} {{\n", struct_name));

    let column_name = to_snake_case(&row.column_name);
    let data_type = convert_data_type(&row.data_type);

    struct_code.push_str(&format!("    {}: {},\n", column_name, data_type));
    struct_code.push_str("}\n");

    struct_code
}

fn convert_data_type(data_type: &str) -> &str {
    match data_type {
        "int" => "i32",
        "bigint" => "i64",
        "real" => "f32",
        "double precision" => "f64",
        "boolean" => "bool",
        "text" => "String",
        // Add more data type conversions as needed

        _ => "Unsupported",
    }
}


fn generate_query_code(row: &TableColumn) -> String {
    // ... (implementation of generate_query_code)
    // query_code
    todo!()
}

// fn parse_struct_fields(struct_code: &str) -> Vec<(String, String, String)> {
//     let struct_regex = regex::Regex::new(r"pub\s+(?P<field>\w+):\s+(?P<type>\w+),?").unwrap();
//     let captures_iter = struct_regex.captures_iter(struct_code);

//     let mut fields = Vec::new();

//     for captures in captures_iter {
//         if let (Some(field), Some(data_type)) = (captures.name("field"), captures.name("type")) {
//             fields.push((field.as_str().to_owned(), data_type.as_str().to_owned(), "".to_owned()));
//         }
//     }

//     fields
// }

fn parse_struct_fields(struct_code: &str) -> Vec<(String, String, String)> {
    let lines = struct_code.lines();
    let mut fields = Vec::new();

    for line in lines {
        let trimmed_line = line.trim();
        if !trimmed_line.starts_with("pub") {
            continue;
        }

        let parts: Vec<&str> = trimmed_line.split(":").collect();
        if parts.len() != 2 {
            continue;
        }

        let field = parts[0].trim().trim_start_matches("pub").trim();
        let data_type = parts[1].trim().trim_end_matches(",").trim();

        fields.push((field.to_owned(), data_type.to_owned(), "".to_owned()));
    }

    fields
}


#[cfg(test)]
mod tests {
    // ... (unit tests can be defined here)
}

fn to_pascal_case(input: &str) -> String {
    let mut output = String::new();
    let mut capitalize_next = true;

    for c in input.chars() {
        if c.is_ascii_alphanumeric() {
            if capitalize_next {
                output.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                output.push(c);
            }
        } else {
            capitalize_next = true;
        }
    }

    output
}

fn to_snake_case(input: &str) -> String {
    let mut output = String::new();
    let mut prev_is_uppercase = false;

    for c in input.chars() {
        if c.is_ascii_uppercase() {
            if !output.is_empty() && !prev_is_uppercase {
                output.push('_');
            }
            output.extend(c.to_lowercase());
            prev_is_uppercase = true;
        } else {
            output.push(c);
            prev_is_uppercase = false;
        }
    }

    output
}
