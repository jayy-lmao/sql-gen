use clap::Parser as _;
use pretty_assertions::assert_eq;

use crate::{generate_rust_from_database, postgres::test_helper::setup_pg_db, Cli};
use std::error::Error;

#[tokio::test]
async fn test_basic_postgres_tables() -> Result<(), Box<dyn Error>> {
    let (pool, uri) = setup_pg_db().await;
    let statement = "CREATE TABLE test_table_0 (id SERIAL PRIMARY KEY, name VARCHAR(255) UNIQUE, description TEXT, parent_id INTEGER REFERENCES test_table_0 (id));";
    sqlx::query(statement).execute(&pool).await?;

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

    sqlx::query(statement_1).execute(&pool).await?;
    sqlx::query(statement_2).execute(&pool).await?;
    sqlx::query(statement_3).execute(&pool).await?;

    let args = Cli::parse_from(["sql-gen", "--db-url", uri.as_str()]);

    let writer = generate_rust_from_database(&args).await;

    assert_eq!(
        writer.write_to_string().trim(),
        r#"
#[derive(sqlx::Type)]
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
async fn test_basic_postgres_table_with_and_type_override_for_enum_type(
) -> Result<(), Box<dyn Error>> {
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

    sqlx::query(statement_1).execute(&pool).await?;
    sqlx::query(statement_2).execute(&pool).await?;
    sqlx::query(statement_3).execute(&pool).await?;

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
/// Table to store todo items with tags and status information.
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
