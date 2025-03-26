use crate::{
    core::models::db::{Table, TableColumnBuilder},
    postgres::{queries::get_tables::get_tables, test_helper::setup_pg_db},
};
use pretty_assertions::assert_eq;
use sqlx::PgPool;
use std::error::Error;

async fn test_table(
    pool: &PgPool,
    statements: &[&str],
    expected: Vec<Table>,
) -> Result<(), Box<dyn Error>> {
    for statement in statements {
        sqlx::query(statement).execute(pool).await?;
    }

    let schemas = vec!["public".to_string()];
    let table_names = None;
    let tables = get_tables(pool, &schemas, &table_names).await?;

    assert_eq!(tables, expected);
    Ok(())
}

#[tokio::test]
async fn test_basic_postgres_tables() -> Result<(), Box<dyn Error>> {
    let (pool, _) = setup_pg_db().await;
    test_table(
        &pool,
        &["CREATE TABLE test_table_0 (id SERIAL PRIMARY KEY, name VARCHAR(255) UNIQUE, description TEXT, parent_id INTEGER REFERENCES test_table_0 (id));"],
        vec![Table {
            table_name: "test_table_0".to_string(),
            table_schema: Some("public".to_string()),
            columns: vec![
                TableColumnBuilder::new("id", "int4", "integer", Some("i32".to_string())).is_primary_key().is_auto_populated().build(),
                TableColumnBuilder::new("name", "varchar", "character varying", Some("String".to_string())).is_unique().is_nullable().build(),
                TableColumnBuilder::new("description", "text", "text", Some("String".to_string())).is_nullable().build(),
                TableColumnBuilder::new("parent_id", "int4", "integer", Some("i32".to_string())).is_nullable().foreign_key_table("test_table_0").foreign_key_id("id").build(),
            ],
            ..Default::default()
        }],
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_basic_postgres_tables_with_comments() -> Result<(), Box<dyn Error>> {
    let (pool, _) = setup_pg_db().await;
    test_table(
        &pool,
        &[
            "CREATE TABLE test_table_with_comments (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) UNIQUE,
            description TEXT
            );",
            "COMMENT ON TABLE test_table_with_comments IS 'Some test table comment';",
            "COMMENT ON COLUMN test_table_with_comments.id IS 'Some test table column comment';",
        ],
        vec![Table {
            table_name: "test_table_with_comments".to_string(),
            table_comment: Some("Some test table comment".to_string()),
            table_schema: Some("public".to_string()),
            columns: vec![
                TableColumnBuilder::new("id", "int4", "integer", Some("i32".to_string()))
                    .is_primary_key()
                    .is_auto_populated()
                    .add_column_comment("Some test table column comment")
                    .build(),
                TableColumnBuilder::new(
                    "name",
                    "varchar",
                    "character varying",
                    Some("String".to_string()),
                )
                .is_unique()
                .is_nullable()
                .build(),
                TableColumnBuilder::new("description", "text", "text", Some("String".to_string()))
                    .is_nullable()
                    .build(),
            ],
        }],
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_basic_postgres_table_with_array() -> Result<(), Box<dyn Error>> {
    let (pool, _) = setup_pg_db().await;
    test_table(
        &pool,
        &["CREATE TABLE test_table_1 (id SERIAL PRIMARY KEY, names TEXT[]);"],
        vec![Table {
            table_name: "test_table_1".to_string(),
            table_schema: Some("public".to_string()),
            columns: vec![
                TableColumnBuilder::new("id", "int4", "integer", Some("i32".to_string()))
                    .is_primary_key()
                    .is_auto_populated()
                    .build(),
                TableColumnBuilder::new("names", "_text", "ARRAY", Some("String".to_string()))
                    .is_nullable()
                    .array_depth(1)
                    .build(),
            ],
            ..Default::default()
        }],
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_postgres_table_with_custom_type() -> Result<(), Box<dyn Error>> {
    let (pool, _) = setup_pg_db().await;
    sqlx::query("DROP TYPE IF EXISTS status CASCADE;")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE TYPE status AS ENUM ('pending', 'shipped', 'delivered');")
        .execute(&pool)
        .await?;

    test_table(
        &pool,
        &["CREATE TABLE test_orders_status_0 (id SERIAL PRIMARY KEY, order_status status NOT NULL);"],
        vec![Table {
            table_name: "test_orders_status_0".to_string(),
            table_schema: Some("public".to_string()),
            columns: vec![
                TableColumnBuilder::new("id", "int4", "integer", Some("i32".to_string()))
                    .is_primary_key()
                    .is_auto_populated()
                    .build(),
                TableColumnBuilder::new("order_status", "status", "USER-DEFINED", None).build(),
            ],
            ..Default::default()
        }],
    )
    .await?;

    Ok(())
}
