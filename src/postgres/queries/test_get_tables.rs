use std::error::Error;

use sqlx::postgres::PgPoolOptions;

use crate::{
    core::models::{Table, TableColumn},
    postgres::queries::get_tables::get_tables,
};

#[tokio::test]
async fn test_get_postgres_tables() -> Result<(), Box<dyn Error>> {
    let mut test_container_db_uri: Option<String> = None;

    let docker = testcontainers::clients::Cli::default();
    let container = docker.run(testcontainers_modules::postgres::Postgres::default());
    let connection_string = &format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        container.get_host_port_ipv4(5432)
    );
    {
        test_container_db_uri = Some(connection_string.to_string());
    }

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_container_db_uri.expect("Did not create postgres string"))
        .await?;

    sqlx::query(
        r#"
            CREATE SCHEMA test_schema;
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
            CREATE TABLE test_schema.test_table (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) UNIQUE,
                description TEXT,
                parent_id INTEGER REFERENCES test_schema.test_table (id)
            );
        "#,
    )
    .execute(&pool)
    .await?;

    let schemas = vec!["test_schema"];
    let table_names = None;
    let tables = get_tables(&pool, schemas, table_names).await?;

    let expected = vec![Table {
        table_name: "test_table".to_string(),
        columns: vec![
            TableColumn {
                column_name: "id".to_string(),
                udt_name: "int4".to_string(),
                data_type: "integer".to_string(),
                is_nullable: false,
                is_unique: false,
                is_primary_key: true,
                foreign_key_table: None,
                foreign_key_id: None,
                table_schema: "test_schema".to_string(),
            },
            TableColumn {
                column_name: "name".to_string(),
                udt_name: "varchar".to_string(),
                data_type: "character varying".to_string(),
                is_nullable: false,
                is_unique: true,
                is_primary_key: false,
                foreign_key_table: None,
                foreign_key_id: None,
                table_schema: "test_schema".to_string(),
            },
            TableColumn {
                column_name: "description".to_string(),
                udt_name: "text".to_string(),
                data_type: "text".to_string(),
                is_nullable: true,
                is_unique: false,
                is_primary_key: false,
                foreign_key_table: None,
                foreign_key_id: None,
                table_schema: "test_schema".to_string(),
            },
            TableColumn {
                column_name: "parent_id".to_string(),
                udt_name: "int4".to_string(),
                data_type: "integer".to_string(),
                is_nullable: true,
                is_unique: false,
                is_primary_key: false,
                foreign_key_table: Some("test_table".to_string()),
                foreign_key_id: Some("id".to_string()),
                table_schema: "test_schema".to_string(),
            },
        ],
    }];

    assert_eq!(tables, expected);

    Ok(())
}
