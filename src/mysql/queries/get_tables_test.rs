use crate::{
    core::models::db::{Table, TableColumnBuilder},
    mysql::{queries::get_tables::get_tables, test_helper::setup_mysql_db},
};
use pretty_assertions::assert_eq;
use sqlx::MySqlPool;
use std::error::Error;

async fn test_table(
    pool: &MySqlPool,
    statements: &[&str],
    expected: Vec<Table>,
) -> Result<(), Box<dyn Error>> {
    // Execute each seed statement
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
async fn test_basic_mysql_tables() -> Result<(), Box<dyn Error>> {
    let (pool, _) = setup_mysql_db().await;
    test_table(
        &pool,
        &["CREATE TABLE test_table_0 (
                id INT AUTO_INCREMENT PRIMARY KEY,
                name VARCHAR(255) UNIQUE,
                description TEXT,
                parent_id INT,
                FOREIGN KEY (parent_id) REFERENCES test_table_0(id)
            );"],
        vec![Table {
            table_name: "test_table_0".to_string(),
            table_schema: None,
            columns: vec![
                TableColumnBuilder::new("id", "int", "int", Some("i32".to_string()))
                    .is_primary_key()
                    .is_auto_populated()
                    .build(),
                TableColumnBuilder::new(
                    "name",
                    "varchar(255)",
                    "varchar",
                    Some("String".to_string()),
                )
                .is_unique()
                .is_nullable()
                .build(),
                TableColumnBuilder::new("description", "text", "text", Some("String".to_string()))
                    .is_nullable()
                    .build(),
                TableColumnBuilder::new("parent_id", "int", "int", Some("i32".to_string()))
                    .is_nullable()
                    .foreign_key_table("test_table_0")
                    .foreign_key_id("id")
                    .build(),
            ],
            ..Default::default()
        }],
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_basic_mysql_tables_with_comments() -> Result<(), Box<dyn Error>> {
    let (pool, _) = setup_mysql_db().await;
    test_table(
        &pool,
        &["CREATE TABLE test_table_with_comments (
                id INT AUTO_INCREMENT PRIMARY KEY COMMENT 'Some test table column comment',
                name VARCHAR(255) UNIQUE,
                description TEXT
            ) COMMENT = 'Some test table comment';"],
        vec![Table {
            table_name: "test_table_with_comments".to_string(),
            table_comment: Some("Some test table comment".to_string()),
            table_schema: None,
            columns: vec![
                TableColumnBuilder::new("id", "int", "int", Some("i32".to_string()))
                    .is_primary_key()
                    .is_auto_populated()
                    .add_column_comment("Some test table column comment")
                    .build(),
                TableColumnBuilder::new(
                    "name",
                    "varchar(255)",
                    "varchar",
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
async fn test_basic_mysql_table_with_array() -> Result<(), Box<dyn Error>> {
    let (pool, _) = setup_mysql_db().await;
    // MySQL does not support array types. We use a JSON column instead.
    test_table(
        &pool,
        &["CREATE TABLE test_table_1 (
                id INT AUTO_INCREMENT PRIMARY KEY,
                names JSON
            );"],
        vec![Table {
            table_name: "test_table_1".to_string(),
            table_schema: None,
            columns: vec![
                TableColumnBuilder::new("id", "int", "int", Some("i32".to_string()))
                    .is_primary_key()
                    .is_auto_populated()
                    .build(),
                // Note: instead of an array, we expect a JSON type without array depth.
                TableColumnBuilder::new(
                    "names",
                    "json",
                    "json",
                    Some("serde_json::JsonValue".to_string()),
                )
                .is_nullable()
                .build(),
            ],
            ..Default::default()
        }],
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_mysql_table_with_custom_type() -> Result<(), Box<dyn Error>> {
    let (pool, _) = setup_mysql_db().await;

    // MySQL does not support custom types (CREATE TYPE). Instead, define ENUM directly.
    test_table(
        &pool,
        &["CREATE TABLE test_orders_status_0 (
                id INT AUTO_INCREMENT PRIMARY KEY,
                order_status ENUM('pending', 'shipped', 'delivered') NOT NULL
            );"],
        vec![Table {
            table_name: "test_orders_status_0".to_string(),
            table_schema: None,
            columns: vec![
                TableColumnBuilder::new("id", "int", "int", Some("i32".to_string()))
                    .is_primary_key()
                    .is_auto_populated()
                    .build(),
                // The expected type is now 'enum' instead of a custom type.
                TableColumnBuilder::new("order_status", "order_status", "enum", None).build(),
            ],
            ..Default::default()
        }],
    )
    .await?;

    Ok(())
}
