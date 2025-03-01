use std::sync::{Mutex, OnceLock};
use testcontainers::clients;
use testcontainers_modules::postgres::Postgres;

type StoredDbContainer = Option<(testcontainers::Container<'static, Postgres>, String)>;

static DOCKER: OnceLock<clients::Cli> = OnceLock::new();
static CONTAINER: OnceLock<Mutex<StoredDbContainer>> = OnceLock::new();

#[cfg(test)]
fn setup_test_db() -> String {
    let container_lock = CONTAINER.get_or_init(|| Mutex::new(None));
    let mut container_guard = container_lock.lock().unwrap();

    if container_guard.is_none() {
        let docker = DOCKER.get_or_init(clients::Cli::default);

        let container = docker.run(Postgres::default());
        let port = container.get_host_port_ipv4(5432);
        let db_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

        *container_guard = Some((container, db_url.clone()));
    }

    container_guard.as_ref().unwrap().1.clone()
}

/// Runs before all tests **only in test mode**
#[cfg(test)]
#[ctor::ctor]
fn initialize_database() {
    if std::env::var("DATABASE_URL").is_ok() {
        return; // Skip setup if DATABASE_URL is already set
    }

    let db_url = setup_test_db();

    std::env::set_var("DATABASE_URL", db_url);
}
