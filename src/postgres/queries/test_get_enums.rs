use std::error::Error;

use sqlx::postgres::PgPoolOptions;

use crate::{core::models::CustomEnum, postgres::queries::get_enums::get_postgres_enums};

#[tokio::test]
async fn test_get_postgres_enums() -> Result<(), Box<dyn Error>> {
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

    // Initialize the database with enum types
    sqlx::query(
        r"
        DROP TYPE IF EXISTS mood CASCADE;
    ",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r"
        CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');
    ",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r"
        DROP TYPE IF EXISTS weather CASCADE;
    ",
    )
    .execute(&pool)
    .await?;
    sqlx::query(
        r"
        CREATE TYPE weather AS ENUM ('sunny', 'rainy', 'cloudy');

    ",
    )
    .execute(&pool)
    .await?;

    // Fetch the enums
    let enums = get_postgres_enums(&pool).await?;

    // Define the expected result
    let expected = vec![
        CustomEnum {
            name: "mood".to_string(),
            schema: "public".to_string(),
            variants: vec!["sad".to_string(), "ok".to_string(), "happy".to_string()],
        },
        CustomEnum {
            name: "weather".to_string(),
            schema: "public".to_string(),
            variants: vec![
                "sunny".to_string(),
                "rainy".to_string(),
                "cloudy".to_string(),
            ],
        },
    ];

    // Assert that the fetched enums match the expected result
    assert_eq!(enums, expected);

    Ok(())
}