use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn get_test_pool() -> PgPool {
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

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_container_db_uri.expect("Did not create postgres string"))
        .await
        .unwrap()
}
