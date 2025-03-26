use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::{
    process::Command,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio::time::timeout;
use uuid::Uuid;

static CONTAINER_GUARD: OnceLock<Arc<ContainerGuard>> = OnceLock::new();

struct ContainerGuard;

impl ContainerGuard {
    fn new() -> Self {
        let container_name = "rust_test_sqlx_mysql_container";
        let port = "3307";

        // Try to create the container
        let output = Command::new("docker")
            .args([
                "run",
                "--rm",
                "-d",
                "--name",
                container_name,
                "-e",
                "MYSQL_ROOT_PASSWORD=root",
                "-p",
                &format!("{port}:3306"),
                "mysql:latest",
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

pub async fn setup_mysql_db() -> (MySqlPool, String) {
    // Ensure exactly one container guard is created (across threads within the same process)
    let _guard = CONTAINER_GUARD.get_or_init(|| Arc::new(ContainerGuard::new()));

    let root_db_url = "mysql://root:root@localhost:3307/mysql";
    wait_for_mysql_ready(root_db_url).await;

    // Connect to the default "mysql" database to create a new test database
    let root_pool = MySqlPoolOptions::new()
        .connect(root_db_url)
        .await
        .expect("Failed to connect to mysql container");

    let db_name = format!("test_db_{}", Uuid::new_v4().simple());

    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&root_pool)
        .await
        .expect("Failed to create test database");

    let test_db_url = format!("mysql://root:root@localhost:3307/{}", db_name);

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&test_db_url)
        .await
        .expect("Failed to connect to test database");
    (pool, test_db_url)
}

async fn wait_for_mysql_ready(db_url: &str) -> MySqlPool {
    const MAX_TRIES: usize = 10;
    const DELAY: Duration = Duration::from_secs(1);
    const CONN_TIMEOUT: Duration = Duration::from_secs(1);

    for _ in 0..MAX_TRIES {
        // Wrap the connection attempt in a timeout
        match timeout(CONN_TIMEOUT, MySqlPoolOptions::new().connect(db_url)).await {
            Ok(Ok(pool)) => return pool,
            Ok(Err(e)) => eprintln!("Connection error: {}", e),
            Err(_) => eprintln!("Connection attempt timed out after {:?}", CONN_TIMEOUT),
        }
        tokio::time::sleep(DELAY).await;
    }
    panic!("MySQL container failed to become ready in time");
}
