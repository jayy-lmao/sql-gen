use std::error::Error;

use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{
    core::models::{Table, TableColumn, TableColumnBuilder},
    postgres::queries::get_tables::get_tables,
};

fn unique(
    column_name: impl ToString,
    udt_name: impl ToString,
    data_type: impl ToString,
) -> TableColumn {
    TableColumn {
        column_name: column_name.to_string(),
        udt_name: udt_name.to_string(),
        data_type: data_type.to_string(),
        is_nullable: false,
        is_unique: true,
        is_primary_key: true,
        foreign_key_table: None,
        foreign_key_id: None,
    }
}

fn nullable(
    column_name: impl ToString,
    udt_name: impl ToString,
    data_type: impl ToString,
) -> TableColumn {
    TableColumn {
        column_name: column_name.to_string(),
        udt_name: udt_name.to_string(),
        data_type: data_type.to_string(),
        is_nullable: true,
        is_unique: false,
        is_primary_key: true,
        foreign_key_table: None,
        foreign_key_id: None,
    }
}

fn non_nullable(
    column_name: impl ToString,
    udt_name: impl ToString,
    data_type: impl ToString,
) -> TableColumn {
    TableColumn {
        column_name: column_name.to_string(),
        udt_name: udt_name.to_string(),
        data_type: data_type.to_string(),
        is_nullable: false,
        is_unique: false,
        is_primary_key: true,
        foreign_key_table: None,
        foreign_key_id: None,
    }
}

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

    let _ = test_table(
        &pool,
        "CREATE TABLE test_table (id SERIAL PRIMARY KEY, name VARCHAR(255) UNIQUE, description TEXT, parent_id INTEGER REFERENCES test_table (id));",
        vec![Table {
            table_name: "test_table".to_string(),
            table_schema: "public".to_string(),
            columns: vec![
                TableColumnBuilder::new("id", "int4", "integer")
                    .is_primary_key()
                    .build(),
                TableColumnBuilder::new("name", "varchar", "character varying")
                    .is_unique()
                    .is_nullable()
                    .build(),
                TableColumnBuilder::new("description", "text", "text")
                    .is_nullable()
                    .build(),
                TableColumnBuilder::new("parent_id", "int4", "integer")
                    .is_nullable()
                    .foreign_key_table("test_table")
                    .foreign_key_id("id")
                    .build(),
            ],
        }],
    )
    .await;

    Ok(())
}
