use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    process::Command,
    sync::{Arc, OnceLock},
};
use uuid::Uuid;

static CONTAINER_GUARD: OnceLock<Arc<ContainerGuard>> = OnceLock::new();

struct ContainerGuard;

impl ContainerGuard {
    fn new() -> Self {
        let container_name = "rust_test_sqlx_container";
        let port = "5434";

        // Try to create the container
        let output = Command::new("docker")
            .args([
                "run",
                "--rm",
                "-d",
                "--name",
                container_name,
                "-e",
                "POSTGRES_USER=postgres",
                "-e",
                "POSTGRES_PASSWORD=postgres",
                "-p",
                &format!("{port}:5432"),
                "postgres:latest",
            ])
            .output()
            .expect("failed to run docker");

        // If container exists already, just start it
        if !output.status.success() {
            Command::new("docker")
                .args(["start", container_name])
                .status()
                .expect("Failed to start existing container");
        }

        Self
    }
}

pub async fn setup_pg_db() -> (PgPool, String) {
    // Ensure exactly one container guard is created (across threads within the same process)
    let _guard = CONTAINER_GUARD.get_or_init(|| Arc::new(ContainerGuard::new()));

    wait_for_postgres_ready().await;

    let root_db_url = "postgres://postgres:postgres@localhost:5434/postgres";
    let root_pool = PgPoolOptions::new()
        .connect(root_db_url)
        .await
        .expect("Failed to connect to postgres container");

    let db_name = format!("test_db_{}", Uuid::new_v4().simple());

    sqlx::query(&format!("CREATE DATABASE {db_name}"))
        .execute(&root_pool)
        .await
        .expect("Failed to create test database");

    let test_db_url = format!("postgres://postgres:postgres@localhost:5434/{db_name}");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_db_url)
        .await
        .expect("Failed to connect to test database");
    (pool, test_db_url)
}

async fn wait_for_postgres_ready() {
    const MAX_TRIES: usize = 10;
    const DELAY: std::time::Duration = std::time::Duration::from_millis(500);
    for _ in 0..MAX_TRIES {
        if Command::new("docker")
            .args(["exec", "rust_test_sqlx_container", "pg_isready"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return;
        }
        tokio::time::sleep(DELAY).await;
    }
    panic!("Postgres container failed to become ready in time");
}
