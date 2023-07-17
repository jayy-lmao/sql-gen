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
    let is_embedded = cfg!(feature = "embedded");

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
                .required(!is_embedded)
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
                .required(!is_embedded),
        );
    if is_embedded {
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

    let mut embedded_db_uri: Option<String> = None;

    if let Some(matches) = matches.subcommand_matches("generate") {
        #[cfg(feature = "embedded")]
        let mut pg_embed: Option<pg_embed::postgres::PgEmbed> = None;

        #[cfg(feature = "embedded")]
        if let Some(input_migrations_folder) = matches.value_of("migrations") {
            println!(
                "Creating DB and applying migrations from {}",
                input_migrations_folder
            );
            let (uri, pg) = migrate_to_temp_db(input_migrations_folder).await;
            embedded_db_uri = Some(uri);
            pg_embed = Some(pg);
            println!("Done!")
        };
        let output_folder = matches.value_of("output").unwrap();
        let context = matches.value_of("context");
        let database_url = matches
            .value_of("database")
            .or(embedded_db_uri.as_deref())
            .expect("Must provide either a input migration folder or a database uri");
        // let tables: Option<Vec<&str>> = matches.values_of("table").map(|tables| tables.collect());
        let schemas: Option<Vec<&str>> =
            matches.values_of("schema").map(|schemas| schemas.collect());
        let force = matches.is_present("force");
        generate::generate(output_folder, database_url, context, force, None, schemas).await?;

        #[cfg(feature = "embedded")]
        if let Some(mut pg) = pg_embed {
            pg.stop_db().await;
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
            embedded_db_uri = Some(uri);
            pg_embed = Some(pg);
            println!("Done!")
        };
        let include_folder = matches.value_of("include").unwrap();
        let output_folder = matches.value_of("output").unwrap();
        let database_url = matches
            .value_of("database")
            .or(embedded_db_uri.as_deref())
            .expect("Must provide either a input migration folder or a database uri");
        // let tables: Option<Vec<&str>> = matches.values_of("table").map(|tables| tables.collect());
        let schemas: Option<Vec<&str>> =
            matches.values_of("schema").map(|schemas| schemas.collect());
        migrate::migrate(include_folder, output_folder, database_url, None, None).await?;

        #[cfg(feature = "embedded")]
        if let Some(mut pg) = pg_embed {
            pg.stop_db().await;
        }
    }
    Ok(())
}

#[cfg(feature = "embedded")]
async fn migrate_to_temp_db(folder: &str) -> (String, pg_embed::postgres::PgEmbed) {
    use std::path::PathBuf;

    let pg_settings = pg_embed::postgres::PgSettings {
        // Where to store the postgresql database
        database_dir: PathBuf::from("data/db"),
        port: 5435,
        user: "postgres".to_string(),
        password: "password".to_string(),
        // authentication method
        auth_method: pg_embed::pg_enums::PgAuthMethod::Plain,
        // If persistent is false clean up files and directories on drop, otherwise keep them
        persistent: false,
        // duration to wait before terminating process execution
        // pg_ctl start/stop and initdb timeout
        // if set to None the process will not be terminated
        timeout: Some(std::time::Duration::from_secs(15)),
        // If migration sql scripts need to be run, the directory containing those scripts can be
        // specified here with `Some(PathBuf(path_to_dir)), otherwise `None` to run no migrations.
        // To enable migrations view the **Usage** section for details
        migration_dir: Some(PathBuf::from(folder)),
    };

    let fetch_settings = pg_embed::pg_fetch::PgFetchSettings {
        version: pg_embed::pg_fetch::PG_V13,
        ..Default::default()
    };
    let mut pg = pg_embed::postgres::PgEmbed::new(pg_settings, fetch_settings)
        .await
        .unwrap();

    // Download, unpack, create password file and database cluster
    println!("Setting up Postgres");
    pg.setup().await.unwrap();

    // start postgresql database
    println!("Starting Postgers");
    pg.start_db().await.unwrap();

    // create a new database
    // to enable migrations view the [Usage] section for details
    println!("Creating Database");
    pg.create_database("postgres").await.unwrap();
    let pg_db_uri: String = pg.full_db_uri("postgres");
    println!("Checking Database Exists");
    pg.database_exists("postgres").await.unwrap();

    // run migration sql scripts
    // to enable migrations view [Usage] for details
    println!("Migrating Database");
    pg.migrate("postgres").await.unwrap();

    // stop postgresql database
    (pg_db_uri, pg)
}
