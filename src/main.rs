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
    let using_test_containers = cfg!(feature = "test-containers");

    let mut generate_subcommand = SubCommand::with_name("generate")
        .about("Generate structs and queries for tables")
        .arg(
            Arg::with_name("output")
                .short('o')
                .long("output")
                .value_name("SQLGEN_MODEL_OUTPUT_FOLDER")
                .help("Sets the output folder for generated structs")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("database")
                .short('d')
                .long("database")
                .value_name("DATABASE_URL")
                .help("Sets the database connection URL")
                .required(!using_test_containers)
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

    let mut migrate_subcommand = SubCommand::with_name("migrate")
        .about("Generate SQL migrations based on struct differences")
        .arg(
            Arg::with_name("include")
                .short('i')
                .long("include")
                .value_name("SQLGEN_MODEL_FOLDER")
                .help("Sets the folder containing existing struct files")
                .takes_value(true)
                .required(true),
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
            Arg::with_name("output")
                .short('o')
                .long("output")
                .value_name("SQLGEN_MIGRATION_OUTPUT")
                .help("Sets the output folder for migrations")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("database")
                .short('d')
                .long("database")
                .value_name("DATABASE_URL")
                .help("Sets the database connection URL")
                .takes_value(true)
                .required(!using_test_containers),
        );
    if using_test_containers {
        generate_subcommand = generate_subcommand.arg(
            Arg::with_name("migrations")
                .short('m')
                .long("migrations")
                .value_name("SQLGEN_MIGRATIONS_INPUT")
                .help("The folder of migrations to apply")
                .takes_value(true),
        );
        migrate_subcommand = migrate_subcommand.arg(
            Arg::with_name("migrations")
                .short('m')
                .long("migrations")
                .value_name("SQLGEN_MIGRATIONS_INPUT")
                .help("The folder of migrations to apply")
                .takes_value(true),
        )
    };

    let matcher = App::new("SQL Gen")
        .subcommand(generate_subcommand)
        .subcommand(migrate_subcommand);
    let matches = matcher.get_matches();

    let mut test_container_db_uri: Option<String> = None;

    if let Some(matches) = matches.subcommand_matches("generate") {
        #[cfg(feature = "test-containers")]
        if let Some(input_migrations_folder) = matches.value_of("migrations") {
            println!(
                "Creating DB and applying migrations from {}",
                input_migrations_folder
            );
            let uri = migrate_to_temp_db(input_migrations_folder).await;
            test_container_db_uri = Some(uri);
            println!("Done!")
        };
        let output_folder = matches.value_of("output").unwrap();
        let context = matches.value_of("context");
        let database_url = matches
            .value_of("database")
            .or(test_container_db_uri.as_deref())
            .expect("Must provide either a input migration folder or a database uri");
        // let tables: Option<Vec<&str>> = matches.values_of("table").map(|tables| tables.collect());
        let schemas: Option<Vec<&str>> =
            matches.values_of("schema").map(|schemas| schemas.collect());
        let force = matches.is_present("force");
        generate::generate(output_folder, database_url, context, force, None, schemas).await?;

        #[cfg(feature = "embedded")]
        if let Some(mut pg) = pg_embed {
            pg.stop_db().await.unwrap();
        }
    } else if let Some(matches) = matches.subcommand_matches("migrate") {
        #[cfg(feature = "embedded")]
        let mut pg_embed: Option<pg_embed::postgres::PgEmbed> = None;
        #[cfg(feature = "embedded")]
        if let Some(input_migrations_folder) = matches.value_of("migrations") {
            println!(
                "Creating DB and applying migrations from {}",
                input_migrations_folder
            );
            let (uri, pg) = migrate_to_temp_db(input_migrations_folder).await;
            test_container_db_uri = Some(uri);
            pg_embed = Some(pg);
            println!("Done!")
        };
        let include_folder = matches.value_of("include").unwrap();
        let output_folder = matches.value_of("output").unwrap();
        let database_url = matches
            .value_of("database")
            .or(test_container_db_uri.as_deref())
            .expect("Must provide either a input migration folder or a database uri");
        // let tables: Option<Vec<&str>> = matches.values_of("table").map(|tables| tables.collect());
        let schemas: Option<Vec<&str>> =
            matches.values_of("schema").map(|schemas| schemas.collect());
        migrate::migrate(include_folder, output_folder, database_url, None, None).await?;

        #[cfg(feature = "embedded")]
        if let Some(mut pg) = pg_embed {
            pg.stop_db().await.unwrap();
        }
    }
    Ok(())
}

#[cfg(feature = "test-containers")]
async fn migrate_to_temp_db(folder: &str) -> String {
    // use testcontainers_modules::{postgres::Postgres, testcontainers::clients::Cli};

    let docker = testcontainers::clients::Cli::default();
    let node = docker.run(testcontainers_modules::postgres::Postgres::default());

    // prepare connection string
    let connection_string = &format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        node.get_host_port_ipv4(5432)
    );
    // container is up, you can use it
    // let mut conn = postgres::Client::connect(connection_string, postgres::NoTls).unwrap();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string)
        .await
        .unwrap();

    let rows = sqlx::query("SELECT 1 + 1").fetch_all(&pool).await.unwrap();
    assert_eq!(rows.len(), 1);

    let first_row = &rows[0];
    let first_column: i32 = sqlx::Row::get(first_row, 0);
    assert_eq!(first_column, 2);

    // stop postgresql database
    connection_string.to_string()
}
