use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn get_test_pool() -> PgPool {
    let docker = testcontainers::clients::Cli::default();
    let container = docker.run(testcontainers_modules::postgres::Postgres::default());
    let connection_string = &format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        container.get_host_port_ipv4(5432)
    );

    PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string)
        .await
        .unwrap()
}
