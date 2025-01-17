use std::sync::LazyLock;
use testcontainers::Container;
use tokio::sync::OnceCell;

use clap::{App, Arg, SubCommand};

mod db_queries;
mod generate;
mod migrate;
mod models;
mod query_generate;
mod utils;

type TestPostgres = testcontainers_modules::postgres::Postgres;

// LazyLock for testcontainers::Cli, created once and shared
static DOCKER_CLI: LazyLock<testcontainers::clients::Cli> =
    LazyLock::new(testcontainers::clients::Cli::default);

// Global LazyLock holding both the container and database pool.
static DB_RESOURCES: LazyLock<OnceCell<(Container<'static, TestPostgres>, String)>> =
    LazyLock::new(OnceCell::new);

async fn prepare_db() -> (Container<'static, TestPostgres>, String) {
    let container = DOCKER_CLI.run(TestPostgres::default());
    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        container.get_host_port_ipv4(5432)
    );

    (container, connection_string)
}

pub async fn get_test_db_string() -> String {
    let (_, conn_string) = DB_RESOURCES.get_or_init(prepare_db).await;
    conn_string.clone()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let generate_subcommand = SubCommand::with_name("generate")
        .about("Generate structs and queries for tables")
        .arg(
            Arg::with_name("models")
                .short('o')
                .long("models")
                .default_value("src/models/")
                .value_name("SQLGEN_MODEL_OUTPUT_FOLDER")
                .help("Sets the output folder for generated structs")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("migrations")
                .short('m')
                .long("migrations")
                .value_name("SQLGEN_MIGRATIONS_INPUT")
                .help("The folder of migrations to apply. Leave blank if you do not wish to apply migrations before generating.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("database")
                .short('d')
                .long("database")
                .default_value("docker")
                .value_name("DATABASE_URL")
                .help(
                    "Sets the database connection URL. Or write docker to spin up a testcontainer",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("context")
                .short('c')
                .long("context")
                .value_name("SQLGEN_CONTEXT_NAME")
                .help("The name of the context for calling functions. Defaults to DB name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("schema")
                .short('s')
                .long("schema")
                .takes_value(true)
                .multiple(true)
                .use_delimiter(true)
                .help("Specify the schema name(s)"),
        )
        .arg(
            Arg::with_name("table")
                .short('t')
                .long("table")
                .takes_value(true)
                .value_name("SQLGEN_TABLE")
                .multiple(true)
                .use_delimiter(true)
                .help("Specify the table name(s)"),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .value_name("SQLGEN_OVERWRITE")
                .takes_value(false)
                .required(false)
                .help("Overwrites existing files sharing names in that folder"),
        );

    let migrate_subcommand = SubCommand::with_name("migrate")
        .about("Generate SQL migrations based on struct differences")
        .arg(
            Arg::with_name("models")
                .short('o')
                .long("models")
                .default_value("migrations")
                .value_name("SQLGEN_MODEL_FOLDER")
                .help("Sets the folder containing existing struct files")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("table")
                .short('t')
                .long("table")
                .value_name("SQLGEN_TABLE")
                .takes_value(true)
                .multiple(true)
                .help("Specify the table name(s)"),
        )
        .arg(
            Arg::with_name("schema")
                .short('s')
                .long("schema")
                .takes_value(true)
                .use_delimiter(true)
                .multiple(true)
                .help("Specify the schema name(s)"),
        )
        .arg(
            Arg::with_name("migrations")
                .short('m')
                .long("migrations")
                .default_value("migrations")
                .value_name("SQLGEN_MIGRATION_OUTPUT")
                .help("Sets the output folder for migrations")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("database")
                .short('d')
                .long("database")
                .default_value("docker")
                .value_name("DATABASE_URL")
                .help("Sets the database connection URL. Or use -d=docker to spin up a test contianer")
                .takes_value(true)
        );

    let matcher = App::new("SQL Gen")
        .subcommand(generate_subcommand)
        .subcommand(migrate_subcommand);
    let matches = matcher.get_matches();

    if let Some(matches) = matches.subcommand_matches("generate") {
        let database_is_docker = matches.value_of("database") == Some("docker");

        if let Some(input_migrations_folder) = matches.value_of("migrations").or({
            if database_is_docker {
                Some("migrations")
            } else {
                None
            }
        }) {
            println!(
                "Creating DB and applying migrations from {input_migrations_folder}"
            );

            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(get_test_db_string().await.as_str())
                .await
                .expect("could not create pool");

            let migrations_path = std::path::Path::new(input_migrations_folder);
            let migrator = sqlx::migrate::Migrator::new(migrations_path)
                .await
                .expect("Could not create migrations folder");

            migrator.run(&pool).await.expect("could not run migration");
        }

        println!("Done!");

        println!("getting output folder");

        let output_folder = matches
            .value_of("models")
            .expect("Could not get output modles folder");

        let context = matches.value_of("context");

        let mut database_url = matches
            .value_of("database")
            .expect("Must provide either a input migration folder or a database uri")
            .to_string();

        if database_url == "docker" {
            database_url = get_test_db_string().await;
        }

        let schemas: Option<Vec<&str>> =
            matches.values_of("schema").map(std::iter::Iterator::collect);
        let force = matches.is_present("force");
        generate::generate(output_folder, &database_url, context, force, None, schemas).await?;
    } else if let Some(matches) = matches.subcommand_matches("migrate") {
        let input_migrations_folder = matches.value_of("migrations").unwrap_or("./migrations");
        println!(
            "Creating DB and applying migrations from {input_migrations_folder}"
        );
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(get_test_db_string().await.as_str())
            .await
            .expect("could not create pool");

        let migrations_path = std::path::Path::new(input_migrations_folder);
        let migrator = sqlx::migrate::Migrator::new(migrations_path)
            .await
            .expect("Could not create migrations folder");
        migrator.run(&pool).await.expect("could not run migration");

        println!("Done!");

        let include_folder = matches
            .value_of("models")
            .expect("no models folder to include");
        let output_folder = matches
            .value_of("migrations")
            .expect("no migrations output");

        let mut database_url = matches
            .value_of("database")
            .expect("Must provide either a input migration folder or a database uri")
            .to_string();

        if database_url == "docker" {
            database_url = get_test_db_string().await;
        }

        // let tables: Option<Vec<&str>> = matches.values_of("table").map(|tables| tables.collect());
        let schemas: Option<Vec<&str>> =
            matches.values_of("schema").map(std::iter::Iterator::collect);

        println!("Finding new migration differences");
        migrate::migrate(include_folder, output_folder, &database_url, None, None).await?;
    }

    Ok(())
}
