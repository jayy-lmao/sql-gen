use crate::{
    core::models::db::{Table, TableColumnBuilder},
    postgres::queries::get_tables::get_tables,
};
use pretty_assertions::assert_eq;
use sqlx::PgPool;
use std::error::Error;

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

#[sqlx::test]
async fn test_basic_postgres_tables(pool: PgPool) -> Result<(), Box<dyn Error>> {
    test_table(
        &pool,
        "CREATE TABLE test_table_0 (id SERIAL PRIMARY KEY, name VARCHAR(255) UNIQUE, description TEXT, parent_id INTEGER REFERENCES test_table_0 (id));",
        vec![Table {
            table_name: "test_table_0".to_string(),
            table_schema: "public".to_string(),
            columns: vec![
                TableColumnBuilder::new("id", "int4", "integer").is_primary_key().build(),
                TableColumnBuilder::new("name", "varchar", "character varying").is_unique().is_nullable().build(),
                TableColumnBuilder::new("description", "text", "text").is_nullable().build(),
                TableColumnBuilder::new("parent_id", "int4", "integer").is_nullable().foreign_key_table("test_table_0").foreign_key_id("id").build(),
            ],
        }],
    )
    .await?;

    Ok(())
}

#[sqlx::test]
async fn test_basic_postgres_table_with_array(pool: PgPool) -> Result<(), Box<dyn Error>> {
    test_table(
        &pool,
        "CREATE TABLE test_table_1 (id SERIAL PRIMARY KEY, names TEXT[]);",
        vec![Table {
            table_name: "test_table_1".to_string(),
            table_schema: "public".to_string(),
            columns: vec![
                TableColumnBuilder::new("id", "int4", "integer")
                    .is_primary_key()
                    .build(),
                TableColumnBuilder::new("names", "_text", "ARRAY")
                    .is_nullable()
                    .build(),
            ],
        }],
    )
    .await?;

    Ok(())
}

#[sqlx::test]
async fn test_postgres_table_with_custom_type(pool: PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query("DROP TYPE IF EXISTS status CASCADE;")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE TYPE status AS ENUM ('pending', 'shipped', 'delivered');")
        .execute(&pool)
        .await?;

    test_table(
        &pool,
        "CREATE TABLE test_orders_status_0 (id SERIAL PRIMARY KEY, order_status status NOT NULL);",
        vec![Table {
            table_name: "test_orders_status_0".to_string(),
            table_schema: "public".to_string(),
            columns: vec![
                TableColumnBuilder::new("id", "int4", "integer")
                    .is_primary_key()
                    .build(),
                TableColumnBuilder::new("order_status", "status", "USER-DEFINED").build(),
            ],
        }],
    )
    .await?;

    Ok(())
}
