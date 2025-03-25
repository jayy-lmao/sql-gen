use clap::Parser as _;
use pretty_assertions::assert_eq;
use sqlx::query;

use crate::{generate_rust_from_database, postgres::test_helper::setup_pg_db, Cli};
use std::error::Error;

#[tokio::test]
async fn test_basic_postgres_tables() -> Result<(), Box<dyn Error>> {
    let (pool, uri) = setup_pg_db().await;
    let statement = "CREATE TABLE test_table_0 (id SERIAL PRIMARY KEY, name VARCHAR(255) UNIQUE, description TEXT, parent_id INTEGER REFERENCES test_table_0 (id));";
    query(statement).execute(&pool).await?;

    let args = Cli::parse_from(["sql-gen", "--db-url", uri.as_str()]);

    let writer = generate_rust_from_database(&args).await;

    assert_eq!(
        writer.write_to_string().trim(),
        r#"#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TestTable0 {
    id: i32,
    name: Option<String>,
    description: Option<String>,
    parent_id: Option<i32>,
}"#
        .to_string()
    );

    Ok(())
}

#[tokio::test]
async fn test_basic_postgres_table_with_enum() -> Result<(), Box<dyn Error>> {
    let (pool, uri) = setup_pg_db().await;
    let statement_1 = "
-- Create an enum type for todo statuses.
CREATE TYPE todo_status AS ENUM ('pending', 'in_progress', 'completed');
";
    let statement_2 = "
-- Create the todos table.
CREATE TABLE todos (
    id SERIAL PRIMARY KEY,                  -- Primary key
    title VARCHAR(255) NOT NULL UNIQUE,     -- Non-nullable and unique field
    description TEXT,                       -- Nullable by default
    tags TEXT[] NOT NULL,                   -- Array field (non-nullable)
    status todo_status NOT NULL DEFAULT 'pending'  -- Enum field with a default value
);
";

    let statement_3 = "
-- Add a comment to the table.
COMMENT ON TABLE todos IS 'Table to store todo items with tags and status information.';
";

    query(statement_1).execute(&pool).await?;
    query(statement_2).execute(&pool).await?;
    query(statement_3).execute(&pool).await?;

    let args = Cli::parse_from(["sql-gen", "--db-url", uri.as_str()]);

    let writer = generate_rust_from_database(&args).await;

    assert_eq!(
        writer.write_to_string().trim(),
        r#"
#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "todo_status")]
pub enum TodoStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "in_progress")]
    InProgress,
    #[sqlx(rename = "completed")]
    Completed,
}

/// Table to store todo items with tags and status information.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Todo {
    id: i32,
    title: String,
    description: Option<String>,
    tags: Vec<String>,
    status: TodoStatus,
}
"#
        .to_string()
        .trim()
    );

    Ok(())
}

#[tokio::test]
async fn test_type_override_for_enum_type() -> Result<(), Box<dyn Error>> {
    let (pool, uri) = setup_pg_db().await;
    let statement_1 = "
-- Create an enum type for todo statuses.
CREATE TYPE todo_status AS ENUM ('pending', 'in_progress', 'completed');
";
    let statement_2 = "
-- Create the todos table.
CREATE TABLE todos (
    id SERIAL PRIMARY KEY,                  -- Primary key
    title VARCHAR(255) NOT NULL UNIQUE,     -- Non-nullable and unique field
    description TEXT,                       -- Nullable by default
    tags TEXT[] NOT NULL,                   -- Array field (non-nullable)
    status todo_status NOT NULL DEFAULT 'pending'  -- Enum field with a default value
);
";

    query(statement_1).execute(&pool).await?;
    query(statement_2).execute(&pool).await?;

    let args = Cli::parse_from([
        "sql-gen",
        "--db-url",
        uri.as_str(),
        "--type-overrides",
        "todo_status=String",
    ]);

    let writer = generate_rust_from_database(&args).await;

    assert_eq!(
        writer.write_to_string().trim(),
        r#"
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Todo {
    id: i32,
    title: String,
    description: Option<String>,
    tags: Vec<String>,
    status: String,
}
"#
        .to_string()
        .trim()
    );

    Ok(())
}

#[tokio::test]
async fn test_field_override() -> Result<(), Box<dyn Error>> {
    let (pool, uri) = setup_pg_db().await;
    let statement_1 = "
-- Create an enum type for todo statuses.
CREATE TYPE todo_status AS ENUM ('pending', 'in_progress', 'completed');
";
    let statement_2 = "
-- Create the todos table.
CREATE TABLE todos (
    id SERIAL PRIMARY KEY,                  -- Primary key
    title VARCHAR(255) NOT NULL UNIQUE,     -- Non-nullable and unique field
    description TEXT,                       -- Nullable by default
    tags TEXT[] NOT NULL,                   -- Array field (non-nullable)
    status todo_status NOT NULL DEFAULT 'pending'  -- Enum field with a default value
);
";

    query(statement_1).execute(&pool).await?;
    query(statement_2).execute(&pool).await?;

    let args = Cli::parse_from([
        "sql-gen",
        "--db-url",
        uri.as_str(),
        "--table-overrides",
        "status=String",
    ]);

    let writer = generate_rust_from_database(&args).await;

    assert_eq!(
        writer.write_to_string().trim(),
        r#"
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Todo {
    id: i32,
    title: String,
    description: Option<String>,
    tags: Vec<String>,
    status: String,
}
"#
        .to_string()
        .trim()
    );

    Ok(())
}

#[tokio::test]
async fn test_table_specific_field_override() -> Result<(), Box<dyn Error>> {
    let (pool, uri) = setup_pg_db().await;
    let statement_1 = "
-- Create an enum type for todo statuses.
CREATE TYPE todo_status AS ENUM ('pending', 'in_progress', 'completed');
";
    let statement_2 = "
-- Create the todos table.
CREATE TABLE todos (
    id SERIAL PRIMARY KEY,                  -- Primary key
    title VARCHAR(255) NOT NULL UNIQUE,     -- Non-nullable and unique field
    description TEXT,                       -- Nullable by default
    tags TEXT[] NOT NULL,                   -- Array field (non-nullable)
    status todo_status NOT NULL DEFAULT 'pending'  -- Enum field with a default value
);
";

    let statement_3 = "
CREATE TABLE other_todos_table (
    id SERIAL PRIMARY KEY,                  -- Primary key
    status todo_status NOT NULL DEFAULT 'pending'  -- Enum field with a default value
);
";

    query(statement_1).execute(&pool).await?;
    query(statement_2).execute(&pool).await?;
    query(statement_3).execute(&pool).await?;

    let args = Cli::parse_from([
        "sql-gen",
        "--db-url",
        uri.as_str(),
        "--table-overrides",
        "todos.status=String,toto.status=i32",
    ]);

    let writer = generate_rust_from_database(&args).await;

    assert_eq!(
        writer.write_to_string().trim(),
        r#"
#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "todo_status")]
pub enum TodoStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "in_progress")]
    InProgress,
    #[sqlx(rename = "completed")]
    Completed,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OtherTodosTable {
    id: i32,
    status: TodoStatus,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Todo {
    id: i32,
    title: String,
    description: Option<String>,
    tags: Vec<String>,
    status: String,
}


"#
        .to_string()
        .trim()
    );

    Ok(())
}

/// Test using the include_tables flag: if multiple tables exist, only the specified table is generated.
#[tokio::test]
async fn test_include_tables_filter() -> Result<(), Box<dyn Error>> {
    let (pool, uri) = setup_pg_db().await;
    // Create two tables in the database.
    let statement1 = "CREATE TABLE table_one (id SERIAL PRIMARY KEY, data TEXT);";
    let statement2 = "CREATE TABLE table_two (id SERIAL PRIMARY KEY, info TEXT);";
    query(statement1).execute(&pool).await?;
    query(statement2).execute(&pool).await?;

    // Only include table_one in the generated output.
    let args = Cli::parse_from([
        "sql-gen",
        "--db-url",
        uri.as_str(),
        "--include-tables",
        "table_one",
    ]);
    let writer = generate_rust_from_database(&args).await;
    let expected = r#"
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TableOne {
    id: i32,
    data: Option<String>,
}
"#;
    assert_eq!(writer.write_to_string().trim(), expected.trim());
    Ok(())
}

/// Test passing extra derives for an enum type using the enum-derive flag.
#[tokio::test]
async fn test_enum_derives_flag() -> Result<(), Box<dyn Error>> {
    let (pool, uri) = setup_pg_db().await;
    let statement_1 = "CREATE TYPE color AS ENUM ('red', 'green', 'blue');";
    let statement_2 = "
            CREATE TABLE palette (
                id SERIAL PRIMARY KEY,
                favorite color NOT NULL
            );
        ";
    query(statement_1).execute(&pool).await?;
    query(statement_2).execute(&pool).await?;

    let args = Cli::parse_from([
        "sql-gen",
        "--db-url",
        uri.as_str(),
        "--enum-derive",
        "Debug,PartialEq",
    ]);
    let writer = generate_rust_from_database(&args).await;
    let expected = r#"
#[derive(Debug, PartialEq)]
#[sqlx(type_name = "color")]
pub enum Color {
    #[sqlx(rename = "red")]
    Red,
    #[sqlx(rename = "green")]
    Green,
    #[sqlx(rename = "blue")]
    Blue,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Palette {
    id: i32,
    favorite: Color,
}
"#;
    assert_eq!(writer.write_to_string().trim(), expected.trim());
    Ok(())
}

/// Test passing extra derives for the model using the model-derive flag.
#[tokio::test]
async fn test_model_derives_flag() -> Result<(), Box<dyn Error>> {
    let (pool, uri) = setup_pg_db().await;
    let statement = "CREATE TABLE simple (id SERIAL PRIMARY KEY, value TEXT);";
    query(statement).execute(&pool).await?;

    let args = Cli::parse_from([
        "sql-gen",
        "--db-url",
        uri.as_str(),
        "--model-derive",
        "Debug,Clone,PartialEq",
    ]);
    let writer = generate_rust_from_database(&args).await;
    let expected = r#"
#[derive(Debug, Clone, PartialEq)]
pub struct Simple {
    id: i32,
    value: Option<String>,
}
"#;
    assert_eq!(writer.write_to_string().trim(), expected.trim());
    Ok(())
}
