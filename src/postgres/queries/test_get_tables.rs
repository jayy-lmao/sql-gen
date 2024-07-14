use std::error::Error;

use sqlx::PgPool;

use crate::{
    core::models::{Table, TableColumnBuilder},
    postgres::queries::{get_tables::get_tables, test_helper::get_test_pool},
};

async fn test_table(
    pool: &PgPool,
    statement: &str,
    expected: Vec<Table>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(statement).execute(pool).await?;

    let schemas = vec!["public"];
    let table_names = None;
    let tables = get_tables(pool, schemas, table_names).await?;

    assert_eq!(tables, expected);
    Ok(())
}

#[tokio::test]
async fn test_get_postgres_tables() -> Result<(), Box<dyn Error>> {
    let pool = get_test_pool().await;
    let _ = test_table(
        &pool,
        "CREATE TABLE test_table (id SERIAL PRIMARY KEY, name VARCHAR(255) UNIQUE, description TEXT, parent_id INTEGER REFERENCES test_table (id));",
        vec![Table {
            table_name: "test_table".to_string(),
            table_schema: "public".to_string(),
            columns: vec![
                TableColumnBuilder::new("id", "int4", "integer").is_primary_key().build(),
                TableColumnBuilder::new("name", "varchar", "character varying").is_unique().is_nullable().build(),
                TableColumnBuilder::new("description", "text", "text").is_nullable().build(),
                TableColumnBuilder::new("parent_id", "int4", "integer").is_nullable().foreign_key_table("test_table").foreign_key_id("id").build(),
            ],
        }],
    )
    .await;

    Ok(())
}
