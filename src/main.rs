use clap::{App, Arg, SubCommand};

mod db_queries;
mod generate;
mod migrate;
mod models;
mod query_generate;
mod utils;

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
            Arg::with_name("serde")
                .long("serde")
                .default_value("true")
                .value_name("SQLGEN_ENABLE_SERDE")
                .help("Adds Serde derices to created structs")
                .takes_value(false),
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
            Arg::with_name("exclude")
                .short('e')
                .long("exclude")
                .takes_value(true)
                .value_name("SQLGEN_EXCLUDE")
                .multiple(true)
                .use_delimiter(true)
                .help("Specify the excluded table name(s)"),
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

    if let Some(matches) = matches.subcommand_matches("generate") {
        let database_is_docker = matches.value_of("database") == Some("docker");

        if let Some(input_migrations_folder) = matches.value_of("migrations").or_else(|| {
            if database_is_docker {
                Some("migrations")
            } else {
                None
            }
        }) {
            println!(
                "Creating DB and applying migrations from {}",
                input_migrations_folder
            );

            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(test_container_db_uri.clone().expect("No db uri").as_str())
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
            .expect("Must provide either a input migration folder or a database uri");

        if database_url == "docker" {
            database_url = test_container_db_uri
                .as_deref()
                .expect("No docker database url");
        }

        let schemas: Option<Vec<&str>> =
            matches.values_of("schema").map(|schemas| schemas.collect());
        let force = matches.is_present("force");
        let include_tables = matches.values_of("table").map(|v| v.collect::<Vec<&str>>());
        let exclude_tables = matches
            .values_of("exclude")
            .map(|v| {
                v.into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or(vec![]);

        if !exclude_tables.is_empty() {
            println!("Excluding tables: {:?}", exclude_tables);
        }

        let enable_serde = matches.is_present("serde");

        generate::generate(
            enable_serde,
            output_folder,
            database_url,
            context,
            force,
            include_tables,
            exclude_tables,
            schemas,
        )
        .await?;
    } else if let Some(matches) = matches.subcommand_matches("migrate") {
        let input_migrations_folder = matches.value_of("migrations").unwrap_or("./migrations");
        println!(
            "Creating DB and applying migrations from {}",
            input_migrations_folder
        );
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(test_container_db_uri.clone().expect("No db uri").as_str())
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
            .expect("Must provide either a input migration folder or a database uri");

        if database_url == "docker" {
            database_url = test_container_db_uri
                .as_deref()
                .expect("No docker database url");
        }

        // let tables: Option<Vec<&str>> = matches.values_of("table").map(|tables| tables.collect());
        let schemas: Option<Vec<&str>> =
            matches.values_of("schema").map(|schemas| schemas.collect());

        println!("Finding new migration differences");
        migrate::migrate(include_folder, output_folder, database_url, None, None).await?;
    }

    Ok(())
}
