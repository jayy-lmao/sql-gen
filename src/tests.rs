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
